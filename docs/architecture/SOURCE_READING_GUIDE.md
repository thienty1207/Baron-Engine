# Baron Engine Source Reading Guide

This guide helps a new AI agent inspect the checked-out implementation from
the product boundary through source, tests, and CI. It is a procedure, not a
claim that the repository has already been read.

## Authority and ownership

- Start with [`AGENTS.md`](../../AGENTS.md) and the canonical
  [handoff](../superpowers/plans/CURRENT.md). The handoff states what is
  current; dated plans and old status sections are historical evidence.
- Baron Core owns product semantics and durable state. Codex and Claude are
  thin native projections over Core.
- `assets/core/**` is the canonical managed runtime content. Adapters render
  host-specific bridges; installed `.baron/core/**` is a projection, not a
  second source of truth.
- Vault Markdown is durable memory. SQLite and search indexes are rebuildable
  accelerators. Project ID, not directory basename, is the memory boundary.
- When docs, old plans, and source disagree, verify the live behavior against
  the current source, tests, and CI. Record the discrepancy; do not silently
  treat an old plan as an instruction to restore old behavior.

## Read order

1. `AGENTS.md`, `README.md`, and `docs/superpowers/plans/CURRENT.md`.
2. `docs/BARON_STATUS.md`, `docs/BARON_STATUS.json`, and
   `notes/build-log/CURRENT.md` for release state and fresh verification.
3. `Cargo.toml`, `Cargo.lock`, and each crate's `Cargo.toml`.
4. Architecture contracts: `ARCHITECTURE.md`, `MEMORY_MODEL.md`,
   `ADAPTERS.md`, `CAPABILITY_REGISTRY.md`, `COMMAND_SURFACE.md`, and
   `CONTEXT_COMPILER.md`.
5. `crates/baron-core/src/lib.rs`, then every inventoried Core Rust source
   file, including private modules and `#[cfg(test)]` code.
6. `crates/baron-adapters/src/lib.rs`, then every inventoried adapter source
   file; follow calls back into Core and inspect preservation/update behavior.
7. `crates/baron-cli/src/main.rs`, `self_update.rs`,
   `update_transaction.rs`, and every other inventoried CLI source file.
8. `assets/core/**`, all three crates' integration tests and relevant fixtures,
   then every `.github/workflows/*.yml` workflow.

The exact files change over time. Regenerate the inventory in the current
checkout instead of relying on counts or filenames written in an older report.

## Generate the coverage inventory

From the repository root in PowerShell:

```powershell
$roots = @(
  'crates/baron-core/src',
  'crates/baron-cli/src',
  'crates/baron-adapters/src',
  'assets/core',
  'crates/baron-core/tests',
  'crates/baron-cli/tests',
  'crates/baron-adapters/tests',
  '.github/workflows'
)
rg --files --hidden -g '!target/**' -g '!.git/**' $roots | Sort-Object
```

For a Rust-only inventory, filter the command output with
`Where-Object { $_.EndsWith('.rs') }`. Record generated, ignored, vendored, or
unavailable paths separately; do not silently omit them. Include modified and
untracked source files in addition to tracked paths.

Use the module roots to check the inventory and discover nested/private code:

```powershell
rg -n '^(pub )?mod |^mod ' crates/baron-core/src crates/baron-adapters/src crates/baron-cli/src
```

Then read each listed file completely. For long files, use bounded line
windows and record every window until the file is covered; searching for names
or reading only declarations is not a full read.

## Trace the system by behavior

After the file-by-file pass, trace these flows across Core, adapters, CLI,
assets, tests, and workflows:

| Flow | Core areas to trace | Cross-check |
|---|---|---|
| Setup and project identity | `config`, `identity`, `safe_io`, `vault`, `state_guard` | CLI `setup`/`init`, adapter install, lifecycle tests |
| Request preparation and routing | `prepare`, `control_plane`, `context`, `survey`, `platform`, `work_shape`, `risk`, `intelligence*` | `PrepareRequestV1`/`PreparePacketV1`, Codex and Claude projections, prepare/route tests |
| Trusted project memory | `memory`, `firewall`, `semantic`, `knowledge`, `domain_language`, `session`, `session_replay` | Vault fixtures, project-isolation tests, search tests |
| Plan, continuity, and recovery | `operation*`, `intent`, `task_state`, `plan`, `continuity`, `harness*` | CLI plan/recovery commands, multi-checkout and crash-boundary tests |
| Proof, gates, and completion | `execution_receipt`, `receipt_authority`, `proof`, `trace`, `review_gate`, `capability` | receipt authority and completion tests, workflow gates |
| Hooks and automation | `automation`, hook correlation, `autopilot`, `capability` | Codex/Claude hook assets, idempotency/replay tests |
| Install, update, migration, release | adapter `install`/`managed`/`update`, CLI self-update/transaction, Core `migration`/`asset_lifecycle`/`release`/`certification` | preservation, rollback, installer, signing, and CI tests |

The table is an index, not a substitute for reading every file. Follow each
public command from its CLI parser through adapter translation into Core, and
follow Core-owned runtime assets back to the installer and tests.

## Coverage ledger and reporting

Keep a temporary or task-local ledger with one row per inventoried path:

| Path | Read fully | Callers/callees traced | Tests or workflow checked | Notes/unknowns |
|---|---|---|---|---|
| `crates/baron-core/src/lib.rs` | yes/no | ... | ... | ... |

Before reporting, reconcile the ledger against a freshly generated inventory.
State counts for each group, paths not read, and why they were excluded. Say
“complete source pass” only when every in-scope source file is marked read in
full and the integration tests/workflows have been checked. Otherwise report
the exact partial coverage. Do not expose secret values while reporting
configuration or fixtures.

## Verification after a code change

Follow `AGENTS.md` for required format, workspace tests, and Clippy commands.
For adapter or documentation changes, also run its focused Core and adapter
contract suites, CLI help checks, relative-link validation, status JSON
parsing, and the tracked retired-adapter gate. Distinguish local results from
hosted CI; never imply a pushed branch has a PR or hosted CI unless verified.
