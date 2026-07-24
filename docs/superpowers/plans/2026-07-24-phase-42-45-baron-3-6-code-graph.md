# Baron 3.6 Optional Code Graph Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an optional, local, project-scoped Graphify code-map provider that improves architecture and dependency context without replacing Baron Survey, Vault memory, source verification, or agent instructions.

**Architecture:** Introduce a Baron-owned code-graph provider boundary and a Graphify `0.9.25` adapter that runs only local `--code-only` extraction and bounded JSON queries. Keep graph state under the current project's rebuildable `.baron/cache`, load only a few task-relevant hits, label inferred edges, and fall back to Survey Engine on every absence or failure.

**Tech Stack:** Rust 2021, Serde JSON, SHA-256, `ignore`, existing Baron capability/context/config modules, optional external Graphify CLI, `wait-timeout` for bounded subprocesses.

**Prerequisite:** Baron `3.5.0` Phase 41 certification is complete and committed.

---

### Task 1: Phase 42 Code-Graph Provider Contract

**Files:**
- Create: `crates/baron-core/src/code_graph.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/Cargo.toml`
- Modify: `Cargo.lock`
- Create: `crates/baron-core/tests/code_graph.rs`
- Modify: `crates/baron-core/tests/capability.rs`
- Modify: `docs/architecture/CAPABILITY_REGISTRY.md`

- [ ] **Step 1: Write RED provider-model tests**

Define the tested public model:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphConfidence {
    Extracted,
    Inferred,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGraphHit {
    pub node_id: String,
    pub label: String,
    pub source_file: Option<String>,
    pub relation: Option<String>,
    pub confidence: GraphConfidence,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeGraphState {
    pub schema_version: u32,
    pub provider: String,
    pub provider_version: String,
    pub project_id: String,
    pub repo_root: String,
    pub source_fingerprint: String,
    pub graph_sha256: String,
    pub graph_size_bytes: u64,
    pub built_at: String,
    pub freshness: GraphFreshness,
    pub diagnostics: Vec<String>,
}

pub trait CodeGraphProvider {
    fn probe(&self, repo_root: &Path) -> Result<ProviderProbe>;
    fn refresh(&self, repo_root: &Path, cache_root: &Path) -> Result<CodeGraphState>;
    fn query(
        &self,
        repo_root: &Path,
        cache_root: &Path,
        question: &str,
        limits: QueryLimits,
    ) -> Result<Vec<CodeGraphHit>>;
}
```

Tests must prove serialization names are stable, inferred/extracted confidence
stays explicit, unsafe relative source paths are rejected, duplicate hits are
deduplicated, and task output is bounded by hit count and character budget.

- [ ] **Step 2: Add RED project identity and cache tests**

Require this layout:

```text
.baron/cache/code-graph/
  state.json
  graphify/<source-fingerprint>/graphify-out/graph.json
```

Tests must prove:

- canonical cache paths stay under the current repository
- `..`, absolute external paths, symlink/junction escape, and foreign project
  identity are refused
- two repositories with the same directory name receive different state
- cache removal loses no Vault Markdown or project source
- no graph path is placed in the Vault

- [ ] **Step 3: Add RED source-fingerprint tests**

Implement the tested signature:

```rust
pub fn compute_code_source_fingerprint(repo_root: impl AsRef<Path>) -> Result<String>;
```

The fingerprint hashes sorted repository-relative source paths, byte sizes, and
modified timestamps through the gitignore-aware `ignore` walker. It skips
`.git`, `.baron`, build output, dependency directories, Graphify output, and
Vault paths. It has no fixed source-file count.

Tests must prove content edits with a changed size or modification time, added
files, removed files, and renamed files invalidate freshness. Heavy ignored
folders must not affect the fingerprint.

- [ ] **Step 4: Run RED tests**

Run:

```powershell
cargo test -p baron-core --test code_graph
```

Expected: FAIL because `code_graph` does not exist.

- [ ] **Step 5: Implement the provider-neutral core**

Implement path validation, state load/write, graph checksum, freshness
comparison, hit normalization, deduplication, and bounded rendering. State
writes use a sibling temporary file plus atomic rename.

Add `wait-timeout = "0.2"` to `baron-core` for provider subprocesses used in the
next phase, but do not invoke Graphify in this task.

- [ ] **Step 6: Connect the optional `code-map` capability**

Represent Graphify as an optional CLI provider:

```rust
CapabilityProvider {
    name: "graphify-local".to_string(),
    capability: "code-map".to_string(),
    kind: ProviderKind::Cli,
    requirement: Requirement::Optional,
    command: Some("graphify".to_string()),
    scan_target: None,
    adapters: Vec::new(),
    description: "Optional local project-scoped code map".to_string(),
}
```

Provider absence must remain a warning/fallback, never a proof-gate failure.
Registration must not overwrite a custom `code-map` provider.

- [ ] **Step 7: Run focused tests and commit Phase 42**

Run:

```powershell
cargo test -p baron-core --test code_graph
cargo test -p baron-core --test capability
git diff --check
```

Then update status/build log and commit:

```powershell
git add Cargo.lock crates/baron-core docs/architecture/CAPABILITY_REGISTRY.md docs/BARON_STATUS.md docs/BARON_STATUS.json notes/build-log
git commit -m "feat: add Baron code graph provider contract"
```

### Task 2: Phase 43 Graphify Local Code-Only Adapter

**Files:**
- Create: `crates/baron-core/src/graphify.rs`
- Modify: `crates/baron-core/src/lib.rs`
- Modify: `crates/baron-core/src/code_graph.rs`
- Create: `crates/baron-core/tests/graphify_provider.rs`
- Create: `crates/baron-core/tests/fixtures/fake-graphify.ps1`
- Modify: `crates/baron-core/tests/capability.rs`
- Modify: `docs/architecture/ARCHITECTURE.md`

- [ ] **Step 1: Write a deterministic fake provider**

The PowerShell fixture accepts:

```text
graphify --version
graphify extract <repo> --code-only --out <cache> --no-cluster
graphify query <question> --graph <graph> --json --budget <n>
```

It returns `graphify 0.9.25`, writes a small deterministic graph for extraction,
and emits JSON query hits. Environment variables select failure modes:

```text
FAKE_GRAPHIFY_MODE=timeout
FAKE_GRAPHIFY_MODE=malformed
FAKE_GRAPHIFY_MODE=oversized
FAKE_GRAPHIFY_MODE=wrong-version
FAKE_GRAPHIFY_MODE=nonzero
```

Tests must never call the internet or a real Graphify installation.

- [ ] **Step 2: Add RED compatibility tests**

The provider accepts only the audited initial contract:

```rust
const SUPPORTED_GRAPHIFY_VERSION: &str = "0.9.25";
const AUDITED_GRAPHIFY_REVISION: &str =
    "2fa6cd3d5548577f8c5f591b713f0bf80c1af183";
```

Tests prove missing binaries and any other version produce an optional
diagnostic and do not execute extraction.

- [ ] **Step 3: Add RED command-safety tests**

Capture every fake-provider invocation and assert Baron calls only:

```text
--version
extract <repo> --code-only --out <project-cache> --no-cluster
query <question> --graph <project-graph> --json --budget <bounded>
```

Assert no invocation contains `install`, `hook`, `global`, `save-result`,
`reflect`, a Vault path, API key, or network URL.

- [ ] **Step 4: Add RED timeout and output-cap tests**

Use these initial hard limits:

```rust
pub const GRAPHIFY_PROBE_TIMEOUT: Duration = Duration::from_secs(3);
pub const GRAPHIFY_REFRESH_TIMEOUT: Duration = Duration::from_secs(120);
pub const GRAPHIFY_QUERY_TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_GRAPH_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_PROVIDER_STDOUT_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_PROVIDER_STDERR_BYTES: u64 = 256 * 1024;
pub const MAX_QUERY_HITS: usize = 8;
pub const MAX_QUERY_CHARS: usize = 2_400;
```

Timeout, non-zero exit, malformed JSON, oversized output, and oversized graph
must preserve the last known-good state and return a fallback diagnostic.

- [ ] **Step 5: Run RED tests**

Run:

```powershell
cargo test -p baron-core --test graphify_provider
```

Expected: FAIL because the Graphify adapter does not exist.

- [ ] **Step 6: Implement bounded subprocess execution**

Spawn with piped input disabled and stdout/stderr redirected to Baron-owned
temporary files. Use `wait_timeout`; kill and wait on timeout. Check file sizes
before reading. Redact environment values from diagnostics.

For Baron-owned calls set:

```text
GRAPHIFY_QUERY_LOG_DISABLE=1
```

Use `--code-only` so Graphify cannot select a semantic backend. Do not forward
provider API keys in command arguments.

- [ ] **Step 7: Implement atomic graph refresh**

Refresh into:

```text
.baron/cache/code-graph/.staging-<process-id>/
```

Validate the resulting graph path, size, JSON top-level shape, project identity,
and checksum. Move it into the source-fingerprint directory only after all
checks pass, then atomically replace `state.json`. A failed refresh deletes only
its staging directory.

- [ ] **Step 8: Parse and normalize bounded queries**

Parse only known JSON fields. Reject absolute/escaping source paths. Preserve
Graphify `EXTRACTED` versus `INFERRED` confidence; unknown confidence becomes
`INFERRED`, never trusted. Sort stable results by provider score, confidence,
and source path, then apply hit/character limits.

- [ ] **Step 9: Run focused tests and commit Phase 43**

Run:

```powershell
cargo test -p baron-core --test graphify_provider
cargo test -p baron-core --test code_graph
cargo test -p baron-core --test capability
```

Update status/build log and commit:

```powershell
git add crates/baron-core docs/architecture/ARCHITECTURE.md docs/BARON_STATUS.md docs/BARON_STATUS.json notes/build-log
git commit -m "feat: add local Graphify code map adapter"
```

### Task 3: Phase 44 Automatic Bounded Context And Source Verification

**Files:**
- Modify: `crates/baron-cli/src/main.rs`
- Modify: `crates/baron-core/src/context.rs`
- Modify: `crates/baron-core/src/code_graph.rs`
- Modify: `crates/baron-adapters/src/install.rs`
- Modify: `crates/baron-adapters/tests/adapter_lifecycle.rs`
- Modify: `crates/baron-core/tests/context_compiler.rs`
- Create: `crates/baron-cli/tests/code_map_cli.rs`
- Modify: `crates/baron-cli/tests/automation_cli.rs`
- Modify: `docs/architecture/CONTEXT_COMPILER.md`

- [ ] **Step 1: Add hidden CLI RED tests**

Add hidden AI-facing commands:

```text
baron automation code-map status [repo-path]
baron automation code-map refresh [repo-path]
baron automation code-map query "<question>" [repo-path]
```

Tests must prove they stay absent from normal top-level help and return stable
JSON with `--json` where applicable.

- [ ] **Step 2: Add task-routing RED tests**

Code-map routing matches architecture, dependency, impact, ownership,
entrypoint, call-flow, and cross-module questions. It skips copy changes,
README edits, simple config text, and tasks with no code-navigation need.

Context behavior:

- fresh compatible graph: load at most eight hits and 2,400 characters
- missing/stale/incompatible/failed graph: show one short diagnostic and use
  existing Survey context
- no task query: do not dump graph content
- inferred hit: include `verify against source before proof`

- [ ] **Step 3: Add source-verification RED tests**

Provide:

```rust
pub fn verify_graph_hit_source(
    repo_root: impl AsRef<Path>,
    hit: &CodeGraphHit,
) -> Result<SourceVerification>;
```

Verification checks that the source file exists inside the current repository
and that an extracted symbol/relationship can be found in current source
evidence. An inferred edge can guide reading but cannot be promoted to verified
proof by this function.

Tests prove stale/deleted/escaping paths and foreign-project hits are rejected.

- [ ] **Step 4: Run RED tests**

Run:

```powershell
cargo test -p baron-cli --test code_map_cli
cargo test -p baron-core --test context_compiler
```

Expected: FAIL at the missing commands and context section.

- [ ] **Step 5: Implement AI automation without startup blocking**

Generated Codex, Claude, and generic instructions say:

1. run normal Baron context first
2. when context reports a useful missing/stale code map for a matching task,
   silently run hidden `automation code-map refresh`
3. query the refreshed map for that task
4. verify selected source files before edits or proof

`context` itself never waits for a long graph build. This keeps fresh sessions
bounded on large repositories. Normal users receive no new primary command.

- [ ] **Step 6: Render the bounded context section**

Add:

```markdown
## Optional Code Map

- Provider: graphify-local 0.9.25
- Freshness: fresh
- Result: ...
- Confidence: extracted|inferred
- Source: path
```

Do not include raw graph JSON, global graph state, query logs, or a full
architecture report.

- [ ] **Step 7: Preserve Baron ownership in generated instructions**

Adapter tests must assert:

- Superpowers remains workflow core
- Baron Context Compiler chooses whether graph context is loaded
- Graphify does not own hooks or modify `AGENTS.md`, `CLAUDE.md`, or agent config
- Survey Engine is the fallback
- source verification is mandatory before proof

- [ ] **Step 8: Run focused tests and commit Phase 44**

Run:

```powershell
cargo test -p baron-cli --test code_map_cli
cargo test -p baron-cli --test automation_cli
cargo test -p baron-core --test context_compiler
cargo test -p baron-adapters --test adapter_lifecycle
```

Update status/build log and commit:

```powershell
git add crates/baron-cli crates/baron-core crates/baron-adapters docs/architecture docs/BARON_STATUS.md docs/BARON_STATUS.json notes/build-log
git commit -m "feat: load bounded optional code graph context"
```

### Task 4: Phase 45 Isolation, Scale, And Baron 3.6 Certification

**Files:**
- Modify: `crates/baron-cli/tests/release_smoke.rs`
- Modify: `crates/baron-core/tests/certification.rs`
- Modify: `crates/baron-core/tests/public_trust_docs.rs`
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `README.md`
- Modify: `docs/BARON_STATUS.md`
- Modify: `docs/BARON_STATUS.json`
- Modify: `docs/superpowers/plans/CURRENT.md`
- Modify: `notes/build-log/CURRENT.md`
- Create: `docs/assessment/baron-3.6.0-code-graph-certification.md`

- [ ] **Step 1: Add RED end-to-end degradation tests**

Prove each case returns useful Survey context and does not fail the task:

- Graphify absent
- unsupported Graphify version
- provider timeout or non-zero exit
- malformed or oversized query output
- malformed or oversized graph
- stale graph after source change
- interrupted refresh with a last known-good graph

- [ ] **Step 2: Add RED project-isolation tests**

Create two same-name repositories sharing one Vault. Build different fake
graphs containing overlapping auth terms. Assert:

- each `.baron/cache/code-graph/state.json` contains its own project identity
- queries never return the other repository's source path
- neither graph appears in Vault memory indexing or recall
- deleting one cache does not modify the other project or Vault

- [ ] **Step 3: Add RED old/large repository tests**

Use the existing large-repository fixture pattern with more than 6,000 entries,
multiple languages, ignored dependency/build folders, and legacy architecture.
Prove fingerprinting has no fixed file cap, context stays bounded, and graph
failure still leaves Survey orientation available.

- [ ] **Step 4: Add RED no-hook/no-instruction-mutation smoke**

Snapshot target repo files before refresh/query. After execution assert Baron
did not add or modify:

```text
.git/hooks/
AGENTS.md
CLAUDE.md
.codex/hooks.json
.claude/settings.json
graphify-out/
~/.graphify/
```

Only `.baron/cache/code-graph/` may change.

- [ ] **Step 5: Run the complete behavior and release gate**

Run:

```powershell
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --locked -p baron-cli
git diff --check
```

Run a release-binary smoke for Codex, Claude, and generic projects with:

- no Graphify installed
- the deterministic compatible fake provider
- an incompatible fake provider
- a large old repository
- two projects sharing one Vault

- [ ] **Step 6: Write the certification report**

Record:

- Graphify audited version/revision and license boundary
- exact commands Baron permits
- proof no installer, hook, global graph, memory layer, or semantic backend ran
- command timeout/output limits
- context character/hit limits
- project-isolation and stale-cache evidence
- all failure/fallback outcomes
- confirmation Graphify remains optional

- [ ] **Step 7: Bump to `3.6.0` only after certification**

Update workspace crates, lockfile, version assertions, status, README, release
docs, current plan, and certification. Re-run the complete gate and verify:

```powershell
target\release\baron.exe --version
```

Expected: `baron 3.6.0`.

- [ ] **Step 8: Commit and push verified 3.6 source**

```powershell
git add Cargo.toml Cargo.lock crates README.md docs notes
git commit -m "release: certify Baron 3.6 optional code graph"
git push origin main
```

Do not create a tag or GitHub Release unless the user explicitly authorizes
release promotion.

## Plan Self-Review

- Graphify is optional and exact-version gated.
- Extraction is local code-only and cannot choose a semantic backend.
- No Graphify installer, hook, global graph, or work-memory command is called.
- Graph cache is rebuildable, project-local, bounded, and outside the Vault.
- Inferred edges never become proof without source verification.
- Context never blocks on a long graph refresh.
- Survey Engine remains the fallback in every failure case.
- No normal user command is added.
- Version `3.6.0` is gated by fresh full-suite evidence.
