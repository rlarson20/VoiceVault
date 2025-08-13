**VoiceFlow Application Pseudocode Framework:**

```
# ===== MAIN APPLICATION STRUCTURE =====

main.py:
  - FastAPI app initialization
  - Configure CORS for web client
  - Set up file upload limits and audio format validation
  - Initialize STT engine (Whisper/SpeechRecognition)
  - Define configuration (vault path, model settings)

# ===== CORE ENDPOINTS =====

POST /transcribe:
  INPUT: audio file (multipart upload)
  PROCESS:
    - Validate file format (wav, mp3, m4a, etc.)
    - Generate ISO8601 timestamp
    - Save temp audio file
    - Call STT engine
    - Generate markdown content with metadata
    - Save to voice-notes/{timestamp}.md
    - Clean up temp file
  OUTPUT: {transcription_text, filename, duration}

GET /:
  RETURN: HTML recording interface

GET /health:
  RETURN: STT model status, disk space, recent files count

# ===== STT ENGINE MODULE =====

stt_engine.py:
  class STTEngine:
    - load_model(model_type="whisper_base")
    - transcribe(audio_file_path) -> text
    - get_audio_duration(file_path) -> seconds
    - preprocess_audio(file_path) -> normalized_file_path

# ===== FILE MANAGEMENT MODULE =====

file_manager.py:
  class VoiceNoteManager:
    - generate_filename() -> YYYYMMDDTHHMMSS.md
    - create_markdown(text, metadata) -> formatted_content
    - save_note(filename, content) -> file_path
    - validate_vault_path() -> boolean
    - cleanup_temp_files() -> None

# ===== RECORDING CLIENT =====

templates/index.html:
  INTERFACE:
    - Record button (start/stop)
    - Audio level indicator
    - Status display (recording/processing/done)
    - Recent transcriptions list
  
  JAVASCRIPT:
    - navigator.mediaDevices.getUserMedia() for mic access
    - MediaRecorder API for audio capture
    - POST audio blob to /transcribe endpoint
    - Display results and handle errors

# ===== CONFIGURATION =====

config.py:
  - VAULT_PATH = "/path/to/obsidian/vault"
  - VOICE_NOTES_DIR = "voice-notes"
  - STT_MODEL = "whisper_base"
  - MAX_RECORDING_DURATION = 300 seconds
  - SUPPORTED_FORMATS = [".wav", ".mp3", ".m4a"]
  - TEMP_DIR = "/tmp/voiceflow"

# ===== ERROR HANDLING =====

exceptions.py:
  - STTProcessingError
  - AudioFormatError  
  - VaultAccessError
  - RecordingTooLongError

# ===== STARTUP SEQUENCE =====

if __name__ == "__main__":
  1. Validate configuration (vault path exists, STT model loads)
  2. Create necessary directories
  3. Test STT engine with sample audio
  4. Start FastAPI server
  5. Log startup status and available endpoints

# ===== DEPLOYMENT CONSIDERATIONS =====

docker/systemd service:
  - Environment variables for vault path
  - Volume mounts for Obsidian vault
  - Health checks via /health endpoint
  - Log rotation for transcription history
  - Automatic restart on failure

# ===== FUTURE EXTENSIONS =====

Optional modules for later:
  - CLI client (voiceflow record "quick note")
  - Batch processing for multiple files
  - Voice activity detection (trim silence)
  - Speaker identification
  - Integration with git auto-commit
  - Web dashboard for managing notes
```

**Key Design Decisions:**

- **Modular STT engine** - easy to swap Whisper for other models
- **Stateless operation** - each transcription is independent
- **Simple file-based storage** - no database needed
- **Web-first interface** - but architected for CLI addition later
- **Error resilience** - failed transcriptions don't crash server
- **Configuration-driven** - easy to adapt to different vault setups

This gives you the blueprint to build incrementally while keeping the architecture clean for future extensions.
