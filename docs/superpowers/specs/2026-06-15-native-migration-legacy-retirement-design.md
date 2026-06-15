# Baron Native Migration And Legacy Retirement Design

## Goal

Move useful project knowledge from Agent Bootstrap into Baron-native structures,
prove that the conversion is complete, and retire the legacy runtime without
losing user-owned content.

## Core Decision

Migration is a transactional Baron takeover, not an in-place rename and not a
permanent compatibility layer.

Baron treats Agent Bootstrap as an import format only. After a successful
migration, the project runs through Baron configuration, Baron adapters, Baron
memory, Baron plans, Baron Product Harness, Baron proof, and Baron traces.

## User Flow

```bash
baron migrate agent-bootstrap [repo-path] --dry-run
baron migrate agent-bootstrap [repo-path]
baron migrate status [repo-path]
baron migrate rollback --id <migration-id> [repo-path] --vault <vault-path>
```

The apply command repeats inventory internally. The user does not need to run
dry-run first.

## Transaction Stages

1. Inventory the legacy repo and source Vault without writing.
2. Classify every relevant asset as import, preserve, quarantine, remove, or
   ignore.
3. Create an immutable rollback backup under
   `Vault/Artifacts/Baron/Migrations/<migration-id>/`.
4. Import legacy Vault memory into the Baron project capsule.
5. Convert repo plan, harness, proof, and trace material into `docs/baron/`.
6. Validate custom skills and agents against Baron-native contracts.
7. Install fresh Baron configuration, indexes, adapters, core agents, and
   skills.
8. Verify source/import counts and content hashes.
9. Remove only positively identified Agent Bootstrap managed assets.
10. Rebuild the Baron memory index and write a migration receipt.

If any stage after backup fails, Baron restores the repo and Baron Vault paths
from the backup and records the failure.

## Data Classification

### Import

- Vault project facts, decisions, tasks, open questions, handoff, research,
  notes, sessions, plans, Product Harness, exports, and backups.
- Repo plans under `docs/superpowers/plans/`.
- Product documents under `docs/product/`, `docs/stories/`,
  `docs/validation/`, and `docs/decisions/`.
- User-created custom skills and custom agents that pass validation.

### Preserve In Place

- Product source code.
- User README content.
- User text outside managed instruction markers.
- User-created files not listed in the Agent Bootstrap manifest or known
  managed paths.
- User plans under root `plans/`.

### Quarantine

- Custom skills without a valid `SKILL.md`, precise trigger, or safe boundary.
- Custom agents without required metadata, evidence contract, or recursion
  boundary.
- Custom assets that claim Superpowers workflow ownership or replace a Baron
  core quality gate.

Quarantined assets are copied to
`.baron/quarantine/<migration-id>/` and the Vault migration artifact. They are
not activated automatically.

### Remove After Verification

- `vault.config.json`
- `.agent-bootstrap-manifest.json`
- `scripts/agent-memory.js`
- Agent Bootstrap managed Git hook when its content matches the managed hook
- Agent Bootstrap managed blocks in root instructions
- Agent Bootstrap bundled skills, core agents, indexes, commands, and config
  after fresh Baron Codex assets are installed
- Legacy generated bridge files whose path and ownership are proven by the
  manifest or known managed signatures

No broad directory deletion is allowed without first preserving unknown custom
content.

## Legacy Ownership Rules

Baron identifies legacy ownership using, in descending confidence:

1. A path and synchronized hash from `.agent-bootstrap-manifest.json`.
2. A known Agent Bootstrap marker block.
3. A known managed path plus a recognized content signature.

Unknown or modified files are preserved or quarantined. They are never deleted
merely because they live under `.codex`, `docs`, `plans`, or `scripts`.

## Custom Asset Contract

An imported skill must:

- contain `SKILL.md`
- declare a name and a precise use trigger
- avoid recursively loading unrelated skills
- not claim Superpowers workflow ownership
- not bypass proof or trace quality gates
- not contain Agent Bootstrap runtime commands

An imported agent must:

- contain a name, description, and developer instructions
- have a narrow role and evidence-backed output contract
- not orchestrate other agents
- not replace `code-reviewer`, `security-auditor`, or `test-engineer`
- not claim completion without proof
- not contain Agent Bootstrap runtime commands

## Baron-Native Core Rewrite

Phase 6 removes remaining Agent Bootstrap wording from Baron core assets.

- The three core agents explicitly name Baron, Superpowers, the Vault firewall,
  proof requirements, and trace gates.
- `frontend-design` remains an attributed optional domain skill.
- `vibe-security-scan` remains an attributed defensive optional domain skill,
  but its routing and execution language becomes Baron-native.
- Superpowers remains the only workflow core.

## Backup And Rollback

Each migration backup contains:

- `manifest.json` with migration id, paths, hashes, and classifications
- `repo/` copies of every repo file Baron may modify or remove
- `vault/` copies of source and destination Vault paths touched by migration
- `quarantine/` copies of rejected custom assets
- `receipt.json` after success or `failure.json` after rollback

Rollback restores only paths recorded in the manifest. It does not overwrite
unrelated files created after migration.

## Verification Gates

Cleanup is allowed only when:

- every imported source file has a matching destination hash or recorded
  normalized conversion hash
- all imported record counts match
- Baron project and local config load successfully
- the selected adapter is installed
- Baron memory index rebuild succeeds
- core asset contract validation passes
- no active file depends on `agent-bootstrap` or `scripts/agent-memory.js`

Migration success is not inferred from command completion alone.

## Non-Goals

- Supporting Agent Bootstrap as a second runtime.
- Deleting the legacy source Vault.
- Automatically activating unsafe custom assets.
- Migrating arbitrary third-party agent kits in Phase 6.
- Replacing Markdown source of truth with SQLite.

