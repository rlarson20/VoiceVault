#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "need to figure out how to properly fix these errors; have similar issues with Volta pedantic"
)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, SupportedStreamConfig};
use std::sync::mpsc::Sender;
use thiserror::Error;
use tokio::sync::oneshot;
use tracing::{debug, error, info};
use vv_core::VvResult;

const WHISPER_SAMPLE_RATE: u32 = 16000;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("No audio input device found")]
    NoDevice,

    #[error("Failed to open audio device: {0}")]
    DeviceOpen(String),

    #[error("Unsupported audio format: {0:?}")]
    UnsupportedFormat(SampleFormat),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Play stream error: {0}")]
    PlayStream(String),
}

impl From<CaptureError> for vv_core::VvError {
    fn from(err: CaptureError) -> Self {
        vv_core::VvError::AudioCapture(err.to_string())
    }
}

pub struct AudioCapture {
    device: Device,
    config: SupportedStreamConfig,
}

impl AudioCapture {
    /// Create a new audio capture instance using the default input device
    /// # Errors
    /// `NoDevice`
    pub fn new() -> VvResult<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or(CaptureError::NoDevice)?;

        info!(
            "Using audio device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        let config = device
            .default_input_config()
            .map_err(|e| CaptureError::DeviceOpen(e.to_string()))?;

        debug!("Default input config: {:?}", config);

        Ok(Self { device, config })
    }

    /// Start recording audio
    /// Sends f32 mono 16kHz samples via the channel
    /// Stops when the stop receiver is signaled
    /// # Errors
    /// `UnsupportedFormat`
    pub fn record(&self, tx: &Sender<Vec<f32>>, stop: oneshot::Receiver<()>) -> VvResult<()> {
        let sample_format = self.config.sample_format();

        let sample_rate = self.config.config().sample_rate.0;
        let channels = self.config.config().channels;

        // Build the audio stream based on the device's sample format
        let stream = match sample_format {
            SampleFormat::F32 => {
                self.build_stream::<f32>(tx.clone(), stop, sample_rate, channels)?
            },
            SampleFormat::I16 => {
                self.build_stream::<i16>(tx.clone(), stop, sample_rate, channels)?
            },
            SampleFormat::U16 => {
                self.build_stream::<u16>(tx.clone(), stop, sample_rate, channels)?
            },
            _ => return Err(CaptureError::UnsupportedFormat(sample_format).into()),
        };

        // Start the stream
        stream
            .play()
            .map_err(|e| CaptureError::PlayStream(e.to_string()))?;

        // Keep the stream alive until stop signal
        // The stream's callback will handle sending samples
        info!("Recording started");

        // The stream runs on a background thread
        // We just need to keep this scope alive
        std::mem::forget(stream);

        Ok(())
    }

    fn build_stream<T>(
        &self,
        tx: Sender<Vec<f32>>,
        mut stop: oneshot::Receiver<()>,
        source_sample_rate: u32,
        source_channels: u16,
    ) -> Result<cpal::Stream, CaptureError>
    where
        T: cpal::Sample + cpal::SizedSample + Into<f32>,
    {
        let err_fn = |err| error!("Audio stream error: {}", err);

        // Build stream callback
        let callback = move |data: &[T], _: &cpal::InputCallbackInfo| {
            // Check for stop signal (non-blocking)
            match stop.try_recv() {
                Ok(()) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    // Stop signal received, don't send more data
                    return;
                },
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {},
            }

            // Convert to f32 and resample if needed
            let samples: Vec<f32> = data.iter().map(|&s| s.into()).collect();

            // Downmix to mono if needed
            let mono_samples: Vec<f32> = if source_channels > 1 {
                samples
                    .chunks(source_channels as usize)
                    .map(|chunk| chunk.iter().sum::<f32>() / f32::from(source_channels))
                    .collect()
            } else {
                samples
            };

            // Resample to 16kHz if needed (simple linear interpolation for MVP)
            let resampled = if source_sample_rate == WHISPER_SAMPLE_RATE {
                mono_samples
            } else {
                linear_resample(&mono_samples, source_sample_rate, WHISPER_SAMPLE_RATE)
            };

            // Send to channel (ignore errors if receiver is closed)
            let _ = tx.send(resampled);
        };

        let config = self.config.clone();
        self.device
            .build_input_stream(&config.into(), callback, err_fn, None)
            .map_err(|e| CaptureError::Stream(e.to_string()))
    }
}

/// Simple linear resampler
/// For production, consider using a dedicated resampling library
fn linear_resample(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return input.to_vec();
    }

    let ratio = f64::from(from_rate) / f64::from(to_rate);
    let output_len = ((input.len() as f64) / ratio).ceil() as usize;

    (0..output_len)
        .map(|i| {
            let src_idx = (i as f64 * ratio) as usize;
            let next_idx = (src_idx + 1).min(input.len() - 1);
            let frac = (i as f64 * ratio) - src_idx as f64;

            // Linear interpolation
            input[src_idx] * (1.0 - frac) as f32 + input[next_idx] * frac as f32
        })
        .collect()
}

/// Write samples as a WAV file
/// # Errors
/// Result
pub fn samples_to_wav(samples: &[f32], path: &std::path::Path) -> VvResult<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: WHISPER_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let writer = hound::WavWriter::create(path, spec)
        .map_err(|e| vv_core::VvError::AudioCapture(format!("Failed to create WAV: {e}")))?;

    let mut writer = writer;

    for &sample in samples {
        // Clamp to [-1.0, 1.0] and convert to i16
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_sample = (clamped * f32::from(i16::MAX)) as i16;
        writer
            .write_sample(i16_sample)
            .map_err(|e| vv_core::VvError::AudioCapture(format!("Failed to write sample: {e}")))?;
    }

    writer
        .finalize()
        .map_err(|e| vv_core::VvError::AudioCapture(format!("Failed to finalize WAV: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_resample_same_rate() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let output = linear_resample(&input, 16000, 16000);
        assert_eq!(output, input);
    }

    #[test]
    fn test_linear_resample_downsample() {
        let input = vec![0.0, 1.0, 2.0, 3.0];
        let output = linear_resample(&input, 48000, 16000);
        // Should be approximately 1/3 the length
        assert!((output.len() as f32 - (input.len() as f32 / 3.0)).abs() < 2.0);
    }

    #[test]
    fn test_samples_to_wav_roundtrip() {
        let samples = vec![0.0, 0.5, 1.0, -0.5, -1.0];
        let temp_file = tempfile::NamedTempFile::new().unwrap();

        samples_to_wav(&samples, temp_file.path()).unwrap();

        // Read back and verify
        let reader = hound::WavReader::open(temp_file.path()).unwrap();
        let read_samples: Vec<f32> = reader
            .into_samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect();

        assert_eq!(read_samples.len(), samples.len());
        for (orig, read) in samples.iter().zip(read_samples.iter()) {
            assert!((orig - read).abs() < 0.01); // Allow small rounding errors
        }
    }
}
