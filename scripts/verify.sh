#!/usr/bin/env bash
set -euo pipefail

# Deterministic verify helper for percolator-prog

usage() {
  cat <<'USAGE'
Usage: scripts/verify.sh [-c cluster] [-p program_id] [--arch sbfv1|sbfv2] [--no-docker]

Options:
  -c, --cluster   Cluster URL/moniker (mainnet-beta|devnet|testnet|localhost). Default: mainnet-beta
  -p, --program   Program ID to verify (default: parsed from src/percolator.rs declare_id!)
      --arch      SBF arch (sbfv1|sbfv2). Default: sbfv1
      --no-docker Build with local cargo-build-sbf instead of Docker
USAGE
}

sha256() { if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'; else sha256sum "$1" | awk '{print $1}'; fi; }

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
PROG_DIR="$ROOT_DIR"

CLUSTER="mainnet-beta"
PROGRAM_ID=""
USE_DOCKER=1
ARCH="sbfv1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -c|--cluster) CLUSTER="$2"; shift 2 ;;
    -p|--program) PROGRAM_ID="$2"; shift 2 ;;
    --no-docker) USE_DOCKER=0; shift ;;
    --arch) ARCH="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if [[ -z "$PROGRAM_ID" ]]; then
  PROGRAM_ID=$(sed -n 's/^declare_id!("\([A-Za-z0-9]*\)");/\1/p' "$PROG_DIR/src/percolator.rs" | head -n1)
fi
[[ -n "$PROGRAM_ID" ]] || { echo "Could not determine program id" >&2; exit 1; }

ONCHAIN_SO="$ROOT_DIR/onchain.so"
LOCAL_SO="$PROG_DIR/target/deploy/percolator_prog.so"

export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version >/dev/null
solana config set --url "$CLUSTER" >/dev/null
solana program dump "$PROGRAM_ID" "$ONCHAIN_SO"
ONCHAIN_SHA=$(sha256 "$ONCHAIN_SO")
echo "on-chain sha256: $ONCHAIN_SHA"

echo "Building local ELF (arch=$ARCH; docker=$USE_DOCKER)"
if [[ $USE_DOCKER -eq 1 ]]; then
  docker run --rm --platform linux/amd64 -v "$PROG_DIR:/work" -w /work \
    solanafoundation/solana-verifiable-build:1.18.9 \
    bash -lc "cargo-build-sbf --manifest-path Cargo.toml --sbf-out-dir target/deploy --arch $ARCH -- --locked"
else
  cargo-build-sbf --manifest-path "$PROG_DIR/Cargo.toml" --sbf-out-dir "$PROG_DIR/target/deploy" --arch "$ARCH" -- --locked
fi

[[ -f "$LOCAL_SO" ]] || { echo "Local build artifact not found: $LOCAL_SO" >&2; exit 1; }
LOCAL_SHA=$(sha256 "$LOCAL_SO")
echo "local    sha256: $LOCAL_SHA"

if [[ "$LOCAL_SHA" == "$ONCHAIN_SHA" ]]; then
  echo "MATCH: local build matches on-chain program"
else
  echo "MISMATCH: local build does NOT match on-chain program" >&2
  exit 3
fi

