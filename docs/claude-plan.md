**STT Quick-Capture System Plan Summary:**

**Architecture:**

- FastAPI server on your Ubuntu server (i7-3770, 16GB RAM)
- Single `/transcribe` POST endpoint accepting audio file uploads
- Local STT model (non-Whisper, prioritizing accuracy over speed)
- Saves transcriptions as `voice-notes/YYYYMMDDTHHMMSS.md` in your Obsidian vault git repo

**Client Options:**

- **Web interface**: MediaRecorder API, auto-uploads on recording stop
- **CLI script**: Uses `arecord`/`ffmpeg`, POSTs via curl/requests
- Both approaches for flexibility

**File Organization:**

- Individual files per transcription (not daily aggregation)
- ISO8601 timestamps in filenames
- Simple markdown format with metadata footer
- Server writes directly to git repo directory that Obsidian watches

**Workflow:**

1. Trigger recording (web UI or CLI command)
2. Audio posted to server endpoint
3. STT processing generates transcription
4. File saved in `voice-notes/` folder
5. Manual git add/commit/push when ready to sync
6. Obsidian auto-refreshes to show new notes

**Success Target:**

- <10 seconds from thought to transcribed file
- Accuracy good enough to minimize editing
- Low enough friction that you actually use it consistently

**Key Technical Decisions:**

- Server-based processing (not laptop-local) for always-available capture
- Manual git workflow (no auto-commits)
- Research non-Whisper STT models for better accuracy on your hardware
- Individual files for easier organization and processing

The plan was designed as the foundation piece to enable your other workflow improvements, especially the quick thought capture that was eating 45% of your context-switching time.
