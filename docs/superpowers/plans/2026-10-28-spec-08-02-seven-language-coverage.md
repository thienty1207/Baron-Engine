# SPEC-08-02 Seven-Language Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Give Baron a bounded, evidence-backed language/framework inventory and local rule/fixture coverage for Python, C++, Java, Rust, Go, TypeScript, and C#.

**Architecture:** Add a read-only Core inventory that discovers language and framework evidence from repository paths and bounded manifest reads, records explicit scope exclusions and ambiguity, and projects planned coverage into PreparePacket. Keep the seven language rule packs and static fixture matrix in Baron-owned assets; fixtures are validated as data and are never executed by this change.

**Tech Stack:** Rust; existing `ignore`, `serde`, and `serde_json` dependencies; Markdown rule packs and fixtures.

**Spec:** `docs/specs/Spec8 - Replace VibeSecurity/SPEC-08-02-SEVEN-LANGUAGE-COVERAGE.md`

## Global Constraints

- Required languages are Python, C++, Java, Rust, Go, TypeScript, and C#.
- Mixed-language repositories must retain separate language evidence; one language cannot suppress another.
- Generated, vendored, fixture, and build-output paths are excluded only by an explicit recorded scope rule.
- C/C++ headers use build-manifest, compilation-database, and consuming-translation-unit evidence; ambiguity is reported without duplicating a physical path.
- Detection and rule selection are not audit execution or proof. PreparePacket remains `not_run`; execution stays blocked by SPEC-08-05 / SPEC-08-03.
- Fixtures are static source text. Do not execute fixture code or install optional scanners.
- Use manifest evidence for framework detection; do not guess a framework from the language alone.
- The framework catalog is partial; every in-scope detected language reports a `framework_detection_catalog_non_exhaustive_requires_validation` gap so an absent known signal is not treated as proof that no framework exists.
- Fixture expected outcomes are static contract labels only; SPEC-08-02 does not execute or behaviorally validate an audit classifier.
- Rule packs remain local/offline and use one shared severity/finding taxonomy.

## Review Focus

- A repository containing several languages must retain every language even when task paths mention only one; tests cover mixed monorepo discovery and PreparePacket projection.
- A generic `package.json` must not imply confirmed TypeScript source; tests distinguish manifest-only evidence and preserve its confidence gap.
- Shared `.h` files must not be silently double-counted or assigned an unsupported language interpretation; tests cover CMake/compile-command/consumer evidence and the ambiguous case.
- Excluded nested paths and symlinked directories must not disappear without a recorded reason; tests cover each explicit exclusion family and verify the scan is bounded.
- Framework signals must come from bounded readable manifests or exact parsed source imports; comments, unrelated package names, and project names must not create signals. The partial catalog gap remains visible, including when no known framework matches.
- Reads use Core's safe regular-file opener and enforce per-file and aggregate limits on the opened handle; tests cover oversized/unreadable manifests and no-match cases.

## Ruling

SPEC-08-02 lists SPEC-08-05 as a dependency, and SPEC-08-00 orders 08-05 before 08-02. The owner explicitly requested 08-02 next. Proceed with static discovery, local rule assets, and static fixture-contract tests only; do not execute target code, run an audit, or claim audit completion. SPEC-08-05 remains required before audit/remediation execution. Cost if this sequencing call is wrong: later 08-05 integration may require changes to the coverage plan contract.

The independent source review found that the known framework catalog could silently omit unlisted frameworks, path metadata checks were separated from reads, source import matching inspected comment text, and fixture tests did not state expected classifications. The catalog remains intentionally incomplete and each in-scope detected language now keeps an explicit validation gap; reads now use Core safe file opening and enforce limits on the opened handle; source signals are parsed from import targets after comment stripping; and fixture pairs state candidate/safe expected outcomes. The fixture contract validates these labels and shape only; actual classifier output remains unproven until SPEC-08-03. Cost if wrong: the catalog gap may be noisy, while static labels alone cannot guarantee model behavior.

## Tasks

### Task 1: Bounded repository language inventory

**Files:** Create `crates/baron-core/src/security_audit.rs`; modify `crates/baron-core/src/lib.rs` and `crates/baron-core/src/prepare.rs`; add module unit tests.

- [x] Write a failing CLI contract test for all seven languages in a mixed monorepo.
- [x] Run the focused test and observe the expected failure: only the task-path language was detected.
- [x] Add focused discovery tests for manifest/source confidence, explicit generated/vendor/fixture/build exclusions, C/C++ header interpretations, unreadable or oversized manifests, symlinks, and scan truncation before implementing those behaviors.
- [x] Implement deterministic, read-only discovery using bounded traversal and manifest reads; return evidence paths, framework signals, explicit exclusions, ambiguity, and gaps.
- [x] Re-run focused tests and review inventory evidence for path containment and output bounds. The Unix-only symlink regression is present but was not executed on this Windows host.

### Task 2: Shared taxonomy, language packs, and fixture matrix

**Files:** Create `assets/core/skills/security-audit/rules/shared/`, seven language rule packs under `rules/{python,cpp,java,rust,go,typescript,csharp}/`, a static fixture matrix under `fixtures/`, and `crates/baron-core/tests/security_audit_coverage.rs`.

- [x] Write failing asset-contract tests for the seven packs, eight shared attack classes, required rule fields, 7×7 applicable positive/safe fixture pairs, and C++ memory-safety plus Rust unsafe/FFI pairs.
- [x] Run the focused asset test and confirm it fails for the missing corpus.
- [x] Author the Baron-owned taxonomy/rules/fixtures. Include protected boundaries, safe patterns, common false positives, and evidence requirements; every positive and safe fixture records its bounded expected outcome.
- [x] Run the asset-contract test. Fixtures remain text-only and no target-controlled code is executed.

### Task 3: PreparePacket coverage plan and runtime entrypoint

**Files:** Modify `crates/baron-core/src/prepare.rs`, `crates/baron-cli/tests/prepare_cli.rs`, and `assets/core/skills/security-audit/SKILL.md`.

- [x] Write failing tests for mixed-language packet output, per-language evidence/rule-pack selection, explicit exclusions, framework signals, unsupported/ambiguous gaps, truncation blockers, and backwards-compatible deserialization.
- [x] Run the focused Core/CLI tests and confirm the expected missing packet fields or blockers.
- [x] Integrate repository discovery into the selected `security-audit` plan, keep rule selection lazy, and preserve `execution_status: not_run` with no fabricated check evidence.
- [x] Update the skill contract to require one coverage unit per in-scope language, retain gaps, and forbid audit claims from detection/fixture presence alone.
- [x] Run focused Core/CLI tests and the asset contract suite.

### Task 4: Status sync and final verification

**Files:** Update SPEC-08-02 status, `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, `docs/superpowers/plans/CURRENT.md`, and `notes/build-log/CURRENT.md`.

- [x] Record SPEC-08-02 scope, the SPEC-08-05 sequencing ruling, tests, and remaining execution blocker.
- [x] Run final JSON parsing, repository-relative Markdown link checks, and `git diff --check`; status JSON parsed, 23 Markdown files/60 relative links had 0 broken, and the diff check passed.
- [x] Review the branch diff and the available independent review; address all findings. No target fixture ran and no audit execution/clean result is claimed. The review was scoped, not a repository-wide line-by-line pass.
- [x] Mark final verification complete after reading the final checks and recording the remaining classifier-validation limitation for SPEC-08-03.
