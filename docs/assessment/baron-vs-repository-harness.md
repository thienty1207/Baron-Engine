# Baron vs repository-harness

This comparison is intentionally narrow and evidence-based.

`repository-harness` and Baron are solving the same broad problem: make a
software repository easier and safer for coding agents to work in. Baron is not
a clone of `repository-harness`; it is a larger memory and execution engine
that includes harness ideas as one layer.

## Short Version

`repository-harness` is simpler. Its README and repository shape are easier to
scan quickly.

In short: repository-harness is simpler; Baron is stronger for deeper memory,
proof, and scale.

Baron is stronger when the problem is long-running coding work across large
repositories, many projects sharing one Vault, strict proof, session replay,
skill routing, migration, and release safety.

## Comparison

| Area | repository-harness | Baron |
| --- | --- | --- |
| First impression | Cleaner, easier to explain | Deeper engine, more moving parts |
| Install story | Project-level install script plus bundled CLI | Native release installers with checksum verification and rollback |
| Agent surfaces | Claude Code, Codex, Cursor, other agents | Codex, Claude, Cursor-style/generic adapters with managed assets |
| Product context | Strong harness docs, feature intake, story packets, test matrix | Product Harness plus active plan, proof, trace, continuity, and Vault mirror |
| Memory | Repo-local durable docs | Vault-backed memory across projects |
| Multi-project safety | Not the main focus | memory firewall prefers current project and blocks weak cross-project recall |
| Session history | Harness traces and records | session replay/search with bounded current-project recall |
| Proof | Story verification and trace scoring | proof gates, trace quality, runtime execution evidence, capability checks |
| Skill routing | Harness operating model | Superpowers core, three quality gates, optional skills, custom asset quarantine |
| Migration | Harness install/merge/refresh | Agent Bootstrap migration, rollback, custom asset validation, quarantine |
| Release safety | Rust CLI release assets and checksum flow | Native release workflow, checksum manifests, installer lifecycle, certification checks |

## Where repository-harness is better today

- It is easier for a new reader to understand in one pass.
- It has public traction and a clearer story for “better repositories for
  agents.”
- Its harness docs are compact and marketable.

## Where Baron is better today

- Baron is built around a shared long-term Vault, not only repo-local docs.
- Baron treats cross-project memory as dangerous unless the match is explicit.
- Baron keeps tool availability separate from executed proof.
- Baron can search bounded prior sessions without dumping full histories.
- Baron validates and quarantines weak custom skills or agents.
- Baron has a broader release and migration safety model.

## Practical Recommendation

Use `repository-harness` when the goal is a lightweight, readable harness layer
inside one repository.

Use Baron when the goal is a durable AI work system for many repositories,
shared memory, old-codebase onboarding, proof-backed completion, and strict
anti-hallucination behavior.

## Honest Gap

Baron still needs better public packaging: release freshness, a crisp README,
demo walkthroughs, and public proof snapshots. Baron 3.1.0 exists to close that
public-trust gap without changing the core engine.
