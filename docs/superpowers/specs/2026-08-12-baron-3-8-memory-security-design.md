# Baron 3.8 Memory, Knowledge, And Security Design

Date: 2026-08-12  
Status: implemented in the `3.8.0` source candidate; public release proof pending

## Decision

Baron 3.8 extends the existing Vault, Memory Firewall, Continuity Ledger,
Control Plane, Proof/Trace, and optional CodeGraph owners. It does not create a
second workflow core, remote memory service, team platform, or UI.

## Durable Boundary

Vault Markdown and current repository files remain the source of truth. SQLite,
Wiki JSON, local CodeGraph JSON, Resume Briefs, and benchmark reports are
bounded derived views. They must be disposable, project-ID keyed, redacted, and
rebuildable after deletion or corruption.

## Knowledge Flow

```text
repo + Vault + current task
  -> project identity and source fingerprint
  -> layered memory and Memory Firewall
  -> bounded Resume Brief
  -> cited Wiki and local CodeGraph hints
  -> adapter context / agent action
  -> proof, trace, checkpoint, and evidence-backed write-back
```

The memory recall path keeps local lexical search as the baseline and adds a
deterministic character-ngram hybrid score. This improves close-language and
identifier matching without requiring a paid embedding service. Project and
trust filters happen before a candidate can influence ranking.

## Memory Layers

- Evidence: sessions, notes, research, and questions; never silently promoted.
- Verified: proof and trace records with current execution evidence.
- Decision: approved decisions, plans, and Harness state.
- Invariant: verified current facts that are not candidates or warnings.

Candidate, stale, contested, and inferred information remains labeled. A
Resume Brief must expose the source revision, current objective, checkpoint,
decisions, blocker, affected files, proof/trace state, unknowns, and next safe
action within a bounded character budget.

## Wiki And Local CodeGraph

Wiki indexes only project README/documentation Markdown and stores heading-aware
citations, hashes, revision, and stale state. Local CodeGraph extracts bounded
symbols and conservative inferred reference edges for Rust, TypeScript/
JavaScript, Python, and Go. Unsupported, stale, corrupt, oversized, or missing
data falls back to Survey/lexical recall. Graph results are advisory until
current source verification.

## Security Boundary

All paths are canonicalized and project-ID bound. Index and context excerpts
redact common secrets. Imported docs, session text, graph strings, and reverse
skill content are data, not executable policy. Optional reverse analysis is
static/read-only by default, never auto-installs tools, and never replaces
`vibe-security-scan` or `security-auditor`.

## Release Boundary

Baron 3.8 is released as one exact source revision with native assets,
checksums, installers, and a fresh Windows install smoke. README and
`releases/latest` must agree on `v3.8.0`; no older public version may remain the
normal install path after promotion. The release workflow also supports a
tag-triggered immutable promotion path so the public tag and source cannot drift.
