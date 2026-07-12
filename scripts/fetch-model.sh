#!/usr/bin/env bash
set -euo pipefail

# Downloads the Gemma 4 E2B GGUF model from HuggingFace into
# src-tauri/models/ for bundling as a Tauri resource.
#
# llama.cpp embeds the tokenizer in the GGUF, so no separate
# tokenizer.json download is needed.

REPO_ID="unsloth/gemma-4-E2B-it-GGUF"
GGUF_FILENAME="gemma-4-E2B-it-Q4_K_M.gguf"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODELS_DIR="$SCRIPT_DIR/../src-tauri/models"
mkdir -p "$MODELS_DIR"

GGUF_URL="https://huggingface.co/${REPO_ID}/resolve/main/${GGUF_FILENAME}"

download_file() {
    local url="$1"
    local dest="$2"
    local name="$3"

    if [ -f "$dest" ]; then
        echo "${name} already exists at ${dest}, skipping."
        return 0
    fi

    echo "Downloading ${name} (~3.1 GB)..."
    echo "  URL: ${url}"
    curl -fSL --progress-bar "$url" -o "${dest}.tmp"
    mv "${dest}.tmp" "$dest"
    echo "${name} downloaded to ${dest}"
}

download_file "$GGUF_URL" "$MODELS_DIR/$GGUF_FILENAME" "GGUF model"

echo "All model files ready in $MODELS_DIR"
