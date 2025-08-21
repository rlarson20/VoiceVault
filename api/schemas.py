from pydantic import BaseModel


class TranscriptionResponse(BaseModel):
    message: str
    received_at: str
    text: str
    filename: str


class HealthCheckResponse(BaseModel):
    status: str
    model: str
    device: str
