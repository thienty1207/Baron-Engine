# Baron 3.5 Skill Intelligence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Strengthen Baron's existing frontend, interface, architecture, and Product Harness guidance with selected Hallmark and Matt Pocock techniques without adding another workflow core or duplicate bundled skill.

**Architecture:** Rewrite a small, pinned subset of external ideas into Baron-owned local references under existing skills. Add one Product Harness domain-language document with bounded context loading. Preserve current routing names, Superpowers ownership, three core quality gates, and the simple user command surface.

**Tech Stack:** Rust 2021, Markdown runtime assets, existing `include_dir` adapter installer, Baron Product Harness and Context Compiler, Serde tests, controlled skill pressure tests.

**Prerequisite:** Baron `3.4.0` Phase 38 certification is complete and committed.

---

### Task 1: Phase 39 Hallmark Distillation Into `frontend-design`

**Files:**
- Modify: `assets/core/skills/frontend-design/SKILL.md`
- Modify: `assets/core/skills/frontend-design/NOTICE.md`
- Create: `assets/core/skills/frontend-design/references/brief-fingerprint.md`
- Create: `assets/core/skills/frontend-design/references/anti-template-gates.md`
- Create: `assets/core/skills/frontend-design/references/responsive-state-proof.md`
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Modify: `crates/baron-core/tests/control_plane.rs`
- Create: `docs/assessment/baron-3.5.0-skill-intelligence-certification.md`

- [ ] **Step 1: Add RED adapter assertions for one frontend owner**

Add a test named
`frontend_design_is_local_deep_and_has_one_routing_owner` that installs Codex,
Claude, and generic adapters and asserts:

```rust
for root in [
    repo.join(".codex/skills"),
    repo.join(".claude/skills"),
    repo.join(".baron/core/skills"),
] {
    assert!(root.join("frontend-design/SKILL.md").is_file());
    assert!(root.join("frontend-design/references/brief-fingerprint.md").is_file());
    assert!(root.join("frontend-design/references/anti-template-gates.md").is_file());
    assert!(root.join("frontend-design/references/responsive-state-proof.md").is_file());
    assert!(!root.join("hallmark").exists());
    assert!(!root.join("matt-skills").exists());
}
```

Also assert the installed operational files contain no `npx skills add`, live
raw-GitHub dependency, Hallmark command router, or second workflow declaration.

- [ ] **Step 2: Run the focused adapter test and confirm RED**

Run:

```powershell
cargo test -p baron-adapters --test adapter_lifecycle frontend_design_is_local_deep_and_has_one_routing_owner -- --exact
```

Expected: FAIL because the three Baron references do not exist yet.

- [ ] **Step 3: Write the Baron-owned frontend references**

Write `brief-fingerprint.md` as a decision sequence:

1. identify product job, audience, content, brand evidence, and interaction
   pressure
2. choose one macrostructure justified by that evidence
3. state three product-specific visual signals
4. reject three generic defaults for this task
5. preserve existing information architecture unless the user approved a
   redesign

Write `anti-template-gates.md` as a bounded changed-surface audit covering:

- interchangeable SaaS card grids
- decorative gradients, floating blobs, and generic dark dashboards
- oversized marketing typography inside tools
- repeated rounded containers with no information hierarchy
- arbitrary iconography, stock imagery, and invented product copy
- theme swaps that keep the same composition
- inaccessible contrast, hidden focus, broken reduced motion, and mobile
  overflow

Write `responsive-state-proof.md` with an evidence matrix for narrow/wide
viewports, long content, loading, empty, error, disabled, keyboard/focus, and
reduced motion. Every row must be `observed`, `not applicable with reason`, or
`not verified`; unknown coverage cannot be reported as passing.

- [ ] **Step 4: Integrate references into the existing skill**

Update `frontend-design/SKILL.md` so it:

- loads only the reference needed for the current UI task
- distinguishes `audit`, `refine`, and `redesign`
- requires a brief fingerprint before new visual composition
- requires the anti-template gate before final response
- requires responsive/state evidence for meaningful frontend work
- keeps Superpowers and Baron proof/trace ownership unchanged

Do not add a `hallmark` folder or Hallmark command surface.

- [ ] **Step 5: Pin attribution without creating a runtime dependency**

Update `NOTICE.md` to record:

```text
Research source: nutlope/hallmark
Audited revision: aeb42fb354ff4efa36ab475773a082315a3af2ce
License observed at audit: MIT
Baron integration: rewritten local principles; no Hallmark installer or runtime dependency
```

- [ ] **Step 6: Prove routing stays narrow**

Extend `control_plane.rs` tests:

- frontend page/layout/responsive tasks route `frontend-design`
- backend/API-only tasks do not route it
- the route still includes the three Baron quality gates
- no route returns `hallmark`

Run:

```powershell
cargo test -p baron-core --test control_plane
cargo test -p baron-adapters --test adapter_lifecycle frontend_design_is_local_deep_and_has_one_routing_owner -- --exact
```

Expected: PASS.

- [ ] **Step 7: Run controlled frontend skill pressure tests**

Use the installed Baron skill against these fixed fixtures and record findings
in the 3.5 certification document:

- operational dashboard with long Vietnamese labels and dense repeated actions
- mobile checkout with loading, error, disabled, and payment states
- brand landing page with a generic purple-card baseline

For each fixture, compare the current 3.4 skill response with the 3.5 candidate.
The candidate passes only if it produces a product-specific structure, catches
the seeded generic pattern, names missing state/viewport proof, and does not
invent brand facts.

- [ ] **Step 8: Commit Phase 39**

Run `git diff --check`, update status/build log with actual evidence, then:

```powershell
git add assets/core/skills/frontend-design crates/baron-adapters/tests/adapter_lifecycle.rs crates/baron-core/tests/control_plane.rs docs/assessment/baron-3.5.0-skill-intelligence-certification.md docs/BARON_STATUS.md docs/BARON_STATUS.json notes/build-log
git commit -m "feat: deepen Baron frontend design intelligence"
```

### Task 2: Phase 40 Deep Modules And Product Domain Language

**Files:**
- Modify: `assets/core/skills/api-and-interface-design/SKILL.md`
- Modify: `assets/core/skills/api-and-interface-design/NOTICE.md`
- Create: `assets/core/skills/api-and-interface-design/references/deep-module-boundaries.md`
- Create: `crates/baron-core/src/domain_language.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-core/src/harness.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Create: `crates/baron-core/tests/domain_language.rs`
- Modify: `crates/baron-core/tests/context_compiler.rs`

- [ ] **Step 1: Add RED deep-module asset assertions**

Extend the three-adapter test to require:

```text
api-and-interface-design/references/deep-module-boundaries.md
```

Assert that no installed skill is named `codebase-design`,
`domain-modeling`, `tdd`, `implement`, `grilling`, or `matt-skills`.

- [ ] **Step 2: Add RED Product Harness domain-language tests**

Define the public core API expected by the tests:

```rust
pub struct DomainLanguageStatus {
    pub path: PathBuf,
    pub term_count: usize,
    pub ambiguous_count: usize,
}

pub fn ensure_domain_language(
    repo_root: impl AsRef<Path>,
    vault: &VaultContext,
) -> Result<DomainLanguageStatus>;

pub fn render_domain_language_context(
    repo_root: impl AsRef<Path>,
    max_chars: usize,
) -> Result<String>;
```

Tests must prove:

- first ensure creates `docs/baron/harness/DOMAIN_LANGUAGE.md` and the Vault
  mirror without inventing any project term
- repeated ensure preserves user-written terms byte-for-byte
- a bounded render includes canonical term, meaning, evidence path, and status
- ambiguous/unknown terms stay labeled and cannot appear as verified facts
- output never exceeds the supplied character budget
- two projects sharing a Vault never share domain-language files

- [ ] **Step 3: Run RED tests**

Run:

```powershell
cargo test -p baron-core --test domain_language
```

Expected: FAIL because `domain_language` does not exist.

- [ ] **Step 4: Implement the project-owned domain-language template**

Create this file only when missing:

```markdown
# Product Domain Language

## Rules

- Add terms only from user, repository, product, or verified runtime evidence.
- Mark disputed or unclear meanings as `ambiguous`.
- Do not promote a term to verified without an evidence path.

## Terms

| Term | Meaning | Status | Evidence |
| --- | --- | --- | --- |
```

Mirror the same project-owned content to
`Projects/<project-id>/ProductHarness/DOMAIN_LANGUAGE.md`. Use the existing
Vault project identity and atomic write helpers. Never merge terms across
projects.

- [ ] **Step 5: Load a bounded domain-language excerpt**

Call `ensure_domain_language` from Product Harness initialization and render a
bounded `## Product Domain Language` section in compact context when terms
exist. Add the full file to skipped-context diagnostics rather than dumping it.

Generated agent instructions must say:

- use established canonical terms when evidence exists
- record a new term only when it changes cross-module understanding
- mark disagreement as ambiguous instead of silently picking a definition
- Product Harness owns this document; Superpowers still owns workflow

- [ ] **Step 6: Write deep-module guidance**

Create `deep-module-boundaries.md` covering:

- keep public surface smaller than hidden implementation complexity
- place invariants at the owning boundary
- avoid pass-through modules that only rename another API
- define one entry point and explicit dependency direction
- design seams that can be tested through public behavior
- preserve compatibility and migration evidence for existing callers
- reject a new abstraction when it does not hide meaningful complexity

Link it from `api-and-interface-design/SKILL.md` only for module/API boundary
tasks. Do not route it for routine implementation.

- [ ] **Step 7: Pin Matt Pocock attribution and rejection boundary**

Update `NOTICE.md` to record:

```text
Research source: mattpocock/skills
Audited revision: ed37663cc5fbef691ddfecd080dff42f7e7e350d
Accepted technique families: codebase-design and domain-modeling
Rejected runtime imports: workflow, TDD, debugging, review, planning, grilling, implementation, handoff, setup, in-progress, deprecated, and personal skills
License observed at audit: MIT
```

- [ ] **Step 8: Run focused verification**

Run:

```powershell
cargo test -p baron-core --test domain_language
cargo test -p baron-core --test context_compiler
cargo test -p baron-adapters --test adapter_lifecycle
```

Expected: PASS with no duplicate skill owner.

- [ ] **Step 9: Commit Phase 40**

Update status/build log with fresh evidence, then:

```powershell
git add assets/core/skills/api-and-interface-design crates/baron-core crates/baron-adapters docs/BARON_STATUS.md docs/BARON_STATUS.json notes/build-log
git commit -m "feat: add deep module and domain language guidance"
```

### Task 3: Phase 41 Routing, Preservation, And Baron 3.5 Certification

**Files:**
- Modify: `crates/baron-core/tests/control_plane.rs`
- Modify: `crates/baron-core/tests/public_trust_docs.rs`
- Modify: `crates/baron-cli/tests/adapter_cli.rs`
- Modify: `crates/baron-cli/tests/release_smoke.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Modify: `docs/assessment/baron-3.5.0-skill-intelligence-certification.md`

- [ ] **Step 1: Add RED preservation and no-duplication tests**

Tests must prove:

- all three adapters install the same frontend/interface references
- custom skills, agents, and custom index sections survive `baron update`
- user-written `DOMAIN_LANGUAGE.md` survives init/update
- only Superpowers is classified as workflow core
- exactly three mandatory quality agents remain
- no Hallmark or Matt directory is installed
- compact context remains under the existing 20,000-character cap

- [ ] **Step 2: Add RED public-flow tests**

Assert the top-level help and root README still expose only the normal user
flow. Hallmark/Matt source names belong in notices and deep docs, not in the
primary install instructions.

- [ ] **Step 3: Run RED tests and implement only failing integration**

Run:

```powershell
cargo test -p baron-core --test control_plane
cargo test -p baron-core --test public_trust_docs
cargo test -p baron-cli --test adapter_cli
cargo test -p baron-cli --test release_smoke
```

Make the smallest routing, preservation, or generated-instruction changes
needed for these contracts. Do not add a new user command.

- [ ] **Step 4: Complete the 3.5 behavior certification**

The certification report must include:

- exact Hallmark and Matt source revisions
- accepted and rejected technique matrix
- three frontend pressure-test comparisons
- one deep-module interface fixture
- one ambiguous domain-term fixture
- adapter parity and preservation evidence
- confirmation that no external installer, live instruction, or workflow skill
  was imported

- [ ] **Step 5: Run the complete release gate**

Run:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
git diff --check
```

Expected: all commands pass. No ignored Baron-owned test may be counted as
proof.

- [ ] **Step 6: Bump to `3.5.0` only after the gate passes**

Update workspace crates, lockfile, source-visible version assertions, status,
README, release docs, current plan, and certification so all report `3.5.0`.
Re-run the full release gate and `target/release/baron.exe --version`.

- [ ] **Step 7: Commit and push verified 3.5 source**

```powershell
git add Cargo.toml Cargo.lock crates assets README.md docs notes
git commit -m "release: certify Baron 3.5 skill intelligence"
git push origin main
```

Do not create a tag or GitHub Release unless the user explicitly authorizes
release promotion.

## Plan Self-Review

- No second workflow skill is introduced.
- Hallmark strengthens only `frontend-design`.
- Matt techniques strengthen only existing interface and Product Harness
  owners.
- Domain language is project-owned, evidence-backed, bounded, and isolated.
- Normal user commands do not change.
- Version `3.5.0` is gated by fresh full-suite evidence.
