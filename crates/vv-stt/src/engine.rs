use byteorder::ByteOrder;
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use hf_hub::{Repo, RepoType, api::sync::Api};
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::info;
use vv_core::{VvError, VvResult};

use candle_transformers::models::whisper::{self, audio as whisper_audio};

/// Task type for Whisper model
pub enum Task {
    Transcribe,
    Translate,
}

/// Configuration for decoding/transcription
pub struct DecodingConfig {
    pub language: Option<String>,
    pub task: Task,
}

/// Whisper speech-to-text engine
pub struct WhisperEngine {
    model: whisper::model::Whisper,
    tokenizer: Tokenizer,
    device: Device,
    mel_filters: Vec<f32>,
    config: whisper::Config,
}

impl WhisperEngine {
    /// Load a Whisper model from HuggingFace Hub
    /// Downloads and caches the model on first run
    pub fn load(model_id: &str, device: &Device, _hf_endpoint: Option<&str>) -> VvResult<Self> {
        info!("Loading Whisper model: {}", model_id);

        // Map model name to HF repo
        let hf_repo = match model_id {
            "tiny.en" => "openai/whisper-tiny.en",
            "tiny" => "openai/whisper-tiny",
            "base.en" => "openai/whisper-base.en",
            "base" => "openai/whisper-base",
            "small.en" => "openai/whisper-small.en",
            "small" => "openai/whisper-small",
            "medium.en" => "openai/whisper-medium.en",
            "medium" => "openai/whisper-medium",
            "large-v3" => "openai/whisper-large-v3",
            "large" => "openai/whisper-large",
            _ => return Err(VvError::UnknownModel(model_id.to_string())),
        };

        // Set up HF API
        let api = Api::new().map_err(|e| {
            VvError::Transcription(format!("Failed to create HuggingFace API: {}", e))
        })?;

        let api = api.repo(Repo::new(hf_repo.to_string(), RepoType::Model));

        // Download model files
        info!("Downloading model files from HuggingFace Hub...");
        let config_filename = api.get("config.json").map_err(|e| {
            VvError::Transcription(format!("Failed to download config.json: {}", e))
        })?;

        let tokenizer_filename = api.get("tokenizer.json").map_err(|e| {
            VvError::Transcription(format!("Failed to download tokenizer.json: {}", e))
        })?;

        let model_filename = api.get("model.safetensors").map_err(|e| {
            VvError::Transcription(format!("Failed to download model.safetensors: {}", e))
        })?;

        // Load config
        let config: whisper::Config =
            serde_json::from_str(&std::fs::read_to_string(config_filename).map_err(|e| {
                VvError::Transcription(format!("Failed to read config.json: {}", e))
            })?)
            .map_err(|e| VvError::Transcription(format!("Failed to parse config.json: {}", e)))?;

        // Load tokenizer
        let tokenizer = Tokenizer::from_file(tokenizer_filename)
            .map_err(|e| VvError::Transcription(format!("Failed to load tokenizer: {}", e)))?;

        // Load mel filters (hardcoded for 80 mel bins)
        let mel_bytes = include_bytes!("melfilters.bytes");
        let mut mel_filters = vec![0f32; mel_bytes.len() / 4];
        byteorder::LittleEndian::read_f32_into(mel_bytes, &mut mel_filters);

        // Load model weights
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[model_filename], whisper::DTYPE, device)
        }
        .map_err(|e| VvError::Transcription(format!("Failed to load model weights: {}", e)))?;

        let model = whisper::model::Whisper::load(&vb, config.clone()).map_err(|e| {
            VvError::Transcription(format!("Failed to create Whisper model: {}", e))
        })?;

        info!("Whisper model loaded successfully");
        Ok(Self {
            model,
            tokenizer,
            device: device.clone(),
            mel_filters,
            config,
        })
    }

    /// Transcribe a WAV audio file
    /// Runs in a blocking thread (candle is sync), so use with tokio::task::spawn_blocking
    pub fn transcribe(&mut self, wav_path: &Path) -> VvResult<String> {
        info!("Transcribing: {:?}", wav_path);

        // Read WAV file
        let (pcm_data, sample_rate) = self.decode_wav(wav_path)?;

        // Verify sample rate
        if sample_rate != whisper::SAMPLE_RATE as u32 {
            return Err(VvError::Transcription(format!(
                "Audio must be {}Hz, got {}Hz",
                whisper::SAMPLE_RATE,
                sample_rate
            )));
        }

        // Convert to mel spectrogram
        let mel_data = whisper_audio::pcm_to_mel(&self.config, &pcm_data, &self.mel_filters);
        let mel_len = mel_data.len();
        let mel = Tensor::from_vec(
            mel_data,
            (
                1,
                self.config.num_mel_bins,
                mel_len / self.config.num_mel_bins,
            ),
            &self.device,
        )
        .map_err(|e| {
            VvError::Transcription(format!("Failed to create mel spectrogram tensor: {}", e))
        })?;

        // Run encoder
        let audio_features =
            self.model.encoder.forward(&mel, true).map_err(|e| {
                VvError::Transcription(format!("Encoder forward pass failed: {}", e))
            })?;

        // Decode using greedy sampling
        let tokens = self.decode_greedy(&audio_features)?;

        // Decode tokens to text
        let text = self
            .tokenizer
            .decode(&tokens, true)
            .map_err(|e| VvError::Transcription(format!("Tokenization failed: {}", e)))?;

        Ok(text)
    }

    /// Transcribe with custom decoding config
    pub fn transcribe_with_config(
        &mut self,
        wav_path: &Path,
        _config: &DecodingConfig,
    ) -> VvResult<String> {
        // For now, just use the basic transcribe
        // TODO: Apply custom language/task config when needed
        self.transcribe(wav_path)
    }

    /// Decode WAV file to PCM data
    fn decode_wav(&self, wav_path: &Path) -> VvResult<(Vec<f32>, u32)> {
        // Read the WAV file
        let wav_data = std::fs::read(wav_path)
            .map_err(|e| VvError::Transcription(format!("Failed to read WAV file: {}", e)))?;

        // Parse WAV header to get format info
        let mut cursor = std::io::Cursor::new(wav_data.as_slice());

        // Verify RIFF header
        let mut riff_header = [0u8; 12];
        cursor
            .read_exact(&mut riff_header)
            .map_err(|e| VvError::Transcription(format!("Failed to read WAV header: {}", e)))?;

        if &riff_header[0..4] != b"RIFF" || &riff_header[8..12] != b"WAVE" {
            return Err(VvError::Transcription(
                "Invalid WAV file: missing RIFF/WAVE headers".to_string(),
            ));
        }

        // Find fmt chunk
        let mut sample_rate = 0u32;
        let mut bits_per_sample = 0u16;
        let mut data_offset = 0;
        let mut data_size = 0;

        loop {
            let mut chunk_header = [0u8; 8];
            if cursor.read_exact(&mut chunk_header).is_err() {
                break;
            }

            let chunk_id = String::from_utf8_lossy(&chunk_header[0..4]);
            let chunk_size = u32::from_le_bytes([
                chunk_header[4],
                chunk_header[5],
                chunk_header[6],
                chunk_header[7],
            ]) as usize;

            match chunk_id.as_ref() {
                "fmt " => {
                    // Skip format tag and channels (2 + 2 bytes)
                    cursor.set_position(cursor.position() + 4);
                    let mut sr_bytes = [0u8; 4];
                    cursor.read_exact(&mut sr_bytes).map_err(|e| {
                        VvError::Transcription(format!("Failed to read sample rate: {}", e))
                    })?;
                    sample_rate = u32::from_le_bytes(sr_bytes);

                    // Skip byte rate and block align (4 + 2 bytes)
                    cursor.set_position(cursor.position() + 6);
                    let mut bps_bytes = [0u8; 2];
                    cursor.read_exact(&mut bps_bytes).map_err(|e| {
                        VvError::Transcription(format!("Failed to read bits per sample: {}", e))
                    })?;
                    bits_per_sample = u16::from_le_bytes(bps_bytes);

                    // Skip any extra bytes
                    let extra_bytes = chunk_size.saturating_sub(16);
                    cursor.set_position(cursor.position() + extra_bytes as u64);
                },
                "data" => {
                    data_offset = cursor.position() as usize;
                    data_size = chunk_size;
                    break;
                },
                _ => {
                    // Skip unknown chunk
                    cursor.set_position(cursor.position() + chunk_size as u64);
                },
            }
        }

        if sample_rate == 0 || data_offset == 0 {
            return Err(VvError::Transcription(
                "Invalid WAV file: missing sample rate or data chunk".to_string(),
            ));
        }

        // Calculate end of data
        let data_end = (data_offset + data_size).min(wav_data.len());

        // Read PCM data
        let pcm_bytes = &wav_data[data_offset..data_end];
        let pcm_data = match bits_per_sample {
            16 => {
                let samples: Vec<i16> = pcm_bytes
                    .chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect();
                samples.into_iter().map(|s| s as f32 / 32768.0).collect()
            },
            _ => {
                return Err(VvError::Transcription(format!(
                    "Unsupported bits per sample: {} (only 16-bit PCM supported)",
                    bits_per_sample
                )));
            },
        };

        Ok((pcm_data, sample_rate))
    }

    /// Simple greedy decoding (no temperature fallback)
    fn decode_greedy(&mut self, audio_features: &Tensor) -> VvResult<Vec<u32>> {
        // Get special tokens
        let sot_token = self
            .get_token_id(whisper::SOT_TOKEN)
            .ok_or_else(|| VvError::Transcription("SOT token not found".to_string()))?;
        let eot_token = self
            .get_token_id(whisper::EOT_TOKEN)
            .ok_or_else(|| VvError::Transcription("EOT token not found".to_string()))?;
        let transcribe_token = self
            .get_token_id(whisper::TRANSCRIBE_TOKEN)
            .ok_or_else(|| VvError::Transcription("Transcribe token not found".to_string()))?;
        let no_timestamps_token = self
            .get_token_id(whisper::NO_TIMESTAMPS_TOKEN)
            .ok_or_else(|| VvError::Transcription("No timestamps token not found".to_string()))?;

        let sample_len = self.config.max_target_positions;
        let mut tokens = vec![sot_token, transcribe_token, no_timestamps_token];

        for i in 0..sample_len {
            let tokens_tensor = Tensor::new(tokens.as_slice(), &self.device)
                .map_err(|e| {
                    VvError::Transcription(format!("Failed to create tokens tensor: {}", e))
                })?
                .unsqueeze(0)
                .map_err(|e| {
                    VvError::Transcription(format!("Failed to unsqueeze tokens: {}", e))
                })?;

            let decoder_output = self
                .model
                .decoder
                .forward(&tokens_tensor, audio_features, i == 0)
                .map_err(|e| {
                    VvError::Transcription(format!("Decoder forward pass failed: {}", e))
                })?;

            let (_batch_size, seq_len, _vocab_size) = decoder_output.dims3().map_err(|e| {
                VvError::Transcription(format!("Failed to get decoder output dims: {}", e))
            })?;

            let logits = decoder_output
                .i((..1, seq_len - 1..))
                .map_err(|e| VvError::Transcription(format!("Failed to slice logits: {}", e)))?
                .i(0)
                .map_err(|e| VvError::Transcription(format!("Failed to remove batch dim: {}", e)))?
                .i(0)
                .map_err(|e| VvError::Transcription(format!("Failed to remove seq dim: {}", e)))?;

            let next_token = logits
                .argmax(0)
                .map_err(|e| VvError::Transcription(format!("Argmax failed: {}", e)))?
                .to_scalar::<u32>()
                .map_err(|e| VvError::Transcription(format!("Failed to get token id: {}", e)))?;

            tokens.push(next_token);

            if next_token == eot_token || tokens.len() >= self.config.max_target_positions {
                break;
            }
        }

        Ok(tokens)
    }

    /// Get token ID from tokenizer
    fn get_token_id(&self, token: &str) -> Option<u32> {
        self.tokenizer.token_to_id(token)
    }
}

// Add std::io::Read import for cursor operations
use std::io::Read;
