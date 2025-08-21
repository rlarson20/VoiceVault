import whisper

model: "whisper.Whisper" = whisper.load_model("base", device="cpu")
# Record a 10-second voice memo on your phone, save as test.wav
result = model.transcribe(
    "Vocaroo 31 Jul 2025 16_12_05 EDT.mp3", language="en", fp16=False
)
print(result["text"])
