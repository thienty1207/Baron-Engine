# Baron 3.4.0 Safe Update Certification

Date: 2026-07-24
Release type: verified source certification

## Scope

Baron 3.4 makes `baron update` the human-authorized path for a verified Baron
runtime update and a conservative refresh of Baron-managed project assets.
It does not extend AI authority: `baron automation reconcile` is local-only,
uses currently installed embedded assets, and cannot download a release or
replace the Baron executable.

## Safety Contract

- `assets/core/` is the sole bundled runtime asset source.
- A managed baseline and frozen three-way transaction keep custom skills,
  custom agents, source files, plans, Harness records, and Vault Markdown out
  of the managed update write set.
- Ambiguous managed edits are staged as `BASE`, `LOCAL`, `UPSTREAM`, and `RESOLVED`
  packets; live project files and the active runtime remain
  unchanged until a verified continuation succeeds.
- Project activation, baseline replacement, and runtime handoff have an
  explicit transaction state and recovery/rollback behavior.
- A target-specific candidate is checked for release version, exact source
  revision, checksum, byte size, and executable identity before it can render
  new managed assets.

## Verification Evidence

| Evidence | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo test --workspace --all-targets --no-fail-fast` | passed; transaction, candidate, recovery, adapter, Vault, lifecycle, and public-doc suites ran from source `3.4.0` |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo build --release --locked -p baron-cli` | passed; `target/release/baron.exe --version` reported `baron 3.4.0` |
| `npx --yes yaml-lint .github/workflows/release.yml` | passed |
| `git diff --check` | passed; only normal Windows line-ending warnings were reported |
| Candidate and transaction fixtures | passed: verified local immutable candidate, staged plan, conflict no-write behavior, abort, stale-input refusal, checkpoint rollback, startup recovery, and Windows finalizer unit coverage |
| Installer lifecycle fixtures | passed: checksum verification, latest resolution, install, update, rollback, uninstall, and pre-3.4 compatibility paths |
| Release-binary smoke | passed: `baron survey . --json` and `baron init . --codex --shadow` completed with no target-repo writes |
| Project/Vault lifecycle smoke | passed through isolated temporary fixtures in the CLI and adapter suites, including custom skill preservation and local-only reconcile repair |

The source gate does not contact GitHub for a real public candidate. Network
release promotion remains a separate human-authorized operation after the
verified source commit is pushed.

## Public Boundary

- Normal users run `baron update` after the one-time installer upgrade from
  Baron 3.3 or older.
- AI agents never run public `baron update`.
- A Git tag or GitHub Release is intentionally outside this source
  certification and requires explicit human release promotion.
