from ..services.STT_Engine import STTEngine

engine: STTEngine = STTEngine()


def get_engine() -> STTEngine:
    return engine
