# Current Build Note

Date: 2026-07-27
Target: Baron 3.6.0
Program target: Baron 3.6.0

## Current Phase

Baron 3.6 final public release (`completed`).

## Final Release Checkpoint

- The user explicitly authorized publishing the immutable `v3.6.0` GitHub
  Release after asking for a reinstall-ready README.
- README now provides a one-block Windows installer, a strict `baron 3.6.0`
  version check, Vault restore guidance, and project refresh guidance.
- `docs/BARON_STATUS.md` has a visible final-release checklist. It is checked
  only after GitHub native builds, immutable tag/assets, and `releases/latest`
  smoke have completed.
- Fresh pre-publish verification passed: `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo build --release --locked -p baron-cli`, YAML lint for
  `.github/workflows/release.yml`, and a release-profile Vault/project
  certification smoke using `baron 3.6.0`.
- The initial immutable promotion run `30243740873` stopped in its Ubuntu
  verification job before any native build, tag, or GitHub Release was made.
  The error was a missing `CandidateBinaryInspector` trait import in an
  Unix-only runtime-activation branch. Windows local verification did not
  compile that branch, which is why the failure first appeared in GitHub.
- A RED/GREEN regression test now covers exact activated-runtime version
  matching. The repair moves that check into one shared helper so both the
  trait contract and the mismatch rejection are compiled on every platform.
- Fresh repair evidence: the exact-version regression, full workspace suite,
  Clippy, and `cargo build --release --locked -p baron-cli` passed. A local
  `x86_64-unknown-linux-gnu` cross-check is blocked only because this Windows
  machine lacks `x86_64-linux-gnu-gcc` for `ring` and bundled SQLite; it is not
  accepted as Linux proof. GitHub Ubuntu must compile the repaired exact source
  before Baron can create a tag or release.
- The repaired release binary completed a fresh `setup --vault`,
  `init --codex --fullstack`, and `certify run --profile release` smoke. It
  produced the expected Vault, Codex adapter, project configuration, and all
  release certification checks passed.
- The second immutable promotion run `30244634765` also stopped before any
  tag, native artifact, or GitHub Release. Its Linux-only self-update fixture
  copied a debug binary larger than the real 128 MB candidate ceiling, so the
  fixture failed before its intended version-mismatch assertion. The fix uses
  a tiny executable Unix wrapper that delegates to the running test binary;
  production candidate limits remain enforced. The fresh full suite, Clippy,
  locked release build, and workflow YAML check passed after this fixture fix.
- An incorrect-SHA dispatch `30245161318` was cancelled before any build or
  promotion after its mismatch was detected. The next dispatch always derives
  `source_revision` directly from `origin/main`, not from copied text.
- The third immutable promotion run `30245233369` also stopped before any tag,
  native artifact, or GitHub Release. The remaining Unix update-transaction
  fixture staged an oversized patched debug binary. It now stages a bounded
  wrapper that delegates to the patched backing binary, preserving the expected
  candidate protocol and the real production size ceiling. The complete local
  test suite, Clippy, locked release build, and workflow YAML gate passed after
  the complete fixture audit.
- The fourth immutable promotion run `30245849879` passed its complete Ubuntu
  test suite, then stopped before tag creation during Clippy. The error set was
  limited to an empty Unix-only placeholder test plus two Windows-only runtime
  owners compiled on Linux. The placeholder test is removed and the runtime
  state path/helper are now explicitly Windows-scoped; the release workflow and
  every production safety ceiling remain unchanged.
- Fresh fourth-repair local evidence passed: `cargo fmt --all -- --check`,
  `cargo test --workspace --all-targets --no-fail-fast --quiet`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo build --release --locked -p baron-cli`, workflow YAML lint, status JSON
  parsing, and a release binary `setup --vault`, `init --codex --fullstack`,
  `certify run --profile release`, and `--version` smoke. The smoke returned
  `baron 3.6.0` and passed every release certification check.
- Repaired source commit `36413199d39c547664d1a2500e8a4444219d858e` is now
  pushed to `origin/main`. The next immutable promotion derives its exact source
  SHA from the current remote branch instead of using copied text.
- Final public proof: GitHub Actions run `30246729740` passed exact-source
  verification, Windows, Linux, macOS Intel, macOS Apple Silicon, and immutable
  promotion. Tag `v3.6.0` resolves to
  `c89486694d9a4431e04106274d0c9f997db42683`; the release contains native
  archives, `SHA256SUMS`, `release-manifest.json`, `install.ps1`, and
  `install.sh`.
- Public Windows smoke: downloaded `install.ps1` from `releases/latest` into a
  new temporary directory, installed with `-Version 3.6.0 -NoPathUpdate`, then
  verified `baron 3.6.0`, `setup --vault`, `init --codex --fullstack`, and
  `context` in a fresh project.
- No release action remains. For a Windows reinstall, follow the short README
  flow: restore the Vault and project folders, install from `releases/latest`,
  run `baron setup --vault`, then run `baron update` in each project.

## Verified Checkpoint Before Phase 35

The bundled Superpowers workflow core has been refreshed to the pinned upstream
`v6.2.0` skill tree without changing Baron's core ownership:

- exact upstream commit:
  `3dcbd5c4b48e02263fbf4a3c01e3fe4f81d584d9`
- upstream subtree: 50 files with no missing or extra file; one recorded Baron
  patch removes the visual companion's remote branding/telemetry request
- Baron-owned root routing and adapter integration remain local
- Codex, Claude, and generic adapter contract test passed
- provenance, MIT license, and offline runtime ownership are recorded
- Unix installs preserve executable mode for embedded shebang scripts
- a short Baron-owned entry contract protects the SDD review breaker and
  behavior-test rules from long-context summarization drift
- extension boundaries for code graphs and optional skill sources are accepted
  in `docs/decisions/0002-extension-ownership-and-code-graph.md`

This checkpoint starts Phase 35 only. It does not bump the Baron version, add
Graphify, or change the 3.4 phase order.

## Blueprint Audit

The two apparent core trees are not equal maintained sources:

- `assets/core/` has 145 runtime asset files and is embedded by
  `baron-adapters`.
- `blueprints/core/` has 7 stale historical files.
- Six paths overlap and all six have different content.
- No Rust crate, installer, manifest, test, or workflow reads
  `blueprints/core/`.

Phase 35 completed the source-of-truth contract, deletion of
`blueprints/core/`, adapter parity proof, managed baseline, conservative
three-way planner, malformed-marker refusal, and no-write CLI preview. The
RED test failed on the stale directory; the same focused test passed after
removal. A fresh install now records only repository-relative Baron-owned
content under `.baron/managed-state/`; custom assets and project state remain
outside its write set.

## What Is Being Built

- Phase 35 has removed stale blueprints, proved `assets/core/` is the only
  runtime source, and recorded the managed baseline plus safe read-only
  three-way preview.
- Phase 36 resolved and verified the exact native release candidate before project activation.
- Phase 37 applies project/runtime changes transactionally, stages conflicts, and recovers interrupted updates; its focused transaction/recovery suites now pass.
- Phase 38 separates human update authority from AI local repair and certifies Baron 3.4; the full certification gate has passed and `3.4.0` is the current stable source baseline.
- Phases 39-41 produced certified Baron 3.5 by distilling selected frontend,
  deep-module, and domain-language techniques into existing Baron owners, with
  no second workflow or frontend skill. Reconcile now restores missing managed
  Domain Language documents without overwriting user content.
- Phases 42-45 produced Baron 3.6 with an optional project-scoped Graphify
  code-only provider, bounded context, source verification, strict isolation,
  and Survey fallback. The source baseline is now `3.6.0`.

## Resume Point

1. Read `docs/BARON_STATUS.md`.
2. Read `docs/superpowers/plans/2026-07-24-baron-3-4-to-3-6-program.md`.
3. Read
   `docs/superpowers/specs/2026-07-24-baron-3-4-to-3-6-controlled-extension-design.md`.
4. Execute
   `docs/superpowers/plans/2026-07-23-phase-35-38-baron-3-4-safe-update.md`.
5. Phase 39 completed: local frontend guidance now has a brief fingerprint,
   anti-template gates, responsive/state proof, three-adapter installation
   proof, narrow routing proof, and no operational live dependency.
6. Phase 40 completed: local deep-module guidance and project-scoped domain
   language now preserve user content, avoid generic memory leakage, and
   withhold divergent repo/Vault terms from trusted context.
7. Phase 41 completed: local assets, narrow routing, adapter parity, user
   preservation, public flow, and the Baron 3.5 source gate passed.
8. Phase 42 completed: provider-neutral graph state, source fingerprint,
   project identity, cache path, junction, query-bound, and optional-capability
   contracts passed without provider invocation.
9. Phase 43 completed: the deterministic fake Graphify adapter accepts only
   `0.9.25`, permits only local probe/code-only extraction/bounded query calls,
   disables provider query logging, uses bounded subprocess files and atomic
   cache promotion, and keeps the last good graph on every tested failure.
10. Phase 44 completed: task-matched context never invokes Graphify, caches at
   most eight local hints, and routes AI to hidden refresh/query only when
   needed. Every graph result is checked against current repo source; inferred
   results remain advisory.
11. Phase 45 completed: same-name graph isolation, Vault exclusion, stale and
   corrupt fallback, 6,100+ source fingerprint coverage, and hook/instruction
   preservation passed.
12. Baron 3.6 source/docs/lock/certification are synchronized. Read
   `docs/assessment/baron-3.6.0-code-graph-certification.md` before any later
   code-map change.
13. `v3.6.0` is publicly released after immutable GitHub proof. No active
   implementation or release action remains.

## Verified Baseline

- Isolated branch: `codex/baron-3-4-to-3-6`.
- Planning baseline commit: `b6e619b`.
- Baseline `cargo test --workspace --all-targets`: passed on 2026-07-23.
- Source and stable-source version are now `3.6.0` after Phase 45's full gate.
- Baron 3.4 and 3.5 have source certification; no GitHub Release/tag was made.
- Baron 3.6 Phases 42-45 have complete source/test evidence.
- Baron `v3.6.0` is the latest downloadable release, built and promoted by
  GitHub Actions run `30246729740`.
- The program has 0 planned phases remaining.
- Superpowers `v6.2.0` adapter contract: passed on 2026-07-24.
- Full post-refresh workspace tests, Clippy, release build, adapter smoke, SDD
  semantic smoke, and visual-server behavior tests: passed on 2026-07-24.

## Non-Negotiables

- Superpowers remains workflow core.
- Core agents remain `code-reviewer`, `security-auditor`, and `test-engineer`.
- Vault Markdown remains source of truth.
- Normal users keep the simple install/setup/init/update flow.
- AI never silently downloads or activates a release.
- Conflicting managed edits never overwrite live project content.
- No release tag or GitHub Release exists before its source and artifacts pass proof.
- `assets/core/` is the only runtime source for bundled skills and agents.
- `blueprints/core/` must be removed before any managed baseline is trusted.
- Optional providers and skills cannot become a second workflow, memory, or
  instruction owner.
- Baron 3.5 may deepen existing owners but must not create Hallmark or Matt
  workflow skills.
- Baron 3.6 may use Graphify only as a local optional code map; it cannot own
  hooks, instructions, Vault memory, global context, or workflow state.
