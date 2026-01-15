#!/bin/bash
# scripts/download_models.sh
# Downloads required ONNX models for Amwaj Media Server

set -e

MODELS_DIR="${MODELS_DIR:-models}"

echo "📦 Creating models directory..."
mkdir -p "$MODELS_DIR"

echo "🔽 Downloading Silero VAD model (3.1 MB)..."
curl -L --progress-bar \
    https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx \
    -o "$MODELS_DIR/silero_vad.onnx"

echo "✅ Silero VAD model downloaded"

# Check if model is valid
if [ -f "$MODELS_DIR/silero_vad.onnx" ]; then
    SIZE=$(wc -c < "$MODELS_DIR/silero_vad.onnx")
    echo "   Size: $SIZE bytes"
fi

echo ""
echo "✅ All models ready in ./$MODELS_DIR/"
echo ""
echo "Models available:"
ls -la "$MODELS_DIR/"
