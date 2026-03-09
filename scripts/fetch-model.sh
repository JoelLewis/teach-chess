#!/usr/bin/env bash
set -euo pipefail

# Downloads the Gemma 2 2B GGUF model and tokenizer from HuggingFace
# into src-tauri/models/ for bundling as a Tauri resource.

REPO_ID="bartowski/gemma-2-2b-it-GGUF"
GGUF_FILENAME="gemma-2-2b-it-Q4_K_M.gguf"
TOKENIZER_FILENAME="tokenizer.json"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$SCRIPT_DIR/../src-tauri/models"
mkdir -p "$MODELS_DIR"

GGUF_URL="https://huggingface.co/${REPO_ID}/resolve/main/${GGUF_FILENAME}"
TOKENIZER_URL="https://huggingface.co/${REPO_ID}/resolve/main/${TOKENIZER_FILENAME}"

download_file() {
    local url="$1"
    local dest="$2"
    local name="$3"

    if [ -f "$dest" ]; then
        echo "${name} already exists at ${dest}, skipping."
        return 0
    fi

    echo "Downloading ${name} (~1.5 GB for model, small for tokenizer)..."
    echo "  URL: ${url}"
    curl -fSL --progress-bar "$url" -o "${dest}.tmp"
    mv "${dest}.tmp" "$dest"
    echo "${name} downloaded to ${dest}"
}

download_file "$GGUF_URL" "$MODELS_DIR/$GGUF_FILENAME" "GGUF model"
download_file "$TOKENIZER_URL" "$MODELS_DIR/$TOKENIZER_FILENAME" "Tokenizer"

echo "All model files ready in $MODELS_DIR"
