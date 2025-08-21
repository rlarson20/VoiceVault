from pathlib import Path
import whisper
import asyncio


class STTEngine:
    def __init__(self, model_name: str = "base"):
        self.loop = asyncio.get_event_loop()
        self.model = whisper.load_model(model_name, device="cpu")
        self.model_id = model_name

    async def transcribe(self, path: str | Path):
        audio_path_str = str(path)
        return await self.loop.run_in_executor(
            None, lambda: self.model.transcribe(audio_path_str, fp16=False)["text"]
        )
