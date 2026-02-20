# VoiceVault

A Rust-based voice capture tool that records audio, transcribes it locally via [Whisper](https://github.com/openai/whisper), and writes [Obsidian](https://obsidian.md/)-compatible Markdown notes to your vault.

## Features

- **Local Speech-to-Text**: Uses OpenAI's Whisper model running entirely on your machine - no API calls, no privacy concerns
- **Obsidian Integration**: Writes notes directly to your vault with proper frontmatter for indexing
- **Multiple Trigger Modes**:
  - **GlobalHotkey**: Press a hotkey combination to start/stop recording (desktop use)
  - **PushToTalk**: Press Enter in the terminal to start/stop recording
  - **HttpEndpoint**: Control recording via HTTP requests (remote control)
  - **UnixSocket**: Control recording via Unix domain socket (IPC)
- **Transcribe-Only Mode**: Transcribe existing audio files without recording
- **Cross-Platform**: macOS, Linux (Windows support planned)

## Installation

### Prerequisites

**Linux only:**
```bash
sudo apt-get install libasound2-dev
# or on Fedora:
sudo dnf install alsa-lib-devel
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/VoiceVault.git
cd VoiceVault

# Build the project
cargo build --release

# The binary will be at target/release/voicevault
```

### Install with Cargo

```bash
cargo install --path .
```

## Configuration

VoiceVault looks for configuration files in the following order (first found wins):

1. `--config <path>` - Path specified via CLI argument
2. `VOICEVAULT_CONFIG` - Environment variable
3. `~/.config/voicevault/config.toml` - Default XDG config location

### Example Configuration

```toml
# Copy this to ~/.config/voicevault/config.toml

[vault]
# Path to your Obsidian vault or notes directory
path = "~/Documents/Vault"

# Folder within vault where recordings are stored
folder = "voice-notes"

# Filename format (strftime-compatible)
filename_format = "%Y-%m-%dT%H-%M-%S"

[stt]
# Whisper model to use
# Options: tiny, tiny.en, base, base.en, small, small.en, medium, medium.en, large, large-v3
model = "base"

[trigger]
# Push-to-talk mode (press a key to start/stop recording)
push_to_talk = true

# Global hotkey mode (requires global_hotkey feature)
global_hotkey = true
hotkey = "Alt+Shift+V"

# HTTP endpoint mode (requires http_endpoint feature)
http_endpoint = true
http_port = 8765

# Unix socket mode (requires unix_socket feature)
unix_socket = true
socket_path = "/tmp/voicevault.sock"

[audio]
# Sample rate in Hz
sample_rate = 16000

# Number of audio channels (1 = mono, 2 = stereo)
channels = 1
```

## Usage

### Basic Usage (PushToTalk Mode)

```bash
# Start the daemon (will prompt you to press Enter to record)
voicevault

# Or with a specific config file
voicevault --config path/to/config.toml
```

### GlobalHotkey Mode

```bash
# Start with global hotkey enabled
voicevault

# Press your configured hotkey (e.g., Alt+Shift+V) to start recording
# Release the hotkey to stop recording
```

**Platform Requirements:**
- **macOS**: You'll be prompted to grant Accessibility permission on first use
- **Linux**: Your user must be in the `input` group:
  ```bash
  sudo usermod -a -G input $USER
  # Log out and back in for changes to take effect
  ```

### HttpEndpoint Mode

```bash
# Start with HTTP endpoint enabled
voicevault

# In another terminal, control recording:
curl http://localhost:8765/status          # Check recording status
curl -X POST http://localhost:8765/start   # Start recording
curl -X POST http://localhost:8765/stop    # Stop recording
```

### UnixSocket Mode

```bash
# Start with Unix socket enabled
voicevault

# In another terminal, control recording:
echo '{"cmd":"start"}' | nc -U /tmp/voicevault.sock
echo '{"cmd":"stop"}' | nc -U /tmp/voicevault.sock
```

### Transcribe-Only Mode

Transcribe an existing audio file without recording:

```bash
voicevault --once --transcribe-only path/to/audio.wav
```

## Model Selection Guide

Whisper models vary in size, speed, and accuracy. Here's a comparison:

| Model   | Size  | Speed   | Accuracy | Recommended For                          |
|---------|-------|---------|----------|------------------------------------------|
| tiny    | ~39MB | Fastest | Lowest   | Quick notes, testing                     |
| tiny.en | ~39MB | Fastest | Low      | English-only quick notes                  |
| base    | ~74MB | Fast    | Medium   | Daily use (recommended)                   |
| base.en | ~74MB | Fast    | Good     | English-only daily use                    |
| small   | ~244MB| Medium  | Good     | Better accuracy needed                    |
| small.en| ~244MB| Medium  | High     | English-only, better accuracy             |
| medium  | ~769MB| Slow    | High     | Professional use                          |
| medium.en| ~769MB| Slow    | Very High| English-only professional use              |
| large   | ~1.5GB| Slowest | Best     | Maximum accuracy (not recommended)        |
| large-v3| ~1.5GB| Slowest | Best     | Latest large model (not recommended)       |

**Recommendation**: Start with `base` or `base.en` for English-only use. They provide good accuracy with reasonable speed.

## Output Format

VoiceVault creates Markdown files with Obsidian-compatible frontmatter:

```markdown
---
trigger: Push-to-Talk
duration_seconds: 5.2
recorded_at: 2025-02-19 10:30:00 +00:00
---

This is the transcribed text from your voice recording.
```

The `trigger` field indicates which trigger mode initiated the recording:
- `Global Hotkey`
- `Push-to-Talk`
- `HTTP Endpoint`
- `Unix Socket`

## Platform-Specific Notes

### macOS

1. **Accessibility Permission**: When using GlobalHotkey mode, macOS will prompt you to grant Accessibility permission to the VoiceVault binary. This is required for global hotkey listening.

2. **Metal GPU Acceleration**: On Apple Silicon Macs, VoiceVault can use Metal acceleration for faster transcription. This is automatically enabled when available.

### Linux

1. **Input Group**: For GlobalHotkey mode, add your user to the `input` group:
   ```bash
   sudo usermod -a -G input $USER
   ```
   Then log out and back in.

2. **ALSA Dependencies**: Ensure you have `libasound2-dev` (Debian/Ubuntu) or `alsa-lib-devel` (Fedora) installed.

3. **Device Permissions**: Make sure your user has permission to access audio devices (typically in the `audio` group).

## Troubleshooting

### "No config file found"

Create a configuration file at `~/.config/voicevault/config.toml` or specify one with `--config`.

### "Failed to initialize audio capture"

- **Linux**: Ensure you're in the `audio` group: `groups | grep audio`
- Check that no other application is using the microphone
- Try specifying a different sample rate in the config

### "Vault path not found"

Ensure the vault directory exists in your config file. VoiceVault won't create the vault directory itself, but it will create the notes folder within it.

### "Hotkey not working"

- **macOS**: Grant Accessibility permission in System Settings > Privacy & Security > Accessibility
- **Linux**: Ensure you're in the `input` group
- Try a different hotkey combination (some combinations may be reserved by the system)

### Transcription is slow

- Try a smaller model (e.g., `tiny` or `base`)
- On macOS, ensure Metal acceleration is working (check logs for "Using Metal acceleration")
- Close other applications that might be using CPU/GPU

### Transcription is inaccurate

- Try a larger model (e.g., `small` or `medium`)
- Speak clearly and at a moderate pace
- Ensure you're in a quiet environment
- Check that your microphone is working properly

## Debug Logging

Enable verbose logging to troubleshoot issues:

```bash
# Set RUST_LOG environment variable
RUST_LOG=debug voicevault

# For even more detailed logging
RUST_LOG=trace voicevault
```

## Development

### Build

```bash
cargo build --workspace
```

### Run Tests

```bash
cargo test --workspace
```

### Run Clippy

```bash
cargo clippy --workspace --deny warnings
```

### Format Code

```bash
cargo fmt --all
```

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [OpenAI Whisper](https://github.com/openai/whisper) - The speech-to-text model
- [Candle](https://github.com/huggingface/candle) - ML framework used for inference
- [rdev](https://github.com/NyxCode/rdev) - Global hotkey listening
- [Obsidian](https://obsidian.md) - The note-taking app this project integrates with
