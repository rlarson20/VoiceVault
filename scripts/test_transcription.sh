#!/bin/bash
set -x

# Test with very short audio (1-2 seconds)
# This should complete in <2 minutes on CPU

echo "Testing transcription with jfk.wav..."
time cargo run --release -- --transcribe-only scripts/fixtures/jfk.wav

echo ""
echo "If this took >5 minutes, CPU inference is too slow."
echo "Consider using a smaller model or GPU acceleration."
