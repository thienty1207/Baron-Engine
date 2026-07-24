# Baron 3.6 Optional Code Map Certification

Status: source-certified on `2026-07-24`; Git tag and GitHub Release not created

Baron 3.6 adds an optional local code map for navigation in large repositories.
It does not turn an external graph program into a Baron workflow, memory, or
instruction owner. Baron Survey remains the reliable starting point whether a
code map exists or not.

## Provider Boundary

| Boundary | Certified behavior |
| --- | --- |
| Accepted provider | `graphify-local` only, exactly version `0.9.25` at audited revision `2fa6cd3d5548577f8c5f591b713f0bf80c1af183`. |
| License and distribution | Baron does not vendor, install, bundle, or redistribute Graphify. A separately installed optional executable remains under its own upstream license boundary. |
| Allowed commands | `graphify --version`; local code-only extraction; bounded local JSON query. |
| Rejected behavior | installer, hook, instruction writer, global graph, Vault memory writer, semantic backend, remote API key forwarding, query log, or workflow ownership. |
| Durable truth | Vault Markdown and current repository source remain the truth. The code map is a rebuildable project-local cache. |

The only graph extraction shape Baron permits is equivalent to local
**code-only extraction** with an explicit project-local output directory. A
query is accepted only as bounded JSON against that same current-project graph.

## Safety Limits

- Probe timeout: 3 seconds; refresh timeout: 120 seconds; query timeout: 10
  seconds.
- Provider stdout is capped at 2 MiB and stderr at 256 KiB.
- A graph is capped at 256 MiB. Context loads at most eight hits and 2,400
  graph characters.
- Cache location is only `.baron/cache/code-graph/` in the current project.
  It is identity-bound, checksum-checked, rebuildable, and outside the Vault.
- Extracted hints need current source evidence. Inferred hints stay advisory
  and cannot be proof, a trace conclusion, or durable memory by themselves.

## Failure And Fallback Evidence

`graphify_provider.rs` verifies a missing, incompatible, non-zero, timed-out,
malformed, or oversized provider result keeps the last known good graph intact
when one exists. `compile_context_for_task` then keeps a useful Project Atlas
and says that Survey remains active instead of failing the task.

`graphify_refresh_and_query_only_change_the_project_local_cache` snapshots
`.git/hooks`, `AGENTS.md`, `CLAUDE.md`, `.codex/hooks.json`,
`.claude/settings.json`, root `graphify-out/`, and a controlled home graph
location. The test proves refresh/query leave them untouched; only the
project-local Baron cache changes.

## Isolation And Scale Evidence

- `same_name_projects_keep_graphs_local_and_out_of_vault_memory` creates two
  same-name projects sharing one Vault. Their identities, cache roots, source
  hits, and graph state stay separate. Graph labels and paths do not enter Vault
  indexing or recall, and deleting one cache leaves the other cache and Vault
  Markdown intact.
- `large_repository_survey_and_context_remain_bounded` builds a mixed-language
  legacy fixture with more than 6,100 source entries plus ignored dependency
  and build folders. A change after entry 6,000 changes the fingerprint, while
  survey/context output remains bounded and missing Graphify leaves Survey
  orientation available.
- `certification_proves_scale_isolation_and_cache_recovery` adds the
  `optional-code-map-boundary` certification gate alongside existing shared
  Vault and cache-rebuild checks.

## Verification Gate

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo test -p baron-core --test graphify_provider --test code_graph_isolation --test certification` | passed |
| `cargo test -p baron-cli --test release_smoke` | passed |
| `cargo test --workspace --all-targets --no-fail-fast` | passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | passed |
| `cargo build --release --locked -p baron-cli` | passed; release binary reported `baron 3.6.0` |
| `git diff --check` | passed |

## Release Boundary

Source `3.6.0` is not a Git tag or GitHub Release. A release promotion remains
a human-authorized action after the verified source commit is pushed.
