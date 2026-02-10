#!/usr/bin/env bash
set -euo pipefail

# Deploy a prebuilt verified binary and submit remote verification.
# This script DOES NOT run automatically — execute only with explicit approval.

if [[ $# -lt 3 ]]; then
  echo "Usage: $0 <program_so> <program_keypair.json> <upgrade_authority.json> [--cluster <url|moniker>]" >&2
  exit 2
fi

PROG_SO="$1"; shift
PROG_KP="$1"; shift
AUTH_KP="$1"; shift

CLUSTER="mainnet-beta"
if [[ ${1-} == "--cluster" ]]; then
  CLUSTER="$2"; shift 2
fi

export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version >/dev/null

echo "Cluster: $CLUSTER"
solana config set --url "$CLUSTER" >/dev/null

solana program deploy "$PROG_SO" --program-id "$PROG_KP" --upgrade-authority "$AUTH_KP"

PROG_ID=$(solana-keygen pubkey "$PROG_KP")
echo "Program ID: $PROG_ID"

ONCHAIN_SO="/tmp/${PROG_ID}.so"
solana program dump "$PROG_ID" "$ONCHAIN_SO"
HASH_ONCHAIN=$(solana-verify get-executable-hash "$ONCHAIN_SO")
echo "On-chain hash: $HASH_ONCHAIN"

echo "Submitting remote verification..."
echo "y" | solana-verify verify-from-repo --remote \
  --program-id "$PROG_ID" \
  https://github.com/purpletrade/percolator-prog \
  --library-name percolator_prog \
  --commit-hash $(git rev-parse HEAD)

echo "Check status: https://verify.osec.io/status/$PROG_ID"

