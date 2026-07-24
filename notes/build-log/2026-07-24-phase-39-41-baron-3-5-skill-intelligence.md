# Baron 3.5 Skill Intelligence Build Log

Date: 2026-07-24
Release: Baron `3.5.0`
Branch: `codex/baron-3-4-to-3-6`

## Scope

Phases 39 through 41 strengthened existing Baron owners without adding another
workflow core, frontend owner, public command, or live runtime dependency.

## Completed Work

- Phase 39 deepened the local `frontend-design` skill with evidence-first brief,
  anti-template, responsive, and state-proof references.
- Phase 40 deepened the existing API/interface owner and added project-scoped
  Product Harness Domain Language that is bounded, evidence-backed, and
  withheld when repo and Vault copies diverge.
- Phase 41 re-audited routing and preservation across Codex, Claude, and generic
  adapters. It added a regression contract proving that reconcile preserves
  custom skills, custom agents, custom routing entries, and user-written Domain
  Language, while recreating only missing Baron-owned Domain Language files.

## RED To GREEN Evidence

The new reconcile preservation test first failed because missing repo/Vault
Domain Language documents were not restored. The reconcile path now calls the
existing create-if-missing platform, architecture, and Harness helpers before
recording its local checkpoint. User content remains untouched.

## Commands Passed Before The 3.5 Version Bump

```text
cargo test -p baron-core --test control_plane --test public_trust_docs
cargo test -p baron-cli --test adapter_cli --test release_smoke
cargo fmt --all -- --check
cargo test --workspace --all-targets --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
git diff --check
```

## Resume Point

Baron `3.5.0` is the source-certified baseline. Continue with Phase 42 only:
design and test Baron's provider-neutral, project-scoped code-graph contract.
Do not invoke or install any graph provider before that contract is proven.
