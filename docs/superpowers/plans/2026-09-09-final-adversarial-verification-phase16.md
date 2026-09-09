# Baron Codex + Claude Core Consolidation — Phase 16

**Status:** `completed — Baron Engine v5.0.0 released`

**Goal:** Independently verify the completed Phase 1–15 implementation against
the final Baron Core, Codex, and Claude release contract, and publish v5.0.0
only if every local and hosted release gate passes.

**Authority:** `docs/refractor/BARON_CODEX_CLAUDE_CORE_OPTIMIZATION_SPEC.md`
and the accepted Phase 16 brief in the task history. The current source,
generated assets, persisted-state fixtures, and observed command behavior are
the evidence.

## Scope boundary

- Re-audit source, generated payloads, persisted-state compatibility, docs,
  workflows, and release inputs independently of earlier phase reports.
- Run adversarial fixtures for safe I/O, path boundaries, identity, prepare,
  memory/context, routing, hooks, Autopilot, ownership, migration, rollback,
  and fresh Codex/Claude installation.
- Add tests or make a minimal behavior-neutral seam only when needed for
  deterministic evidence. Repair only a concrete release-blocking or high
  value correctness issue found by this audit.
- Review and classify every ignored test; no relevant broken behavior may be
  hidden by `#[ignore]`.
- Keep the current source buildable and preserve user-owned project, Vault,
  adapter, managed-state, continuity, journal, dedup, and Autopilot bytes.
- Do not begin a later phase. Version bump, commit, push, tag, and GitHub
  Release are authorized only after all pre-release gates and hosted CI pass.

## Initial audit evidence

- Branch is `main`; `origin/main` points to the same pre-release commit.
- The cumulative Phase 1–15 work is present in the working tree and no
  tracked files exist under `target/`.
- `cargo fmt --all -- --check` passes.
- The compatible system Windows PowerShell workspace sweep reports zero test
  failures. Three ignored Phase 1 fixtures fail only when explicitly run,
  confirming the historical behaviors were intentionally hardened in Phase 2.
  The ignored Phase 14 release smoke passes against the final 5.0.0 binary.
- The retired-adapter content and repository filename guards return no matches.
- The current source version is 5.0.0. Verified commit
  `cb2940e79fe4ed4cda7c52ea64c46c049b189514` is tagged `v5.0.0`; hosted run
  `34350086718` passed and published the immutable GitHub Release.

## Security remediation checkpoint (2026-09-09)

- SEC-04 is closed for security-sensitive gates: typed current-operation
  receipts carry exact project/task/operation/adapter/session/request/gate and
  source bindings; free-form summaries, cached capability evidence, child text,
  and persisted unkeyed receipts remain diagnostic.
- SEC-03 implementation tests pass for deterministic Ed25519 manifest signing,
  compiled-key verification, signature-first ordering, exact release identity,
  artifact checks, and mutation safety. Both bootstrap installers verify the
  detached signature before consuming security fields or mutating an existing
  installation. The release workflow consumes the protected
  `BARON_RELEASE_SIGNING_KEY` base64 raw-seed contract and checks its derived
  production identity without exposing the seed.
- Fresh native Codex Security scan
  `6312e662-b7ba-45da-b42a-d02fba00193f` completed on the final
  Windows/Linux snapshot with zero reportable findings. The focused scan
  covers installer path, Rust release/updater, release workflow, and
  receipt/gate authority surfaces; unrelated repository surfaces are recorded
  as deferred. The remote bootstrap-script self-authentication limitation is
  documented as an explicit trust boundary.
- The complete post-bump workspace test matrix, formatter, Clippy with warnings
  denied, release build, final-binary smoke, diff checks, and retired-adapter
  guards passed with zero unexpected failures. TAC is unavailable on this host
  and is reported separately. macOS is intentionally excluded from the
  release matrix. Hosted run `34350086718` passed signing and publication.

## Release-blocker repair checkpoint

- The first Windows junction adversarial run proved that initialization could
  write configuration through a linked `.baron` directory before a later
  adapter-path check rejected the project. The test was kept red until the
  fix was applied.
- `config`, `vault`, and durable Core state writers now use the shared safe I/O
  primitives for link rejection, preserve-first replacement, and explicit
  missing-file semantics. The linked `.baron` and linked control-plane tests
  pass on the current Windows host.
- No release metadata or persisted project/Vault data changed during the
  boundary repair. The source is now the verified 5.0.0 candidate.
- Hosted completion: the final diff was committed and pushed, every required
  hosted job passed, and the annotated tag and GitHub Release were published.

## Verification and release gates

1. Re-run focused adversarial and compatibility tests, adding deterministic
   coverage where the brief identifies a gap.
2. Review high-risk source paths and all ignored tests; record concrete
   findings and classify any limitation or blocker.
3. Run formatter, full workspace tests, Clippy, diff checks, retired-adapter
   guards, and current-binary smoke. Do not claim hosted-platform evidence
   locally.
4. Remove the generated `target/` directory, verify it is neither tracked nor
   staged, then build a clean release candidate and run the final v5.0.0 smoke
   matrix from the rebuilt binary.
5. If every gate is green, update product version/changelog/status metadata,
   commit the verified source, push the intended branch, wait for the actual
   GitHub matrix, then create and verify the annotated `v5.0.0` tag and GitHub
   Release. If any blocker remains, stop before version bump/tag/publication.

## Final release decision

The independent source/worktree audit, adversarial fixtures, security
remediation tests, complete workspace checks, clean-target rebuild, final
binary smoke, fresh native review, hosted CI, signed metadata verification,
annotated tag, and GitHub Release are complete. SEC-03 and SEC-04 are closed.
TAC is unavailable on this host and is reported separately. The next action is
normal Baron 5.0.0 maintenance.
