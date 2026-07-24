# Baron 3.5 Skill Intelligence Certification

Status: in progress

This record captures the evidence for the Baron-owned skill-intelligence
release. It is not a claim that any external skill, installer, or runtime is
required by a generated project.

## Source Boundaries

| Topic | Audited revision | Accepted locally | Rejected |
| --- | --- | --- | --- |
| Frontend composition research | `aeb42fb354ff4efa36ab475773a082315a3af2ce` | Brief fingerprint, bounded anti-template gates, responsive/state proof. | A second frontend skill, installer, command router, or live dependency. |
| Module and domain-language research | `ed37663cc5fbef691ddfecd080dff42f7e7e350d` | Pending Phase 40. | Workflow, planning, TDD, debugging, review, handoff, setup, personal, or duplicate runtime skills. |

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

## Remaining 3.5 Gates

- Phase 40 project-scoped domain language and deep-module guidance.
- Phase 41 adapter preservation, public-flow, full workspace, static, release,
  and smoke verification.
