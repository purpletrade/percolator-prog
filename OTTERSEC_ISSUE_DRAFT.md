Title: Remote verification fails for percolator-prog — requires platform-tools v1.52 (Cargo/Rust 1.89) to reproduce

Summary
- We deployed `percolator-prog` to mainnet and can deterministically reproduce the on-chain ELF locally with Solana 1.18.26 + platform-tools v1.52 (rustc/cargo 1.89.0).
- OtterSec’s remote builder appears to use linux/amd64 + platform-tools v1.41 (Cargo 1.75.0) and/or ignores the requested `--tools-version v1.52`, resulting in build failure or a different binary.
- We’d like support for platform-tools v1.52 so the remote builder reproduces our hash and marks the program as verified.

Program
- Program ID: `GFzXiEhiRauw6k59L15zz4UJ9ZANaF5gpPtxEaYCo8jv`
- Repo: https://github.com/purpletrade/percolator-prog
- Commit: `6a206ee`
- Library name: `percolator_prog`

Hashes
- On-chain executable hash: `e5d99925ccd118865dfc64262edfd5531b76c4c8e894236a710e8aac300dbb4b`
- Locally reproduced (macOS arm64, platform-tools v1.52) hash: same as above

Local Reproduction
- Toolchain: Solana 1.18.26; platform-tools v1.52; rustc/cargo 1.89.0
- Command: `cargo build-sbf --tools-version v1.52 --sbf-out-dir target/deploy`

Remote Invocation
- Command used: `solana-verify verify-from-repo --remote --program-id GFzX... https://github.com/purpletrade/percolator-prog --library-name percolator_prog --commit-hash 6a206ee --base-image ghcr.io/solanafoundation/solana-verifiable-build:2.3.2 -- --tools-version v1.52`
- Result: remote builder reported failure / could not fetch requested base image, or built under older toolchain and did not match.

Request
- Add support for platform-tools v1.52 (or equivalent Rust/Cargo) in the remote builder.
- Respect the `--tools-version` (and `--base-image` when applicable) so builds can be reproduced deterministically for programs compiled with newer toolchains.

Context
- We’ve documented the reproducible build steps in our repo at `percolator-prog/REPRO_BUILD_NOTE.md`.
- We’re happy to test a new remote image or provide more logs if helpful.
