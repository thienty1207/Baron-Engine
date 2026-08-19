# Baron Reasonix Adapter Compatibility Design

## Decision

Add DeepSeek-Reasonix as a first-class Baron adapter without changing the
Baron intelligence engine, memory model, generation selector, or release
version. The public/source version remains `4.2.0`; this is a maintenance
adapter track, not Baron 4.3.

## Shared-brain contract

Codex, Claude, generic agents, and Reasonix are frontends over one Baron
project identity and one Vault. The adapter name is provenance on lifecycle
events, never a memory namespace.

The following remain shared and unchanged during a switch:

- `.baron/project.toml` project identity and platform routing;
- `.baron/local.toml` machine-local Vault routing;
- Vault Markdown, memory index inputs, session replay, Wiki, and CodeGraph;
- plan, Harness, continuity, proof, trace, decisions, and recovery state.

The project configuration gains an optional `active_adapter` field. Existing
projects without the field continue to use the first registered adapter. A
switch changes only the active adapter and registers the target adapter when
needed; it never changes `project_id` or the Vault path.

## Reasonix surface

The adapter exposes:

- `baron init --reasonix [platform]`;
- `baron context --reasonix`;
- `baron capability check --adapter reasonix`;
- `baron runtime check --adapter reasonix`;
- `baron automation hook <event> --adapter reasonix`;
- `baron adapter status` and `baron adapter switch --to reasonix`.

The installation owns a managed `REASONIX.md` bridge and bounded
`.reasonix/commands` assets. Reasonix settings/hooks are merged only when the
existing JSON is Baron-owned or has an explicit Baron marker. Unmarked user
settings, bridge scripts, and instructions are preserved and reported as
conflicts; the installer never silently replaces them.

## Switch and rollback

`baron adapter switch --to <adapter>` records a shared continuity checkpoint,
performs a project/Vault identity preflight, installs only safe managed assets,
sets `active_adapter`, and records one shared adapter-switch event. The reverse
switch restores the prior active adapter while retaining all memory and logs.
Adapter-local runtime leases are disposable; durable Baron history is written
through the common Vault/continuity path.

## Preservation rules

1. New target files may be created.
2. A file with a Baron managed marker and an unchanged baseline may be updated.
3. A file without a marker is user-owned and is not overwritten.
4. A changed managed block is a conflict; Baron reports it and leaves the live
   file untouched.
5. User hooks/settings are merged only in the Baron-owned portion; malformed,
   ambiguous, or unmarked settings fail closed with a diagnostic.
6. Existing Codex/Claude files remain available for rollback and are not
   deleted by a Reasonix switch.

## Non-goals

- no 4.3 version or engine-generation bump;
- no replacement of Vault, memory, Wiki, CodeGraph, or the 4.2 fallback paths;
- no DeepSeek API/model hard-coding;
- no automatic adoption of an existing user-written Reasonix bridge;
- no invented Reasonix hook protocol beyond the verified command/settings
  surface.

## Acceptance evidence

The adapter is complete only when tests prove Codex -> Reasonix -> Codex reads
and writes the same project-bound Vault history, preserves user-owned files,
rejects ambiguous conflicts without writes, and keeps all existing Codex,
Claude, generic, memory, release, and fallback tests green.
