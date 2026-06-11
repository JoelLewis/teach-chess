#!/usr/bin/env bash
set -euo pipefail

# Downloads the Gemma 3 1B GGUF model and tokenizer from HuggingFace
# into src-tauri/models/ for bundling as a Tauri resource.

REPO_ID="unsloth/gemma-3-1b-it-GGUF"
GGUF_FILENAME="gemma-3-1b-it-Q4_K_M.gguf"
TOKENIZER_REPO_ID="unsloth/gemma-3-1b-it"
TOKENIZER_FILENAME="tokenizer.json"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$SCRIPT_DIR/../src-tauri/models"
mkdir -p "$MODELS_DIR"

GGUF_URL="https://huggingface.co/${REPO_ID}/resolve/main/${GGUF_FILENAME}"
TOKENIZER_URL="https://huggingface.co/${TOKENIZER_REPO_ID}/resolve/main/${TOKENIZER_FILENAME}"

download_file() {
    local url="$1"
    local dest="$2"
    local name="$3"

    if [ -f "$dest" ]; then
        echo "${name} already exists at ${dest}, skipping."
        return 0
    fi

    echo "Downloading ${name} (~770 MB for model, ~33 MB for tokenizer)..."
    echo "  URL: ${url}"
    curl -fSL --progress-bar "$url" -o "${dest}.tmp"
    mv "${dest}.tmp" "$dest"
    echo "${name} downloaded to ${dest}"
}

download_file "$GGUF_URL" "$MODELS_DIR/$GGUF_FILENAME" "GGUF model"
download_file "$TOKENIZER_URL" "$MODELS_DIR/$TOKENIZER_FILENAME" "Tokenizer"

echo "All model files ready in $MODELS_DIR"
