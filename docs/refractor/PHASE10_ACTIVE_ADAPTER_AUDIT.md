# Phase 10 `active_adapter` Audit

Date: 2026-09-08

Phase 10 leaves the serialized project field in place for compatibility. The
field is no longer read to authorize or select a correctness-sensitive runtime
operation.

| Location | Use after Phase 10 | Classification |
| --- | --- | --- |
| `crates/baron-core/src/config.rs` `ProjectConfig.active_adapter`, `active_adapter`, initialization, and `set_active_adapter` | Parse, preserve, write, and expose legacy project preference | Migration compatibility and diagnostics/UI |
| `crates/baron-cli/src/main.rs` adapter status, switch, and root shortcut output | Show or mutate the explicit compatibility/UI preference; output declares it non-authoritative | Diagnostics/UI and compatibility |
| `crates/baron-cli/src/update_transaction.rs` fixture field | Build a serialized update fixture with the compatibility field absent | Test/migration compatibility |
| `crates/baron-core/src/capability.rs` registered adapter reporting | Evaluate registered Codex/Claude adapters for diagnostics without consulting the global preference | Diagnostic report only |
| `crates/baron-core/src/proof.rs`, `control_plane.rs`, `prepare.rs`, `context.rs`, `automation.rs`, and `continuity.rs` | Use `OperationContext` or adapter-neutral behavior; the legacy capability-proof wrapper fails closed instead of promoting cached adapter state | Runtime correctness authority removed |

The strict runtime type is `baron_core::operation::SupportedAdapter`, whose only
values are Codex and Claude. `OperationContext` carries that adapter together
with optional session and request identifiers. Generic and historical adapter
values remain in the legacy config/hook boundary for the later migration phase;
they cannot construct a new runtime operation context.

The verification command used for this audit is:

```text
rg -n "active_adapter" crates/baron-core/src crates/baron-cli/src
```

Its remaining matches are limited to the compatibility/UI and fixture rows
above; no correctness path calls `active_adapter`.

## Pre-implementation target RED evidence

The Phase 10 Core target first failed to compile because the repository had no
operation identity module or explicit route/context, proof, continuity, and
journal entry points. After those target seams were added, the CLI target
observed five behavioral failures before the implementation change: omitted
capability checks and runtime checks silently used the serialized active value,
omitted continuity checkpoints did the same, capability-bearing proof had no
explicit adapter input, and switch preview did not declare its compatibility
only status. The reciprocal explicit Claude prepare test was the one target
that already passed because Phase 5 prepare accepted an explicit adapter.

These failures were expected target RED: they exposed project-global adapter
authority and missing attribution seams. They were not weakened; the final
Phase 10 targets now pass with explicit Codex/Claude identity and fail closed
when correctness-sensitive identity is omitted.
