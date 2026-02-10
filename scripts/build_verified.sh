#!/usr/bin/env bash
set -euo pipefail

# Build percolator-prog in the verifiable-build Docker (linux/amd64)
# Solana 1.18.x / Cargo 1.75.0 for OtterSec compatibility

IMG="solanafoundation/solana-verifiable-build:1.18.9"
ARCH="linux/amd64"

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)

echo "Using image: $IMG"

docker run --rm \
  --platform "$ARCH" \
  -v "$ROOT_DIR:/work" \
  -w /work \
  "$IMG" \
  bash -lc 'set -e; solana --version; cargo-build-sbf --manifest-path Cargo.toml --sbf-out-dir target/deploy -- --locked'

OUT_DIR="$ROOT_DIR/target/deploy"
SO="$OUT_DIR/percolator_prog.so"

if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$SO" | awk '{print $1}' > "$SO.sha256"
else
  sha256sum "$SO" | awk '{print $1}' > "$SO.sha256"
fi

echo "Built artifact: $SO"
echo "SHA256: $(cat "$SO.sha256")"

