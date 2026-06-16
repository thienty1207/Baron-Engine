# Baron Architecture

Baron is split into a core engine and adapter outputs.

```text
baron-cli
  -> baron-core
       -> survey engine
       -> memory engine
       -> context compiler
       -> plan engine
       -> harness engine
       -> proof engine
       -> trace engine
       -> control-plane engine
       -> harness improvement engine
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
  -> control-plane route and quality-gate evidence
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
- Baron may propose harness improvements, but must not rewrite core policy or
  architecture without human approval.
- Baron must never promote cross-project memory as truth without confidence.
- Release installers must verify SHA-256 and staged binary version before
  replacing the active executable.
- Rollback and uninstall must never traverse into project or Vault paths.
