# Baron 3.4 Safe Self-Update Design

Date: 2026-07-23
Status: approved roadmap design
Target release: `3.4.0`

## Problem

Baron currently has two separate update stories:

- `baron update` refreshes Baron-managed project assets from the already installed binary.
- The PowerShell and shell installers download and replace the Baron binary.

That split is safe, but it is not the simple long-term experience Baron promises.
The user should be able to run one public command, `baron update`, without
losing project instructions, custom skills, custom agents, Vault memory, or a
working Baron installation.

The difficult part is not downloading a newer executable. The difficult part is
deciding what to do when all three of these versions are different:

- the managed content Baron originally installed
- the content the user or an agent changed locally
- the managed content shipped by the new Baron release

A safe update must preserve local ownership, refuse ambiguous overwrites, and
remain recoverable if the process, machine, or network stops at any point.

## Non-Negotiable Contract

- Superpowers remains the workflow core.
- The mandatory quality agents remain `code-reviewer`, `security-auditor`, and
  `test-engineer`.
- Vault Markdown remains the durable memory source of truth.
- SQLite remains a rebuildable cache and is not part of the update transaction.
- `baron update` remains the only normal user command for updating Baron.
- AI automation must not silently download or activate a new Baron release.
- AI may repair current installed assets through the hidden local-only
  `baron automation reconcile` flow.
- Custom skills, custom agents, custom routing blocks, project source, plans,
  Harness records, and Vault memory are outside Baron's overwrite authority.
- No update may claim success before release identity, project activation, and
  runtime activation have proof.
- Unknown or conflicting state stops the update instead of being guessed.

## User Experience

Normal update:

```powershell
baron update
```

Expected behavior:

1. Baron checks whether a newer verified release exists.
2. Baron downloads only the executable for the current platform.
3. Baron verifies release version, source revision, target, size, and SHA-256.
4. Baron computes the project update without changing managed targets.
5. If the update is unambiguous, Baron applies it transactionally.
6. Baron activates the new runtime and records a receipt.
7. Baron reports the installed version and preserved local assets.

If no newer release exists, Baron refreshes the registered adapters from the
installed runtime and reports that the binary was already current.

If a managed file has a real conflict, Baron writes a bounded conflict packet
under `.baron/update/` and stops before modifying managed targets. The AI may
explain the conflict and prepare the resolved copy, but final application
requires explicit user authority.

## One-Time Bootstrap Boundary

Baron `3.3.x` cannot gain self-update behavior retroactively. A user must install
`3.4.0` once through the existing checksum-verified installer. From `3.4.0`
onward, `baron update` owns the complete update experience.

This limitation must be stated plainly in release documentation. Baron must not
pretend an older binary can execute code it does not contain.

## Ownership Model

Every Baron-managed file is classified by one merge policy:

| Policy | Examples | Baron ownership |
| --- | --- | --- |
| Managed marker | `AGENTS.md`, `CLAUDE.md`, `AGENT.md` | only text between Baron markers |
| Managed routing | skill and agent indexes | managed routing block only |
| Structural JSON | Codex/Claude settings or hooks | Baron-owned keys and entries only |
| Managed full file | bundled core skill/agent assets | entire file, with baseline conflict detection |
| User-owned | custom skills, custom agents, source, Vault memory | never overwritten |

The installed baseline is recorded at:

```text
.baron/managed-state/
  manifest.json
  base/
    codex/
    claude/
    agent/
```

`manifest.json` records stable relative paths, merge policy, installed version,
and baseline SHA-256. The `base/` tree stores the exact managed content that
Baron last activated. It is not a second source of truth; it is the merge
ancestor required to distinguish a user edit from an upstream change.

## Three-Way Decision Matrix

For each managed target, Baron compares:

- `BASE`: last successfully installed managed content
- `LOCAL`: current project content
- `UPSTREAM`: content embedded in the verified candidate

The deterministic rules are:

| Condition | Result |
| --- | --- |
| `LOCAL == BASE` | take `UPSTREAM` |
| `UPSTREAM == BASE` | keep `LOCAL` |
| `LOCAL == UPSTREAM` | keep one copy |
| only Baron marker changed upstream | replace marker, preserve surrounding local text |
| only Baron-owned JSON entries changed upstream | merge those entries, preserve user entries |
| local and upstream edits are provably non-overlapping | auto-merge |
| edits overlap or ownership is uncertain | stage conflict and stop |

Full managed files use a conservative rule. If both local and upstream content
changed from the same base and Baron cannot prove a non-overlapping line merge,
the result is a conflict. Preserving data is more important than completing an
update without interruption.

## Update Workspace

Every update uses a unique transaction directory:

```text
.baron/update/
  state.json
  BASE/
  LOCAL/
  UPSTREAM/
  RESOLVED/
  backups/<transaction-id>/
  receipts/<transaction-id>.json
```

`state.json` contains:

- transaction id
- source and target versions
- exact release source revision
- repository identity
- adapter set
- current platform target
- candidate executable path and checksum
- transaction status
- hashes for every staged input and output
- last successful checkpoint
- rollback metadata

Supported states are:

```text
discovered
downloaded
verified
planned
conflict
project_activated
runtime_pending
completed
rolled_back
aborted
```

State transitions are monotonic. A continuation is refused if staged files,
project identity, managed targets, candidate binary, or transaction metadata
changed after the conflict packet was created.

## Conflict Continuation

For every conflict Baron stores the exact `BASE`, `LOCAL`, and `UPSTREAM`
versions plus one empty or partially merged `RESOLVED` file.

The intended AI flow is:

1. Read the bounded conflict summary.
2. Compare all three versions.
3. Explain the concrete behavior that differs.
4. Ask the user which behavior should survive when intent is not already clear.
5. Edit only the staged `RESOLVED` copy.
6. Run hidden `baron update --continue`.

`--continue` validates all frozen hashes before writing. `--abort` removes only
the staged transaction and does not alter project files or Vault memory. These
flags are recovery surfaces and are not promoted in the normal README command
flow.

## Verified Release Candidate

The immutable release manifest remains the release identity boundary. Version
`3.4.0` extends it with one raw update candidate per supported target while
keeping existing install archives.

Each candidate record contains:

- target triple
- executable filename
- version
- source revision
- SHA-256
- byte size

The updater accepts only HTTPS release sources in production. A test-only
directory source is injected through the update service interface; tests never
depend on the public network.

Before a candidate may plan or write a project update, Baron proves:

- manifest product and schema are supported
- candidate version is newer than the active version
- downgrade is refused
- target triple matches the running platform
- candidate size and SHA-256 match the immutable manifest
- manifest source revision is a valid exact commit identity
- running the staged candidate with `--version` reports the expected version

Any failure leaves managed project targets and the active binary unchanged.

## Candidate-Owned Project Activation

The verified candidate, not the old runtime, renders the new managed assets.
The active Baron process stages and verifies the candidate, then delegates the
update plan and application to that candidate through a hidden protocol.

This avoids a false update where the old binary refreshes old assets and only
afterward installs a new executable.

The candidate protocol receives only:

- repository path
- transaction state path
- expected project identity
- expected source and target versions

It revalidates all fields before use. It does not accept arbitrary output paths
or paths that escape the repository's `.baron/update/` workspace.

## Runtime Activation And Recovery

On Unix-like platforms Baron can atomically rename the verified candidate into
the installed binary location after project activation.

On Windows the running executable may be locked. Baron therefore launches a
small delayed finalizer from the already verified candidate, records
`runtime_pending`, exits, and lets the finalizer:

1. wait for the parent process to close
2. back up the installed executable
3. move the candidate into place
4. run `baron --version`
5. write a completed receipt
6. restore the backup if validation fails

At the start of every later Baron invocation, a bounded recovery check inspects
unfinished transactions. It either completes a verified pending handoff or
restores the previous binary and project backup. It never guesses success from
the absence of an error message.

Project activation and runtime activation are one logical transaction. If the
runtime cannot be activated, Baron restores the previous managed project files
so the installed binary and project assets stay compatible.

## Automation Boundary

Generated Codex, Claude, and generic instructions currently suggest
`baron update` for state mismatch. That becomes unsafe once `baron update` can
contact the network and activate a new release.

Baron `3.4.0` changes the contract:

- Humans run public `baron update` to authorize a release update.
- AI agents run hidden `baron automation reconcile` for local repair using the
  already installed runtime.
- Reconcile never downloads, replaces the binary, changes version, or bypasses
  conflict rules.
- State mismatch still stops durable work until local repair succeeds or the
  user authorizes an update.

## Security And Privacy

- No credentials are stored in update state.
- Release URLs, manifest identity, and hashes are recorded for audit.
- Candidate paths and managed paths are canonicalized and constrained.
- Symlink or junction escapes from `.baron/update/` are rejected.
- Vault files are excluded from the update write set.
- Project source files are excluded from the update write set.
- Custom skill and agent paths are excluded unless they are registered Baron
  managed assets with a matching baseline.
- Logs redact query strings and authentication-like values from source URLs.

## Phase Boundaries

### Phase 35 - Managed Baseline And Update Planner

Build the local ownership ledger and a read-only three-way planner. No network
download and no binary replacement are allowed in this phase.

### Phase 36 - Verified Release Candidate And Binary Handoff

Add release candidate records, secure resolution, download, exact verification,
and platform-specific handoff primitives. No candidate may change project files
until every identity check passes.

### Phase 37 - Conflict-Safe Activation And Recovery

Add transactional project activation, conflict staging, continuation, abort,
rollback, delayed Windows finalization, and crash recovery.

### Phase 38 - Automation Contract And Baron 3.4 Certification

Separate human-authorized release updates from AI local reconciliation, update
all adapters and docs, bump to `3.4.0`, and certify the complete flow across
supported platforms.

## Out Of Scope

- Background or scheduled release installation
- Silent AI-authorized network updates
- Updating project dependencies
- Modifying source architecture during Baron update
- Migrating or compacting Vault memory as part of update
- A GUI update manager
- Replacing immutable release promotion rules

## Acceptance Standard

Baron `3.4.0` is complete only when:

- a normal user needs only `baron update`
- an old `3.3.x` installation has a documented one-time bootstrap path
- custom and user-owned assets survive
- conflicting managed edits cannot be overwritten silently
- a failed download, checksum, candidate, merge, write, process interruption, or
  Windows executable handoff leaves a recoverable prior state
- AI automation cannot silently authorize a remote update
- source, docs, tests, runtime assets, release manifest, and installers agree
- the full no-skip verification and real update lifecycle smoke pass
