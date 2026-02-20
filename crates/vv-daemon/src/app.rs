use crate::cli::Args;
use anyhow::Result;
use candle_core::Device;
use chrono::Local;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::signal::ctrl_c;
use tokio::sync::oneshot;
use tracing::{debug, error, info, warn};
use vv_capture::AudioCapture;
use vv_core::{Config, ConfigLoader, Note, TriggerKind, VvError};
use vv_stt::WhisperEngine;
use vv_trigger::Trigger;
use vv_vault::VaultWriter;

// For MVP, we only support PushToTalk trigger
use vv_trigger::PushToTalk;

/// Main application runner
pub struct App {
    args: Args,
}

impl App {
    /// Create a new application instance
    pub fn new(args: Args) -> Self {
        Self { args }
    }

    /// Run the main application loop
    pub async fn run(&self) -> Result<()> {
        info!("Starting VoiceVault");

        // Handle transcribe-only mode
        if let Some(audio_path) = &self.args.transcribe_only {
            return self.run_transcribe_only(audio_path).await;
        }

        info!("Starting daemon mode");

        // Load configuration
        let mut config = Self::load_config(&self.args)?;

        // Apply CLI overrides
        if let Some(_trigger) = &self.args.trigger {
            // TODO: Apply trigger override when trigger.mode exists
            debug!("Trigger override specified (not yet implemented)");
        }
        if let Some(model) = &self.args.model {
            config.stt.model = model.clone();
        }
        if let Some(vault_path) = &self.args.vault {
            config.vault.path = vault_path.clone();
        }

        // Validate configuration
        ConfigLoader::validate(&config)?;
        info!("Configuration validated");
        info!("Vault: {:?}", config.vault.path);
        info!("Model: {}", config.stt.model);

        // Initialize STT engine (use CPU for MVP, Metal support later)
        let device = Device::Cpu; // MVP: always use CPU
        info!("Using device: {:?}", device);

        info!("Loading Whisper model...");
        let stt_engine = Arc::new(Mutex::new(
            thread::spawn(move || {
                // Use None for hf_endpoint in MVP
                WhisperEngine::load(&config.stt.model, &device, None)
            })
            .join()
            .map_err(|e| {
                VvError::Transcription(format!("Failed to join loading thread: {:?}", e))
            })??,
        ));

        // Initialize audio capture
        let audio_capture = AudioCapture::new()?;

        // Initialize vault writer
        let vault_writer = Arc::new(Mutex::new(VaultWriter::new(&config.vault)?));

        // Build trigger - for MVP, only PushToTalk
        let trigger = PushToTalk::new();
        info!("Trigger: PushToTalk (press Enter to start/stop recording)");

        info!("VoiceVault ready");

        // Set up signal handling
        Self::main_loop(audio_capture, stt_engine, vault_writer, trigger).await?;

        info!("VoiceVault exited cleanly");
        Ok(())
    }

    /// Load configuration from file or use defaults
    fn load_config(args: &Args) -> Result<Config, VvError> {
        // Try to load from config file, or use defaults
        match ConfigLoader::load(args.config.clone()) {
            Ok(config) => Ok(config),
            Err(VvError::Config(_)) | Err(VvError::ConfigNotFound(_)) => {
                warn!("No config file found, using defaults");
                Ok(Config::default())
            },
            Err(e) => Err(e),
        }
    }

    /// Run in transcribe-only mode: load model, transcribe file, print result, exit
    async fn run_transcribe_only(&self, audio_path: &Path) -> Result<()> {
        info!("Transcribe-only mode: {}", audio_path.display());

        // Verify audio file exists
        if !audio_path.exists() {
            return Err(VvError::Transcription(format!(
                "Audio file not found: {}",
                audio_path.display()
            ))
            .into());
        }

        // Load minimal config for model selection
        let config = Self::load_config(&self.args)?;
        let model = self
            .args
            .model
            .clone()
            .unwrap_or_else(|| config.stt.model.clone());

        info!("Using model: {}", model);

        // Initialize STT engine
        let device = Device::Cpu;
        info!("Loading Whisper model...");

        let mut stt_engine = thread::spawn(move || WhisperEngine::load(&model, &device, None))
            .join()
            .map_err(|e| anyhow::anyhow!("Failed to join loading thread: {:?}", e))??;

        // Transcribe
        info!("Transcribing: {}", audio_path.display());
        let text = stt_engine.transcribe(audio_path)?;

        // Print result
        println!();
        println!("{}", text);
        println!();

        info!("Transcription complete");
        Ok(())
    }

    /// Main application loop
    async fn main_loop(
        _audio_capture: AudioCapture,
        stt_engine: Arc<Mutex<WhisperEngine>>,
        vault_writer: Arc<Mutex<VaultWriter>>,
        trigger: PushToTalk,
    ) -> Result<()> {
        loop {
            tokio::select! {
                // Wait for start signal
                result = async { trigger.wait_for_start().await } => {
                    result?;
                    info!("Recording started");

                    // Channel to receive audio samples
                    let (tx, rx) = mpsc::channel();
                    let (stop_tx, stop_rx) = oneshot::channel();

                    // Create a new audio capture instance for each recording
                    let audio_capture = match AudioCapture::new() {
                        Ok(capture) => capture,
                        Err(e) => {
                            error!("Failed to create audio capture: {}", e);
                            continue;
                        }
                    };

                    // Spawn audio capture in a separate thread
                    let capture_result = thread::spawn(move || {
                        audio_capture.record(tx, stop_rx)
                    });

                    // Wait for stop signal
                    match trigger.wait_for_stop().await {
                        Ok(()) => {
                            info!("Recording stopped");

                            // Send stop signal to capture thread
                            let _ = stop_tx.send(());

                            // Wait for capture thread to finish
                            if let Ok(Err(e)) = capture_result.join() {
                                warn!("Capture thread ended with error: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Error waiting for stop: {}", e);
                            let _ = stop_tx.send(());
                            let _ = capture_result.join();
                            continue;
                        }
                    }

                    // Collect all samples from the channel
                    // Give a small timeout to allow any pending samples to be delivered
                    let samples: Vec<f32> = rx.try_iter().flatten().collect();
                    let duration_secs = samples.len() as f32 / 16000.0;

                    if duration_secs < 0.1 {
                        warn!("Recording too short ({:.1}s), skipping", duration_secs);
                        continue;
                    }

                    info!("Collected {:.1}s of audio ({} samples)", duration_secs, samples.len());

                    // Write temporary WAV file
                    let temp_wav = PathBuf::from("/tmp/voicevault_recording.wav");
                    if let Err(e) = vv_capture::samples_to_wav(&samples, &temp_wav) {
                        error!("Failed to write WAV file: {}", e);
                        continue;
                    }

                    // Transcribe
                    info!("Transcribing...");
                    let temp_wav_clone = temp_wav.clone();
                    let stt_engine_clone = Arc::clone(&stt_engine);
                    let transcription_result = thread::spawn(move || {
                        let mut engine = stt_engine_clone.lock().unwrap();
                        engine.transcribe(&temp_wav_clone)
                    }).join();

                    let text = match transcription_result {
                        Ok(Ok(text)) => text,
                        Ok(Err(e)) => {
                            error!("Transcription failed: {}", e);
                            // Clean up temp file
                            let _ = std::fs::remove_file(&temp_wav);
                            continue;
                        }
                        Err(e) => {
                            error!("Transcription thread panicked: {:?}", e);
                            let _ = std::fs::remove_file(&temp_wav);
                            continue;
                        }
                    };

                    info!("Transcription: {}", text);

                    // Create note
                    let note = Note::new(
                        text.clone(),
                        Local::now(),
                        duration_secs as f64,
                        TriggerKind::PushToTalk,
                    );

                    // Write to vault
                    let vault_writer_clone = Arc::clone(&vault_writer);
                    match vault_writer_clone.lock().unwrap().write(&note) {
                        Ok(path) => {
                            info!("Saved note to: {}", path.display());
                        }
                        Err(e) => {
                            error!("Failed to write note: {}", e);
                        }
                    }

                    // Clean up temp file
                    let _ = std::fs::remove_file(&temp_wav);
                }

                // Handle Ctrl+C
                _ = ctrl_c() => {
                    info!("Received SIGINT, shutting down...");
                    break;
                }
            }
        }

        Ok(())
    }
}
