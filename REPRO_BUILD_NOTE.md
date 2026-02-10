Reproducible Build Note — Mainnet Program

Program
- Program ID: `GFzXiEhiRauw6k59L15zz4UJ9ZANaF5gpPtxEaYCo8jv`
- Library name: `percolator_prog`
- Source repo: https://github.com/purpletrade/percolator-prog
- Commit used to deploy: `6a206ee` (build: Switch percolator dep from path to git for verified builds)

On-chain Executable Hash
- `e5d99925ccd118865dfc64262edfd5531b76c4c8e894236a710e8aac300dbb4b`
- To confirm: `solana-verify get-executable-hash GFzXiEhiRauw6k59L15zz4UJ9ZANaF5gpPtxEaYCo8jv`

Deterministic Build Environment
- Solana CLI: `1.18.26`
- solana-cargo-build-sbf platform-tools: `v1.52`
- rustc: `1.89.0`
- cargo: `1.89.0`
- Host: macOS arm64

Reproduction Steps (local)
1) Install Solana CLI 1.18.26
2) From repo root (at commit `6a206ee`):
   - `cargo build-sbf --tools-version v1.52 --sbf-out-dir target/deploy`
3) Compare the hash:
   - `solana-verify get-executable-hash target/deploy/percolator_prog.so`
   - Expect: `e5d99925ccd118865dfc64262edfd5531b76c4c8e894236a710e8aac300dbb4b`

Notes
- This binary was built with platform-tools v1.52 (rustc 1.89.0). OtterSec’s current remote builder runs an older toolchain and cannot reproduce this hash today; the green badge will appear once their builder supports `v1.52` or equivalent.
