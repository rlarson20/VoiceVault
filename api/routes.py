from fastapi.responses import FileResponse
from api.schemas import HealthCheckResponse, TranscriptionResponse
from api.dependencies import get_engine
from services.STT_Engine import STTEngine
from services.file_manager import FileManager
from fastapi import FastAPI, File, UploadFile, Depends, HTTPException
from datetime import datetime

app = FastAPI(
    title="VoiceVault API",
    version="1.0.0",
    description="A simple API for transcribing voice notes.",
)
file_manager = FileManager()


@app.post("/transcribe", response_model=TranscriptionResponse)
async def transcribe_voice_note(
    voice_file: UploadFile = File(...), stt_engine: STTEngine = Depends(get_engine)
):
    if not voice_file.content_type.startswith("audio/"):
        raise HTTPException(
            status_code=400, detail="Invalid file type. Please upload an audio file."
        )
    temp_path = None
    try:
        temp_path = await file_manager.save_temp_file(voice_file)
        transcription_text = await stt_engine.transcribe_voice_note(temp_path)
        ts = datetime.now().isoformat()
        md_content = file_manager.generate_markdown(transcription_text, ts)
        final_path = file_manager.save_note(md_content, ts)
        return TranscriptionResponse(
            message="Note transcribed successfully",
            received_at=ts,
            text=transcription_text,
            filename=final_path.name,
        )
    finally:
        if temp_path:
            file_manager.cleanup_file(temp_path)


@app.get("/")
async def root():
    """
    GET /
    Return: HTML recording interface
    """
    return FileResponse("./templates/index.html")


@app.get("/health", response_model=HealthCheckResponse)
async def health_check(stt_engine: STTEngine = Depends(get_engine)):
    return HealthCheckResponse(
        status="ok",
        model=stt_engine.model_id,
        device="cpu",  # TODO: change out STT engine to not depend on just whisper
    )


@app.get("/static/{filename}")
async def get_static(filename: str):
    return FileResponse(f"./static/{filename}")
