# SPEC-06.2 — DeepSeek Harness Adapter

- **Status:** Optional future design; deferred. Implementation is not approved or scheduled. Begin only after the owner explicitly requests SPEC-06 implementation.
- **Parent scope:** SPEC-06, cross-model continuation between Codex and DeepSeek Harness.
- **Depends on:** SPEC-06.1 task identity and resume contract.

## Goal

Add DeepSeek Harness (DSH) as an additive native host adapter over Baron Core so a user can open an already Codex-initialized checkout in DSH and work on the same Baron project and task.

The target is **DeepSeek Harness desktop**, which supports local workspaces and coding-agent tools. The regular DeepSeek chat application is not a local project adapter.

## Product decisions

- Baron owns project identity, memory, task state, routing, skills, gates, proof, trace, and recovery. DSH remains the coding host and executes the model/tool loop.
- DSH owns model/provider selection and API credentials. The user configures DeepSeek or another supported/custom provider in DSH. Baron does not add a second LLM client or create a project `.env` for provider keys.
- Add the host identifier `dsh` without changing the existing `codex` or `claude` identifiers.
- DSH is an additive adapter. Attaching it to a Codex project must preserve its Project ID, identity binding, Vault route, Codex projection, DSH user profile, third-party plugins, and user-owned project files.
- The selected user flow is sequential and additive: run `baron init --codex` as today, then run `baron init --dsh` once to attach DSH. A project may also be initialized with DSH first. Either order must preserve the same Project ID and Vault; DSH attachment must not reset identity or reinitialize the Vault.

## Adapter behavior

1. Discover and validate the existing `.baron/project.toml`, `.baron/local.toml`, Core runtime, and Vault binding before writing any host projection. Missing or ambiguous identity returns a repair/blocker path; it never creates a replacement Project ID or Vault.
2. Register a DSH-native adapter/plugin using supported DSH extension points. Use a structured local Baron bridge for Core operations. Do not rely only on a model-invoked MCP tool for mandatory lifecycle work.
3. At DSH session start and task entry, send a validated `PrepareRequestV1`-equivalent request to Core with host/session/request identity and the user's task. Consume the same semantic preparation contract used by Codex and Claude.
4. Project only the selected Core context, required workflow instructions, and next safe action into DSH. Do not copy all Core assets or duplicate Baron routing/memory logic inside the DSH plugin.
5. Record DSH session identity separately from task/project identity. Provider/model changes never create or rebind a Baron project.
6. Preserve unrelated DSH profile entries, plugins, hooks, skills, custom model definitions, and credentials when installing, updating, rolling back, or uninstalling the Baron adapter. Existing entries outside Baron-owned markers are not rewritten or removed.
7. If DSH is unavailable, the plugin API is incompatible, or the bridge cannot reach Core, report the degraded integration clearly. Do not claim a Baron-managed session or successful prepare when it did not occur.

## Model and credential boundary

DSH's model settings remain the source for provider ID, model ID, base URL, protocol, and credentials. Baron must not copy the API key into `.baron/project.toml`, `.baron/local.toml`, Vault Markdown, lifecycle logs, generated instructions, or test output. Switching provider/model is performed in DSH and starts or continues a DSH session against the same Baron task as defined by SPEC-06.1.

## Acceptance criteria

- `baron init --dsh` attaches DSH to an existing Codex project without changing Project ID, identity binding, Vault routing, or Codex-managed files.
- Repeating the DSH attach/update operation is idempotent and preserves DSH-owned configuration and other plugins.
- A DSH session opened in the same checkout resolves the same Baron project identity and receives a valid Core preparation packet.
- The user can configure a DeepSeek model or another provider/custom endpoint in DSH without Baron storing or logging the credential.
- The adapter works with a fresh DSH session after model selection; it does not claim that changing the default model rewrites the model identity of an already-running DSH session.
- DSH adapter installation and removal affect only files/config entries recorded as Baron-owned and produce recoverable diagnostics on collisions or changed baselines.
- The ordinary DeepSeek chat app is not advertised as a supported local coding host.

## Required verification cases

1. Add DSH to an existing Codex-initialized project and verify unchanged Project ID, Vault root, Codex assets, user instructions, and DSH profile entries.
2. Initialize a fresh disposable project with `baron init --codex` followed by `baron init --dsh`, then repeat in the reverse order; verify both resolve the same Core identity.
3. Change the selected DSH model/provider and verify Baron project/task identity remains unchanged.
4. Exercise missing DSH, unsupported DSH version/plugin contract, missing Core, and corrupted project identity; each returns an explicit, non-destructive failure.
5. Verify update/rollback/uninstall preserve DSH user settings, custom skills, unrelated plugins, and credentials.

## External host contract references

- DeepSeek Harness workspace and model setup: <https://deepseek-harness.github.io/deepseek-harness/en/guide/quickstart>
- DeepSeek Harness providers and custom model APIs: <https://deepseek-harness.github.io/deepseek-harness/en/guide/providers>
- DeepSeek Harness MCP extension: <https://deepseek-harness.github.io/deepseek-harness/en/guide/mcp-memory>
