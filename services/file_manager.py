from pathlib import Path
from tempfile import NamedTemporaryFile
from fastapi import UploadFile
from datetime import datetime
import os


class FileManager:
    def __init__(self, storage_dir: str = "voice-notes"):
        self.storage_path = Path(storage_dir)
        self.storage_path.mkdir(exist_ok=True)

    async def save_temp_file(self, file: UploadFile) -> Path:
        """Saves uploaded file to temp location"""
        temp_file = NamedTemporaryFile(delete=False, suffix=Path(file.filename).suffix)
        contents = await file.read()
        with open(temp_file.name, "wb") as f:
            f.write(contents)
        return Path(temp_file.name)

    def generate_markdown(self, text: str, received_at: str) -> str:
        return f"""
        #Voice Note - {received_at}

        {text}

        ---
        *Transcribed on: {datetime.now().isoformat()}
        *Source timestamp: {received_at}
        """

    def save_note(self, content: str, timestamp: str) -> Path:
        """saves to final destination"""
        filename = f"{timestamp.replace(':', '-')}.md"
        filepath = self.storage_path / filename
        with open(filepath, "w") as f:
            f.write(content)
        return filepath

    def cleanup_file(self, path: Path):
        if path.exists():
            os.remove(path)
