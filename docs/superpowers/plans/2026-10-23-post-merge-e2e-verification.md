# Post-Merge Status Reconciliation and End-to-End Verification Plan

> **For agentic workers:** Execute this plan inline. The user explicitly requested the documentation update, local checkout sync, and full verification in this task.

**Goal:** Record PR #4's verified merge state, align the maintained status documents, and exercise the merged Baron source from build/install through project initialization, logging, search, and continuity.

**Architecture:** Fast-forward the local `main` to the already-fetched `origin/main`, then make a documentation-only status update in the local checkout. Run the workspace checks and install the checked-out source into a task-local install root; all runtime exercises use a new scratch project, Vault, and machine home so the user's project and Vault data are not modified.

**Tech Stack:** Rust/Cargo, PowerShell 7, Baron CLI, GitHub Actions evidence, Codex and Claude native adapters.

**Spec:** User request in this task; repository constraints in `AGENTS.md` and `docs/architecture/COMMAND_SURFACE.md`.

## Global Constraints

- Core is the canonical product semantics and `.baron/core/**` owner; Codex and Claude are the only supported native adapters.
- Preserve user-owned project files, Vault data, machine configuration, and all existing untracked files.
- Do not create a new project identity in the user's real project or use the user's real Vault for fixtures.
- Do not commit, push, release, tag, publish, or rewrite history.
- Do not print secrets, tokens, passwords, or environment variable values in output or reports.
- Distinguish checked-out merged source verification from the separately published stable release artifact.

## Review Focus

- **Post-merge CI still running:** report merge-commit Windows results only after they finish; do not infer them from PR-head CI.
- **Published release may predate PR #4:** verify release metadata separately and do not claim the release binary contains the merge unless its source can be established.
- **Native agent CLIs may be absent or unauthenticated:** test generated Codex/Claude projections and hook contracts regardless; report live host execution only if an executable can safely run.
- **Other agent hosts are not native adapters:** test the host-neutral managed `AGENTS.md` fallback and the three mandatory quality-agent projections; do not label unsupported hosts as native integrations.
- **Search and continuity are project-scoped:** seed and query only the scratch project, and verify the resulting journal/state stays within the scratch Vault boundary.

---

### Task 1: Capture the merged baseline and current phase

**Files:**
- Read: `docs/BARON_STATUS.md`
- Read: `docs/BARON_STATUS.json`
- Read: `docs/superpowers/plans/CURRENT.md`
- Read: `notes/build-log/CURRENT.md`

- [x] Confirm PR #4 is merged and `origin/main` is at `cf319a12cfc19e2739303b1b26ee71357ae6c0cd`.
- [x] Fast-forward local `main` from `9038afa9f840e3a62acc0f97c6aaa31849247525` to `origin/main`; preserve existing untracked files.
- [x] Record test outcomes, proof/trace status, persisted-state boundary, and next safe action in the maintained documents.

### Task 2: Reconcile maintained status documents

**Files:**
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`

- [x] Replace stale PR-creation-blocked wording with PR #4 merged evidence and completed merge-CI results.
- [x] Record that the Control Plane false-positive implementation is merged and that this follow-up is end-to-end validation, not SPEC-06 or a release.
- [x] Record commands, host availability, test outcomes, proof/trace limitations, and safe next action after verification.
- [x] Parse status JSON and check repository-relative links and formatting.

### Task 3: Re-run Windows source checks on merged `main`

**Files:**
- Read: `.github/workflows/ci.yml`
- Build/test: current source at merge commit `cf319a1`

- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo test --workspace --all-targets --no-fail-fast -j 1` with the repository-supported PowerShell 7 override and isolated `BARON_HOME`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Record each exit code and test totals; no source-test failures occurred.

### Task 4: Install the merged source into an isolated test environment

**Files:**
- Create scratch-only install root, Vault, and project outside the repository.

- [x] Inspect the official installer behavior and latest published release metadata without changing a global installation.
- [x] Build/install the checked-out merge source into a task-local root; confirm `baron --version` and CLI help.
- [x] Create a fresh scratch Vault and a backend/Fiber test project under the scratch root.
- [x] Initialize Codex and Claude sequentially in that same scratch project; verify both owner projections and no conflicts.
- [x] Run the available Codex CLI read-only against the scratch project; Claude Code CLI was absent, so validate its generated projection and structured hook protocol directly.
- [x] Verify the generic `AGENTS.md` fallback and all three mandatory quality-agent projections; make no native-adapter claim for other hosts.

### Task 5: Exercise logs, search, and continuity

**Files:**
- Scratch project and scratch Vault only.

- [x] Use the installed CLI to create/read a Task State or continuity checkpoint and verify recovery/next-action data; exact binding resumed the interrupted plan.
- [x] Write scratch-project hook events and verify append-only journal entries and idempotent prompt retries.
- [x] Run Wiki search for a seeded query; verify the source project returns the hit and a different project does not.
- [x] Record the command families and observed outputs without exposing secrets.

### Task 6: Final verification and status update

**Files:**
- Modify: the four maintained status documents from Task 2.

- [x] Recheck `git status`, branch/HEAD/upstream equality, and preservation of pre-existing untracked paths.
- [x] Re-run JSON parsing, `git diff --check`, and documented link checks.
- [x] Update the four status documents with final evidence, limitations, and the next task; leave the workspace uncommitted and unpushed.

## Execution results — 2026-10-24

- Source checks on merge SHA `cf319a12cfc19e2739303b1b26ee71357ae6c0cd`: fmt passed; workspace tests passed with 1,029 passed, 0 failed, 4 ignored in 112 test binaries; warnings-denied Clippy passed. Post-merge GitHub Actions run 39982258262 passed Format/Clippy and native Linux/Windows tests.
- Release/source separation: the downloaded v5.0.0 installer passed in a disposable root and predates PR #4. `cargo install --path .\crates\baron-cli --root <scratch>\merged-source --locked` succeeded on the merged checkout; the source binary reports `baron 5.0.0`. Cargo printed a non-fatal warning that locked `chacha20 v0.10.1` is yanked.
- Scratch project checks: setup, Codex init, and Claude init all passed with no conflicts; the two adapters shared a Project ID and Vault. Go `go test ./...` passed and a live Fiber `/health` request returned status `ok`. The three mandatory agent projections and generic Baron `AGENTS.md` fallback were present.
- Control Plane regression: generated Codex contract status was `Passed: yes`; structured prepare returned `ok: true` with no `control_plane_invalid` blocker. Wiki indexed 12 docs/51 sections and exact search found the seeded marker; a second project with a distinct Project ID did not return the scratch document.
- Hooks/logging: Codex and Claude SessionStart and UserPromptSubmit calls passed; duplicate prompt retries returned identical output. The scratch Vault automation journal recorded both adapters plus the real ephemeral Codex CLI session. Automation reconciliation reported `attention_required` for absent proof, trusted reviewer receipts, and trace; this read-only smoke run intentionally created none. A stop event after deliberate interruption returned `block` and `completed: false`, the expected fail-closed result for a non-active plan.
- Continuity: checkpoint/status wrote to the scratch project and Vault. `plan start` with the exact adapter/session/request binding returned `Action: resumed`; the plan was then interrupted with an explicit next action. A separate new Codex session without a matching plan remained unknown. Session Replay reported 0 sources/0 messages for the ephemeral Codex session.
- Host caveats: the Claude Code executable is absent. The live Codex CLI completed, but its no-path `baron context --codex` diagnostic ran against `\\?\C:\Users\tytyb` and failed Windows path inspection inside that host shell; the same command with an explicit scratch project path, and from a shell rooted at a scratch project, passed. The scratch project is not a Git repo, so Codex's `git status` returned “not a git repository.” These observations do not invalidate the passing direct hooks, explicit-path context, or source test suites.
- Persisted-state boundary: runtime/install/project/Vault artifacts are in `%TEMP%\baron-e2e-20261023-93f4937f70a845c9affe210e05716c78`; only the four maintained status docs and this plan are changed in the repository. No source edits, commit, push, release, or publication.

## Superseding repository boundary — 2026-10-24

The no-commit/no-push boundary above described the original 2026-10-23
verification task. On 2026-10-24 the owner authorized committing and pushing
`codex/version-5-0-1` with the AI handoff and source-reading documentation
update. This later authorization does not authorize a PR, tag, or release;
current branch and release state are in [`CURRENT.md`](CURRENT.md).
