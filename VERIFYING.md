Percolator Program Verification

This document describes how to reproduce the exact program ELF and compare it to the on-chain binary for the deployed program ID.

Requirements
- Solana CLI v1.18.26 (includes `cargo-build-sbf`) or Docker
- Network access to fetch crates (or a vendored dependency set)
- The exact `Cargo.lock` used at deploy time

Quick Start (recommended)
- Run `scripts/verify.sh` from repo root. It will:
  - Dump the on-chain ELF for the program ID defined in `src/percolator.rs`
  - Build a local ELF using `cargo-build-sbf` and the current lockfile
  - Compare SHA-256 hashes and report MATCH/MISMATCH

Apple Silicon / Docker
- Use Docker with the official toolchain image: `solanalabs/solana:v1.18.26`
- The script automatically runs Docker with `--platform linux/amd64` for consistency.

Known Gotcha: constant_time_eq edition=2024
- Newer versions of `blake3` (>= 1.8.x) depend on `constant_time_eq 0.4.2`, which uses Rust edition 2024.
- Solana v1.18.26’s SBF toolchain uses `rustc 1.75`, which cannot compile edition 2024 crates.
- If your build fails with an error mentioning `feature 'edition2024' is required`, you likely have an updated `Cargo.lock`.

How to restore a compatible dependency set
1) Best: Use the exact `Cargo.lock` from the deployment commit (e.g., from your CI artifacts or git tag) and build with `--locked`.
2) If that lock is unavailable, temporarily pin `blake3` to `=1.5.0` (which depends on `constant_time_eq = 0.3.0`) to reconstruct the older toolchain-compatible graph:

   Add to `percolator-prog/Cargo.toml`:

   [patch.crates-io]
   blake3 = "=1.5.0"

   Then run the verify script again. Do not deploy with this change unless it matches the on-chain hash; this pin is only to reproduce the original build.

Choosing SBF arch
- If the original deployment targeted `sbfv2`, run the script with `--arch sbfv2`.
- If unknown, try both; the script will report a mismatch if incorrect.

Manual commands (without the script)
1) Dump on-chain program: `solana -u mainnet-beta program dump <PROGRAM_ID> onchain.so`
2) Build locally: `cargo-build-sbf --sbf-out-dir target/deploy -- --locked`
3) Compare: `shasum -a 256 onchain.so target/deploy/percolator_prog.so`

If the hashes match
- You can mark the program as verified in explorers that support it by providing the source, `Cargo.lock`, and toolchain version (Solana 1.18.26, SBF tools v1.41).

If the hashes do not match
- Ensure you used the correct `Cargo.lock`, SBF arch (`sbfv1` vs `sbfv2`), and no feature flags.
- If you cannot recover the original lockfile, exact verification may not be possible, and a redeploy with pinned dependencies may be the safer path.

