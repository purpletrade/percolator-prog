# Agent Context: percolator-prog

This file gives any AI agent (Claude Code, Cursor, Copilot, etc.) full context
to maintain this repository. Read this before making changes.

## What this repo is

A fork of [aeyakovenko/percolator-prog](https://github.com/aeyakovenko/percolator-prog)
(Anatoly Yakovenko's experimental Solana perpetual futures program). We maintain
a **verified build** overlay: security.txt, pinned dependencies, vendored crates,
and CI that produces a reproducible `.so` for Solana Explorer's green badge.

The core math lives in the `percolator` crate (a separate repo). This repo wraps
it in a single on-chain Solana program.

## Critical facts

| Item | Value |
|------|-------|
| **Program ID** | `GFzXiEhiRauw6k59L15zz4UJ9ZANaF5gpPtxEaYCo8jv` |
| **On-chain status** | **IMMUTABLE** (`Authority: none`). Cannot be upgraded. Future deploys require a **new program ID**. |
| **Upstream** | `https://github.com/aeyakovenko/percolator-prog` (branch: `main`) |
| **Upstream last synced** | Commit `d9bdfa2` (Feb 9, 2026) |
| **percolator math crate** | `https://github.com/purpletrade/percolator.git` rev `581dcaa` |
| **Solana version** | 1.18.x (CLI 1.18.9, docker image 1.18.9) |
| **Rust / Cargo** | 1.75.0 (installed via rustup in CI; not pre-installed in docker image) |
| **Lockfile format** | v3 (Cargo 1.75 compatible) |
| **Build target** | linux/amd64, SBF v1 |

## Repository layout

```
percolator-prog/
  src/percolator.rs          # The entire on-chain program (single file)
  Cargo.toml                 # Dependencies, [patch.crates-io] MSRV pins
  Cargo.lock                 # Pinned lockfile v3 (COMMITTED, never regenerate in CI)
  .cargo/config.toml         # Vendored source replacements
  vendor/                    # All dependencies vendored (~76MB, offline builds)
  .github/workflows/
    verified-build.yml       # Manual dispatch: builds .so in Docker, uploads artifacts
    upstream-sync.yml        # Scheduled: checks for new upstream commits
  scripts/
    build_verified.sh        # Local Docker build helper
    deploy_and_verify.sh     # Deploy + submit solana-verify (needs explicit approval)
    verify.sh                # Compare local build hash to on-chain hash
  tests/                     # Integration, unit, kani formal verification, fuzz
  SECURITY.md                # Security contact metadata
  WEBSITE_SECURITY_COPY.md   # Website security copy
  VERIFYING.md               # How to verify the build
  REPRO_BUILD_NOTE.md        # Reproducible build notes
  OTTERSEC_ISSUE_DRAFT.md    # OtterSec verification issue template
  VANITY_KEYGRIND.md         # Key grinding documentation
  audit.md                   # Kani formal verification audit (143 proofs)
```

## Build system

### Why it works this way

The `solanafoundation/solana-verifiable-build:1.18.9` Docker image has
`solana-cli` and `cargo-build-sbf` but **no `cargo` binary on PATH**.
`cargo-build-sbf` needs `cargo` to run `cargo metadata` before it can build.
The CI workflow installs Rust 1.75.0 via rustup into `/tmp/cargo/bin` to
provide the `cargo` binary.

### Pinned lockfile (do not regenerate in CI)

The committed `Cargo.lock` pins:
- `blake3 = 1.5.0` (avoids `constant_time_eq 0.4.x` which requires edition2024)
- `constant_time_eq = 0.3.1`
- `solana-program = 1.18.26`

If you regenerate the lockfile in CI, newer `blake3` (>=1.8.x) will pull in
`constant_time_eq 0.4.x` which uses `edition = "2024"` and breaks Cargo 1.75.

### [patch.crates-io] in Cargo.toml

These pins force MSRV-compatible versions even if someone regenerates the lock:
- `blake3` -> git tag 1.5.0
- `jobserver` -> git tag 0.1.31
- `borsh` / `borsh-derive` -> git tags borsh-v1.5.6 / borsh-derive-v1.5.6
- `indexmap` -> git tag 2.0.2
- `proc-macro-crate` -> git tag v3.0.1

### Vendoring

All deps (including git sources) are vendored in `vendor/`. The
`.cargo/config.toml` redirects all sources to `vendored-sources`. Every git
source in `[patch.crates-io]` and the `percolator` crate must have a
corresponding `[source."git+..."]` entry in `.cargo/config.toml`.

If you add a new git dependency or change a git tag/rev, you must:
1. `cargo vendor vendor --versioned-dirs=false`
2. Update `.cargo/config.toml` with any new source entries cargo vendor prints
3. Commit both `vendor/` and `.cargo/config.toml`

### Docker image quirks

- **No cargo on PATH**: Must install via rustup (see CI workflow)
- **Files created as root**: Use `sudo chown` before writing to Docker-created dirs
- **Platform tools v1.41**: Downloaded by `cargo-build-sbf --force-tools-install`

## CI workflows

### verified-build.yml (manual dispatch)

Builds the program in the Solana verifiable-build Docker image.

Trigger: Actions -> "Verified SBF Build (Solana 1.18.x)" -> Run workflow -> select branch

Produces artifacts: `percolator_prog.so` + `percolator_prog.so.sha256`

**The workflow definition must exist on `main`** for GitHub to show it in the
Actions UI. The `--ref` dropdown selects which branch's code gets built.
Always keep `main` and the build branch in sync for the workflow file.

### upstream-sync.yml (scheduled daily + manual)

Checks `aeyakovenko/percolator-prog` for new commits since our last sync.
Opens a GitHub issue if upstream has new changes. See "Upstream sync" below.

## Deployment

### Current state

The program at `GFzXiEhiRauw6k59L15zz4UJ9ZANaF5gpPtxEaYCo8jv` is **immutable**
(Authority: none). It was deployed in slot 399216144. It **cannot be upgraded**.

Any future deploy **must use a new program ID**. Generate a new keypair, update
`declare_id!()` in `src/percolator.rs`, rebuild, deploy, and verify.

### Deploy a new program

```bash
# 1. Generate new program keypair
solana-keygen grind --starts-with PRPL:1
# or use an existing keypair

# 2. Update declare_id! in src/percolator.rs
#    declare_id!("<NEW_PROGRAM_ID>");

# 3. Rebuild (CI or local), get the .so artifact

# 4. Deploy with upgrade authority (use --max-len for future headroom)
solana program deploy \
  --program-id /path/to/new-program-keypair.json \
  --upgrade-authority /path/to/upgrade-authority.json \
  --max-len 300000 \
  percolator_prog.so

# 5. Verify
echo "y" | solana-verify verify-from-repo --remote \
  --program-id <NEW_PROGRAM_ID> \
  https://github.com/purpletrade/percolator-prog \
  --library-name percolator_prog \
  --commit-hash $(git rev-parse HEAD)

# 6. Check status
# https://verify.osec.io/status/<NEW_PROGRAM_ID>
```

### Upgrade an existing (upgradeable) program

```bash
solana program deploy \
  --program-id <PROGRAM_ID_PUBKEY> \
  --upgrade-authority /path/to/upgrade-authority.json \
  percolator_prog.so

echo "y" | solana-verify verify-from-repo --remote \
  --program-id <PROGRAM_ID_PUBKEY> \
  https://github.com/purpletrade/percolator-prog \
  --library-name percolator_prog \
  --commit-hash $(git rev-parse HEAD)
```

## Upstream sync

### Upstream: aeyakovenko/percolator-prog

- Default branch: `main`
- No CI/workflows in upstream
- Uses `percolator = { path = "../percolator" }` (local sibling)
- We changed this to `percolator = { git = "...", rev = "581dcaa" }`
- Last synced: commit `d9bdfa2`

### How to sync

```bash
# 1. Add upstream remote (one-time)
git remote add upstream https://github.com/aeyakovenko/percolator-prog.git

# 2. Fetch upstream
git fetch upstream main

# 3. Review what changed
git log d9bdfa2..upstream/main --oneline
git diff d9bdfa2..upstream/main -- src/percolator.rs

# 4. Create sync branch
git checkout -b sync-upstream-$(date +%Y%m%d)

# 5. Cherry-pick or merge relevant commits
#    BE CAREFUL: upstream Cargo.toml uses path dep, we use git dep.
#    Do NOT overwrite our Cargo.toml, .cargo/config.toml, or vendor/.
#    Only take src/percolator.rs changes and test changes.
git cherry-pick <commit> --no-commit
# or manually apply changes to src/percolator.rs

# 6. Regenerate lockfile if deps changed
cargo +solana generate-lockfile

# 7. Re-vendor if deps changed
cargo vendor vendor --versioned-dirs=false
# Update .cargo/config.toml with any new source entries

# 8. Verify lockfile pins
grep -A1 'name = "blake3"' Cargo.lock          # must be 1.5.0
grep -A1 'name = "solana-program"' Cargo.lock   # must be 1.18.x

# 9. Commit and push
git add src/percolator.rs Cargo.toml Cargo.lock vendor .cargo/config.toml
git commit -m "sync: merge upstream percolator-prog to <new-rev>"
git push -u origin sync-upstream-$(date +%Y%m%d)

# 10. Run CI, download .so, deploy, verify
```

### What to watch for during sync

- **Cargo.toml changes**: Upstream may add/remove deps. Merge carefully — keep
  our `[patch.crates-io]`, git dep for percolator, and `solana-security-txt`.
- **New features / instructions**: Review for security implications.
- **Breaking changes to RiskEngine**: May require updating the percolator math
  crate rev too.
- **NEVER overwrite**: `Cargo.lock`, `.cargo/config.toml`, `vendor/`,
  `.github/`, `scripts/`, `SECURITY.md`, `CLAUDE.md`.

## Rules for agents

1. **Never commit unless explicitly asked.**
2. **Never push unless explicitly asked.**
3. **Never deploy or run on-chain transactions.**
4. **Never modify upgrade authority.**
5. **Never regenerate Cargo.lock in CI.** Use the committed lock with `--locked`.
6. **Keep `main` and build branch workflow files in sync.** GitHub reads the
   workflow definition from the default branch (`main`).
7. **After any dep change**: regenerate lockfile locally, re-vendor, update
   `.cargo/config.toml`, verify blake3 pin.
8. **Test locally before pushing CI changes** when possible.
9. **The program ID `GFzX...` is immutable.** Future deploys need a new ID.
10. **Follow existing code style.** The program is `#![no_std]`, single-file,
    uses `declare_id!` and `security_txt!`.
