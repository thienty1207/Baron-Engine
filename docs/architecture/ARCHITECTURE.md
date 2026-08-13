# Baron Architecture

Baron is split into a core engine and adapter outputs.

```text
baron-cli
  -> baron-core
       -> survey engine
       -> memory engine
       -> read-only memory consolidation candidates
       -> guarded Baron 4.0 intelligence selector (3.8 recovery fallback)
       -> cited Wiki link graph and local CodeGraph candidate paths
       -> bounded defensive security router and authorization gate
       -> context compiler
       -> plan engine
       -> harness engine
       -> proof engine
       -> trace engine
       -> control-plane engine
       -> harness improvement engine
       -> work-shape decision engine
       -> trusted execution receipt ledger
       -> measured Harness experiment ledger
       -> bounded application runbook reader
       -> certification engine
       -> release metadata and checksum verifier
  -> baron-adapters
       -> codex adapter
       -> claude adapter
       -> generic agent adapter
```

GitHub Actions builds the same `baron-cli` on four native runner targets.
Installers are thin lifecycle clients around those verified archives; they are
not another runtime and never own project or Vault data.

## Data Flow

```text
repo + vault + user task
  -> survey/context compiler
  -> memory firewall
  -> active plan and harness state
  -> work-shape decision (read-only, focused, durable, or confirmation)
  -> control-plane route and trusted quality-gate receipt evidence
  -> bounded application runbook when the task needs runtime operations
  -> harness audit and improvement loop
  -> certification gate when release confidence matters
  -> adapter-specific context output
  -> agent work
  -> proof + trace + memory write-back
```

## Source Hierarchy

1. User request and repo files.
2. Verified project memory.
3. Active plan and product harness state.
4. Verified global memory.
5. Cross-project memory only when explicitly matched.
6. Stale/unknown memory as reference only.

## Safety Model

- Shadow mode reads only.
- Update mode must preserve user-owned files.
- Adapter files use managed markers where possible.
- Baron must never mark completion without verification evidence.
- Baron must never count a mandatory skill/agent gate without recorded gate
  evidence.
- A hand-written or stale proof sentence is reported evidence only. Trusted
  proof and gate completion require a Baron-owned receipt bound to the current
  project source.
- Harness interventions remain candidates until explicitly approved, then
  require a comparable fresh-agent rerun before keep/revise/remove.
- Project operation facts belong to the repository-owned runbook; missing
  readiness, interface, credentials, ports, fixtures, and cleanup ownership
  remain unknown rather than being invented.
- Baron may propose harness improvements, but must not rewrite core policy or
  architecture without human approval.
- Baron must never promote cross-project memory as truth without confidence.
- Release installers must verify SHA-256 and staged binary version before
  replacing the active executable.
- Rollback and uninstall must never traverse into project or Vault paths.

## Optional Local Code Map

The optional `graphify-local` provider is a project-scoped code-navigation
accelerator for large or older repositories. It is not part of Baron's memory,
instruction, hook, workflow, or global-context systems.

- Baron accepts only its pinned compatible local provider version.
- Baron permits only a version probe, code-only extraction into
  `.baron/cache/code-graph/`, and bounded local JSON queries.
- Provider output is staged, size-checked, path-checked, identity-bound, and
  checksummed before the cache state changes.
- A missing, incompatible, stale, malformed, timed-out, or failed provider
  leaves the last known-good cache intact and returns to the Survey Engine.
- Graph results are navigation hints only. Current repository files remain the
  source required for implementation, proof, traces, and durable memory.
