from fastapi import FastAPI, File, UploadFile
from fastapi.responses import FileResponse, JSONResponse
from whisper import load_model
import magic
from datetime import datetime

app = FastAPI(
    title="VoiceVault API",
    version="1.0.0",
    description="A simple API for transcribing voice notes.",
)


async def is_audio_file(file: UploadFile) -> bool:
    contents = await file.read(2048)
    await file.seek(0)

    mime_type = magic.from_buffer(contents, mime=True)
    return mime_type.startswith("audio/")


async def get_file_extension(file: UploadFile) -> str:
    contents = await file.read(2048)
    await file.seek(0)

    mime_type = magic.from_buffer(contents, mime=True)
    if mime_type == "audio/wav":
        return ".wav"
    elif mime_type == "audio/mpeg":
        return ".mp3"
    elif mime_type == "audio/mp4":
        return ".m4a"
    else:
        return ""  # Unsupported format


@app.post("/transcribe")
async def transcribe_voice_note(voice_file: UploadFile = File(...)):
    """
    POST /transcribe
    INPUT: audio file (wav, mp3, m4a, etc)
    PROCESS:
      - validate file format
      - generate ISO8601 timestamp
      - save temp audio file
      - call STT engine
      - generate MD content with metadata
    OUTPUT: json response with transcription text, duration, metadata
    """
    # step 1: validate file format
    if not await is_audio_file(voice_file):
        return JSONResponse(
            status_code=400,
            content={"message": "Invalid audio file format."},
        )
    if not (ext := await get_file_extension(voice_file)):
        return JSONResponse(
            status_code=400,
            content={"message": "Unsupported audio file format."},
        )
    # step 2: generate ISO8601 timestamp
    ts = datetime.now().isoformat()
    # step 3: save as file: /tmp/voice_note_timestamp.ext
    note_name = f"voice_note_{ts}{ext}"  # need to determine extension
    with open(f"/tmp/{note_name}", "wb") as f:
        contents = await voice_file.read()
        f.write(contents)
    # step 4: using STT engine, transcribe the audio file
    note = f"/tmp/{note_name}"
    if not note:
        return JSONResponse(
            status_code=500,
            content={"message": "Failed to save audio file."},
        )
    model = load_model("base", device="cpu")  # Load the Whisper model
    result = model.transcribe(note, language="en", fp16=False)
    text: str = result["text"]
    # step 5: generate markdown content with metadata about transcription
    # skipped for now, just return text
    # step 6: send JSON response with transcription text and metadata
    return JSONResponse(
        content={
            "message": "Note transcribed successfully",
            "received_at": ts,
            "text": text.strip(),
        }
    )


@app.get("/")
async def root():
    """
    GET /
    Return: HTML recording interface
    """
    return FileResponse("./templates/index.html")


@app.get("/health")
def health_check():
    """
    GET /health
    Return: JSON response with stt model status, disk space, recent file count
    """
