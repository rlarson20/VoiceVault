from ..services.STT_Engine import STTEngine
from functools import lru_cache


@lru_cache(maxsize=1)
def get_engine() -> STTEngine:
    return STTEngine()
