import whisper

model: "whisper.Whisper" = whisper.load_model("small.en", device="cpu")
result = model.transcribe(audio="file", fp16=False)
