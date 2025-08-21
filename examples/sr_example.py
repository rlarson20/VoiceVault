import speech_recognition as sr
r = sr.Recognizer()
# Uses macOS built-in speech recognition
with sr.AudioFile("audio.wav") as source:
    audio = r.record(source)
text = r.recognize_sphinx(audio)  # or r.recognize_google() if online
