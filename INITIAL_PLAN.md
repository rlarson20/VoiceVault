# VoiceVault — Rust Rewrite Plan

## Context

VoiceVault is a voice capture tool that records audio, transcribes it locally via
Whisper, and writes the result as a Markdown note to an Obsidian vault. The existing
Python codebase is being discarded entirely. This is a ground-up Rust rewrite.

Before starting, the human has already created a fresh git branch. Begin from that
branch and do not touch any existing Python files — they can be deleted or ignored.
UPDATE: the branch has been cleaned of everything sans a README, a test audio file, and the git files.

After each step described below at implementation order, make sure to check in with the user.

---

## Goals

- Sub-second capture initiation from any trigger mode
- Fully local STT (no network required) via Whisper through `candle`
- Cross-platform: macOS (CoreAudio) and Linux (ALSA/PipeWire) from day one
- Multiple capture trigger modes selectable at runtime via config or CLI flag
- Obsidian-compatible Markdown output written to a user-configured vault path
- Clean trait boundaries so STT backend and vault writer are swappable later

---

## Workspace Structure

Use a Cargo workspace. Do not use a single-crate monolith.

```
voicevault/
├── Cargo.toml                  # workspace manifest
├── config.example.toml         # example config file, committed to repo
├── crates/
│   ├── vv-core/                # shared types, config schema, errors
│   ├── vv-capture/             # audio recording via cpal
│   ├── vv-stt/                 # Whisper inference via candle
│   ├── vv-trigger/             # trigger mode enum + handler impls
│   ├── vv-vault/               # Obsidian markdown writer
│   └── vv-daemon/              # main binary — wires everything together
└── tests/                      # integration tests
```

---

## Dependency Decisions

These are not suggestions — use these exact crates unless a hard blocker is found,
in which case document why in a `DECISIONS.md` at the repo root.

| Purpose             | Crate                                             | Notes                                                  |
| ------------------- | ------------------------------------------------- | ------------------------------------------------------ |
| Audio I/O           | `cpal = "0.16"`                                   | Apache-2.0; CoreAudio on macOS, ALSA on Linux          |
| Whisper inference   | `candle-core`, `candle-nn`, `candle-transformers` | HF-maintained; Metal on macOS, CPU fallback everywhere |
| Global hotkey       | `hotkey-listener`                                 | evdev on Linux (X11+Wayland), rdev on macOS            |
| Async runtime       | `tokio` (full features)                           |                                                        |
| CLI parsing         | `clap` (derive feature)                           |                                                        |
| Config file         | `serde` + `toml`                                  |                                                        |
| Error handling      | `thiserror` (library crates), `anyhow` (binary)   |                                                        |
| Logging             | `tracing` + `tracing-subscriber`                  |                                                        |
| WAV encoding        | `hound`                                           | for buffering audio before STT                         |
| HTTP trigger server | `axum`                                            | only compiled when HttpEndpoint trigger is used        |
| Unix socket IPC     | std `UnixListener`                                | no extra dep needed                                    |

**macOS-specific note:** Enable candle's `metal` feature behind a `cfg` — do not
make it unconditional. Use a Cargo feature flag `metal` on `vv-stt` that downstream
can opt into.

**Linux-specific note:** `hotkey-listener` uses evdev on Linux which reads from
`/dev/input`. The user's account must be in the `input` group. Document this in
`README.md`.

---

## crates/vv-core

Shared types used by all other crates. No business logic here.

### Config schema (`config.rs`)

```toml
# config.example.toml
[vault]
path = "/Users/you/Notes"
folder = "captures"          # subfolder within vault; created if absent
filename_format = "%Y-%m-%dT%H-%M-%S"  # strftime-compatible

[stt]
model = "small.en"           # tiny.en | base.en | small.en | medium.en | large-v3
device = "cpu"               # cpu | metal (macOS only)

[trigger]
mode = "GlobalHotkey"        # GlobalHotkey | PushToTalk | HttpEndpoint | UnixSocket
hotkey = "Alt+Shift+V"       # used when mode = GlobalHotkey
http_port = 8765             # used when mode = HttpEndpoint
socket_path = "/tmp/voicevault.sock"  # used when mode = UnixSocket
```

Config is loaded from (in priority order):

1. `--config <path>` CLI flag
2. `$VOICEVAULT_CONFIG` env var
3. `~/.config/voicevault/config.toml`
4. Hard error — do not silently use defaults for vault path or model

Implement `Config` as a plain `serde`-derived struct. Expose a `Config::load(path:
Option<PathBuf>) -> Result<Config>` function that handles the search order above.

### Error type (`error.rs`)

Define a top-level `VvError` using `thiserror`. Each crate defines its own error
type and `vv-core` provides the common ones (config parse errors, vault write errors).

### Note type (`note.rs`)

```rust
pub struct Note {
    pub text: String,
    pub recorded_at: chrono::DateTime<chrono::Local>,
    pub duration_secs: f32,
    pub trigger: TriggerKind,
}
```

`Note::to_markdown(&self) -> String` renders the note as Obsidian-compatible
frontmatter + body:

```markdown
---
created: 2026-02-19T17:32:00
duration: 12.4s
trigger: GlobalHotkey
tags: [voice-capture]
---

<transcribed text here>
```

---

## crates/vv-capture

Owns all audio recording logic. No STT here.

### Public API

```rust
pub struct AudioCapture { ... }

impl AudioCapture {
    pub fn new() -> Result<Self>;
    /// Blocks the calling thread; writes f32 mono 16kHz PCM samples into
    /// a channel. Caller sends a stop signal via the returned sender.
    pub fn record(
        &self,
        tx: mpsc::Sender<Vec<f32>>,
        stop: oneshot::Receiver<()>,
    ) -> Result<()>;
}
```

Implementation notes:

- Use `cpal` to open the default input device
- Resample or configure to 16kHz mono f32 — Whisper expects this exactly
- If the device doesn't natively support 16kHz, implement a simple linear
  resampler (do not add a full resampling library for MVP)
- Buffer chunks of ~100ms and send over the channel
- On stop signal, flush remaining buffer and close the channel

### WAV writing utility

Expose `pub fn samples_to_wav(samples: &[f32], path: &Path) -> Result<()>` using
`hound`. This is used by `vv-stt` which reads from file (matching candle's example
pattern) for the MVP. A streaming interface can come later.

---

## crates/vv-stt

Whisper inference via `candle`. Treat the
[candle whisper example](https://github.com/huggingface/candle/tree/main/candle-examples/examples/whisper)
as the primary reference — read it before writing any code here.

### Public API

```rust
pub struct WhisperEngine {
    model: Whisper,
    tokenizer: Tokenizer,
    config: DecodingConfig,
    device: Device,
}

pub struct DecodingConfig {
    pub language: Option<String>,   // None = auto-detect
    pub task: Task,                 // Transcribe | Translate
}

impl WhisperEngine {
    pub fn load(model_id: &str, device: &Device) -> Result<Self>;
    pub async fn transcribe(&self, wav_path: &Path) -> Result<String>;
}
```

Model loading:

- Use `hf-hub` to download model weights from HF Hub on first run
- Cache in `~/.cache/voicevault/models/`
- Supported model IDs: `tiny.en`, `base.en`, `small.en`, `medium.en`, `large-v3`
- Run inference in a `tokio::task::spawn_blocking` block — candle is sync

Device selection:

- `cpu` always works
- `metal` only on macOS; guard with `#[cfg(feature = "metal")]` and `candle_core::MetalDevice`
- Log device selection at startup with `tracing::info!`

---

## crates/vv-trigger

Defines the `Trigger` trait and all trigger mode implementations.

### Trait

```rust
#[async_trait]
pub trait Trigger: Send + Sync {
    /// Waits for the user to initiate a recording, then returns.
    async fn wait_for_start(&self) -> Result<()>;
    /// Waits for the user to end the recording, then returns.
    async fn wait_for_stop(&self) -> Result<()>;
}
```

### TriggerMode enum (mirrors config)

```rust
pub enum TriggerMode {
    GlobalHotkey { hotkey: String },
    PushToTalk,
    HttpEndpoint { port: u16 },
    UnixSocket { path: PathBuf },
}

pub fn build_trigger(mode: &TriggerMode) -> Box<dyn Trigger>;
```

### Implementations

**GlobalHotkey** — use `hotkey-listener`. Hold = recording, release = stop.
Note: On macOS, the process needs Accessibility API permission. Print a clear
actionable error message if permission is denied, not a panic.

**PushToTalk** — readline from stdin: Enter to start, Enter again to stop.
Useful for SSH/headless environments. No external deps.

**HttpEndpoint** — `axum` server with two routes:

```
POST /start  → begins recording
POST /stop   → ends recording, triggers transcription
GET  /status → returns JSON { recording: bool }
```

This is the integration point for `tg_hyper_librarian` and other future clients.

**UnixSocket** — listen on a Unix domain socket. Accepts newline-delimited
JSON messages: `{"cmd": "start"}`, `{"cmd": "stop"}`. Simple protocol, no framing
library needed.

---

## crates/vv-vault

Writes `Note`s to the Obsidian vault.

### Public API

```rust
pub struct VaultWriter {
    vault_path: PathBuf,
    folder: String,
    filename_format: String,
}

impl VaultWriter {
    pub fn new(config: &VaultConfig) -> Result<Self>;
    pub fn write(&self, note: &Note) -> Result<PathBuf>;
}
```

Implementation notes:

- Create `vault_path/folder/` if it doesn't exist
- Filename: `{filename_format}.md` rendered with `chrono::format`
- Write atomically: write to a `.tmp` file first, then `fs::rename` — avoids
  partial writes being picked up by Obsidian's file watcher
- Return the final `PathBuf` so the caller can log or display it

---

## crates/vv-daemon (main binary)

The orchestrator. Wires all crates together. `src/main.rs` should be thin —
real logic lives in `src/app.rs`.

### CLI interface (clap)

```
voicevault [OPTIONS]

Options:
  -c, --config <PATH>      Path to config file
      --trigger <MODE>     Override trigger mode from config
      --model <MODEL>      Override STT model from config
      --vault <PATH>       Override vault path from config
  -v, --verbose            Increase log verbosity (stackable)
  -h, --help
  -V, --version
```

### Startup sequence

1. Parse CLI args
2. Load config (CLI overrides take precedence)
3. Initialize `tracing_subscriber` — level from verbosity flags
4. Validate vault path exists and is writable; fail fast with a clear message
5. Initialize `WhisperEngine` (downloads model if needed; logs progress)
6. Initialize `AudioCapture`
7. Build trigger from config
8. Enter the main loop (see below)

### Main loop

```
loop {
    trigger.wait_for_start().await          // blocks until user initiates
    log "Recording started"
    spawn capture task → collects samples into Vec<f32>
    trigger.wait_for_stop().await           // blocks until user stops
    log "Recording stopped, transcribing..."
    write samples to temp WAV
    text = stt_engine.transcribe(wav_path).await
    note = Note { text, recorded_at, duration, trigger_kind }
    path = vault_writer.write(&note)
    log "Saved to {path}"
    delete temp WAV
}
```

Handle SIGINT/SIGTERM gracefully — finish any in-progress transcription before exit.
Use `tokio::signal` for this.

---

## Tests

### Unit tests (inline, per crate)

- `vv-core`: config parsing (valid, missing fields, bad vault path)
- `vv-core`: `Note::to_markdown` output format
- `vv-vault`: atomic write behavior; folder creation

### Integration tests (`tests/`)

- `test_push_to_talk_full_pipeline`: spawn daemon in PushToTalk mode, send stdin
  signals, assert a `.md` file appears in a temp vault dir with non-empty content.
  Use a pre-recorded WAV fixture for STT to keep the test deterministic and fast.
- `test_http_trigger`: same as above but drive via HTTP POST to `/start` and `/stop`.

Add a `tests/fixtures/` directory with a short (3s) English WAV file for use in
integration tests. Do not download models during CI — mock the STT engine with a
trait stub that returns a fixed string.

---

## README.md

Write a proper README covering:

- What this is
- Installation (cargo install from source)
- Dependencies: on Linux, `libasound2-dev` is required to build; user must be in
  the `input` group for GlobalHotkey mode
- Config file location and all options (mirror `config.example.toml` with explanations)
- All four trigger modes with usage examples
- macOS permissions note for GlobalHotkey (Accessibility API)
- Model size/speed tradeoff table

---

## Build and CI notes

- MSRV: 1.77 (matches cpal's minimum)
- Clippy must pass with `--deny warnings` on both macOS and Linux
- Use `cargo nextest` if available, else `cargo test`
- The `metal` feature should only be tested on macOS runners
- Do not check in model weights or audio files >1MB

---

## Implementation Order

Work in this order to stay unblocked:

1. Workspace scaffold + `vv-core` (config, errors, Note type)
2. `vv-vault` (no external deps beyond serde/chrono — fast to verify)
3. `vv-capture` (cpal — verify audio recording works end-to-end)
4. `vv-stt` (candle — most complex; get `tiny.en` working first)
5. `vv-trigger` (PushToTalk first — no system permissions needed)
6. `vv-daemon` (wire everything together with PushToTalk + tiny.en)
7. Remaining trigger modes (GlobalHotkey, HttpEndpoint, UnixSocket)
8. Tests, README, cleanup

Do not skip ahead to step 7 before step 6 is working end-to-end.

---

## Explicit Non-Goals for MVP

- No GUI or system tray
- No streaming transcription (batch only: record → file → transcribe)
- No note editing or browsing
- No Telegram bot integration (that's `tg_hyper_librarian`'s job via the UnixSocket trigger)
- No model fine-tuning
- No Windows support
