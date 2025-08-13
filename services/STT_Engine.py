import whisper
import asyncio


class STTEngine:
    def __init__(self, model_name: str = "base"):
        self.loop = asyncio.get_event_loop()
        self.model = whisper.load_model(model_name, device="cpu")

    async def transcribe(self, path: str):  # TODO: add Path supp
        return await self.loop.run_in_executor(
            None, lambda: self.model.transcribe(path, fp16=False)["text"]
        )

    def get_audio_duration(self):
        pass

    def preprocess_audio(self):
        pass

    def status(self):
        return {
            "status": "ok",
            "model": "test",
            "model_loaded": True,
            "disk_free": "23 GB",
            "uptime": 12345,
        }

    """
    - load_model(model_type="whisper_base")
    - transcribe(audio_file_path) -> text
    - get_audio_duration(file_path) -> seconds
    - preprocess_audio(file_path) -> normalized_file_path
    """
