Vanity Program ID (PURPLE) — Key Grinding Guide

Goal
- Generate a program keypair whose base58 public key starts with `PURPLE`.

Command (CPU)
- macOS/Linux (use available cores):
  solana-keygen grind --starts-with PURPLE:1 \
    --ignore-case false \
    --num-threads $(getconf _NPROCESSORS_ONLN 2>/dev/null || sysctl -n hw.ncpu) \
    --out percolator-prog-vanity.json

Notes
- Finding a 6-character exact prefix can take time (minutes to hours) depending on hardware.
- If time is constrained, consider shorter prefixes (e.g., `PURP`, `PRPL`) or partial patterns (case-insensitive if acceptable).
- Securely back up the generated keypair. Restrict file permissions.

After Grinding
1) Update `declare_id!("<NEW_PROGRAM_ID>")` in `src/percolator.rs`.
2) Rebuild in Docker (linux/amd64) with the verified toolchain and pinned lockfile.
3) Deploy with `--program-id percolator-prog-vanity.json` and verify via OtterSec.
4) Set `--final` after the green badge appears.
