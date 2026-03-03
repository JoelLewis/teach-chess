#!/usr/bin/env bash
set -euo pipefail

# Downloads the appropriate Stockfish binary for the current platform
# into src-tauri/binaries/ with the Tauri sidecar naming convention.

STOCKFISH_VERSION="17"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARIES_DIR="$SCRIPT_DIR/../src-tauri/binaries"
mkdir -p "$BINARIES_DIR"
BINARIES_DIR="$(cd "$BINARIES_DIR" && pwd)"

detect_target() {
    local os arch target
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux)
            case "$arch" in
                x86_64)  target="x86_64-unknown-linux-gnu" ;;
                aarch64) target="aarch64-unknown-linux-gnu" ;;
                *)       echo "Unsupported Linux arch: $arch" >&2; exit 1 ;;
            esac
            ;;
        Darwin)
            case "$arch" in
                x86_64)  target="x86_64-apple-darwin" ;;
                arm64)   target="aarch64-apple-darwin" ;;
                *)       echo "Unsupported macOS arch: $arch" >&2; exit 1 ;;
            esac
            ;;
        MINGW*|MSYS*|CYGWIN*)
            target="x86_64-pc-windows-msvc"
            ;;
        *)
            echo "Unsupported OS: $os" >&2; exit 1
            ;;
    esac

    echo "$target"
}

download_stockfish() {
    local target="$1"
    local url filename ext=""

    case "$target" in
        *linux*)
            url="https://github.com/official-stockfish/Stockfish/releases/download/sf_${STOCKFISH_VERSION}/stockfish-ubuntu-x86-64-avx2.tar"
            filename="stockfish-ubuntu-x86-64-avx2.tar"
            ;;
        *darwin*)
            url="https://github.com/official-stockfish/Stockfish/releases/download/sf_${STOCKFISH_VERSION}/stockfish-macos-x86-64-avx2.tar"
            filename="stockfish-macos-x86-64-avx2.tar"
            ;;
        *windows*)
            url="https://github.com/official-stockfish/Stockfish/releases/download/sf_${STOCKFISH_VERSION}/stockfish-windows-x86-64-avx2.zip"
            filename="stockfish-windows-x86-64-avx2.zip"
            ext=".exe"
            ;;
    esac

    local sidecar_name="stockfish-${target}${ext}"

    if [ -f "$BINARIES_DIR/$sidecar_name" ]; then
        echo "Stockfish already exists at $BINARIES_DIR/$sidecar_name"
        return 0
    fi

    echo "Downloading Stockfish ${STOCKFISH_VERSION} for ${target}..."
    local tmpdir
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "${tmpdir:-}"' EXIT

    curl -fSL "$url" -o "$tmpdir/$filename"

    echo "Extracting..."
    if [[ "$filename" == *.tar ]]; then
        tar xf "$tmpdir/$filename" -C "$tmpdir"
        local sf_bin
        sf_bin="$(find "$tmpdir" -name 'stockfish*' -not -name '*.txt' -not -name '*.md' -not -name '*.cff' -type f | head -1)"
        cp "$sf_bin" "$BINARIES_DIR/$sidecar_name"
    elif [[ "$filename" == *.zip ]]; then
        unzip -q "$tmpdir/$filename" -d "$tmpdir"
        local sf_bin
        sf_bin="$(find "$tmpdir" -name 'stockfish*.exe' -type f | head -1)"
        cp "$sf_bin" "$BINARIES_DIR/$sidecar_name"
    fi

    chmod +x "$BINARIES_DIR/$sidecar_name"
    echo "Stockfish installed to $BINARIES_DIR/$sidecar_name"
}

main() {
    local target
    target="$(detect_target)"
    echo "Detected target: $target"
    download_stockfish "$target"
}

main "$@"
