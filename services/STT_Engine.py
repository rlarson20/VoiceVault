from pathlib import Path
import whisper
import asyncio


class STTEngine:
    def __init__(self, model_name: str = "base"):
        self.model = whisper.load_model(model_name, device="cpu")
        self.model_id = model_name

    async def transcribe(self, path: str | Path):
        """Transcribe audio file to text"""
        audio_path_str = str(path)
        
        # Run the blocking transcription in a thread pool
        loop = asyncio.get_running_loop()
        result = await loop.run_in_executor(
            None, 
            lambda: self.model.transcribe(audio_path_str, fp16=False)
        )
        return result["text"]
