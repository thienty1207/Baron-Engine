# Baron 3.5 Skill Intelligence Certification

Status: source-certified on `2026-07-24`; Git tag and GitHub Release not created

This record captures the evidence for the Baron-owned skill-intelligence
release. It is not a claim that any external skill, installer, or runtime is
required by a generated project.

## Source Boundaries

| Topic | Audited revision | Accepted locally | Rejected |
| --- | --- | --- | --- |
| Frontend composition research | `aeb42fb354ff4efa36ab475773a082315a3af2ce` | Brief fingerprint, bounded anti-template gates, responsive/state proof. | A second frontend skill, installer, command router, or live dependency. |
| Module and domain-language research | `ed37663cc5fbef691ddfecd080dff42f7e7e350d` | Deep module boundaries and project-scoped domain language. | Workflow, planning, TDD, debugging, review, handoff, setup, personal, or duplicate runtime skills. |

## Phase 39 Evidence

### Adapter And Routing Contracts

- `frontend_design_is_local_deep_and_has_one_routing_owner` installs Codex,
  Claude, and generic adapters. It proves each installation has the same local
  frontend skill and its three references, while no extra frontend owner is
  installed.
- The operational files are checked for live URLs, installer commands, and a
  second workflow declaration. Attribution remains in a notice file only.
- `frontend_route_stays_local_and_backend_work_does_not_load_it` proves a
  responsive checkout routes the existing frontend skill and all three quality
  gates, while a backend API task does not load frontend guidance.

### Controlled Pressure Fixtures

The pressure check is a deterministic guidance comparison, not a fabricated
model benchmark. The pre-3.5 baseline was inspected from source revision
`b268ccb`; the candidate is tested through installed skill assets and routing
contracts. A live model response is deliberately not recorded as proof because
model behavior varies by provider and prompt.

| Fixture | Baseline gap | Candidate required behavior | Result |
| --- | --- | --- | --- |
| Operational dashboard with long Vietnamese labels and dense repeated actions | No explicit anti-template or long-content evidence matrix. | Preserve operational density, reject interchangeable cards, and require narrow/wide plus long-content evidence. | Covered by `anti-template-gates.md` and `responsive-state-proof.md`. |
| Mobile checkout with loading, error, disabled, and payment states | State evidence existed only as a broad rubric. | Route frontend guidance, retain the three quality gates, and require state evidence without claiming unobserved coverage. | Covered by control-plane contract and responsive/state matrix. |
| Brand landing page with a generic purple-card baseline | No structured brief required before a visual composition. | Require product evidence, one justified macrostructure, three product signals, and rejected generic defaults. | Covered by `brief-fingerprint.md` and changed-surface gate. |

### Commands Run

```text
cargo test -p baron-adapters --test adapter_lifecycle frontend_design_is_local_deep_and_has_one_routing_owner -- --exact
cargo test -p baron-core --test control_plane
```

Both commands passed on the Phase 39 working tree. Full Baron 3.5
certification remains a Phase 41 gate.

## Phase 40 Evidence

- `api_interface_design_has_local_deep_module_guidance_without_duplicate_skills`
  proves Codex, Claude, and generic installations receive the same local
  boundary reference and no duplicate architecture or workflow skill.
- `domain_language.rs` proves first creation, no invented terms, byte-for-byte
  preservation of user terms, ambiguous-term labeling, bounded rendering, and
  strict project isolation inside a shared Vault.
- `context_compiler.rs` proves compact context includes only term, status, and
  evidence, not the full file body. The memory index deliberately excludes this
  file because its generic line parser would bypass that bound.
- When repo and Vault copies differ, Baron preserves both copies, withholds the
  terms from trusted compact context, and records the mismatch rather than
  silently choosing a definition. This is an additional safety decision made
  during implementation to protect durable memory from accidental overwrite.

### Commands Run

```text
cargo fmt --all -- --check
cargo test -p baron-core --test domain_language --test context_compiler
cargo test -p baron-adapters --test adapter_lifecycle
cargo test -p baron-cli --test adapter_cli
```

All commands passed on the Phase 40 working tree. Full Baron 3.5 certification
remains a Phase 41 gate.

## Phase 41 Evidence

### Ownership And Public Flow

- `control_plane.rs` proves Superpowers is the only workflow owner, exactly
  `code-reviewer`, `security-auditor`, and `test-engineer` are mandatory gates,
  and optional domain skills stay lazy and explainable.
- `adapter_lifecycle.rs` proves all three adapters install the same local
  frontend and interface references, while no `hallmark`, `matt-skills`,
  `codebase-design`, `domain-modeling`, `tdd`, `implement`, or `grilling`
  directory is installed.
- `public_trust_docs.rs` proves the normal README stays limited to install,
  Vault setup, agent/platform init, and update. Research-source names and deep
  automation commands do not enter that user flow.

### Preservation Fixture

`automation_reconcile_preserves_custom_routing_and_recovers_missing_domain_language`
first failed, then passed after the local reconcile path restored only missing
Baron-owned support documents. The fixture proves all of the following in one
real temporary project and Vault:

- user-written repo and Vault Domain Language content survives byte-for-byte;
- custom skill and agent files survive;
- custom skill and agent routing entries survive;
- deleting the two managed Domain Language files causes them to be recreated;
- reconcile never downloads a release or replaces the runtime.

### Deep-Module And Ambiguous-Term Fixtures

- The deep-module fixture installs
  `api-and-interface-design/references/deep-module-boundaries.md` under Codex,
  Claude, and generic adapters and rejects duplicate architecture/workflow
  directories.
- The ambiguous-term fixture preserves both divergent copies, labels the
  mismatch, and withholds their terms from trusted compact context instead of
  selecting a meaning silently.

## Accepted And Rejected Technique Matrix

| Existing Baron owner | Accepted locally | Explicitly rejected |
| --- | --- | --- |
| `frontend-design` | Evidence brief, anti-template checks, responsive and state proof. | A second frontend skill, installer, command, or live instruction. |
| `api-and-interface-design` | Boundary invariants, explicit dependency direction, public-behavior seams, compatibility evidence. | Workflow, planning, TDD, debugging, review, handoff, setup, personal, or duplicate architecture skills. |
| Product Harness Domain Language | Evidence-backed canonical terms, ambiguous status, bounded rendering, project isolation. | Invented terms, automatic conflict resolution, cross-project sharing, or generic unbounded memory excerpts. |

No external installer, live instruction, or workflow skill was imported. All
operational assets are local Baron files; source attribution remains isolated in
notice/provenance material.

## Verification Gate

| Command | Result before version bump | Final result after bump |
| --- | --- | --- |
| `cargo test -p baron-core --test control_plane --test public_trust_docs` | passed | passed |
| `cargo test -p baron-cli --test adapter_cli --test release_smoke` | passed | passed |
| `cargo fmt --all -- --check` | passed after formatting the new test | passed |
| `cargo test --workspace --all-targets --no-fail-fast` | passed | passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed | passed |
| `cargo build --release --locked -p baron-cli` | passed | passed; `target/release/baron.exe --version` reported `baron 3.5.0` |
| `git diff --check` | passed | passed |

## Release Boundary

Source `3.5.0` is not a Git tag or GitHub Release. A release promotion remains
a human-authorized action after the source push.
