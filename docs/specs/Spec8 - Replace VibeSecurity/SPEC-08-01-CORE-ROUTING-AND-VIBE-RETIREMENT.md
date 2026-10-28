# SPEC-08-01 — Core Routing and Vibe Retirement

**Parent:** [SPEC-08-00](SPEC-08-00-REPLACEMENT-OVERVIEW.md)<br>
**Status:** Implemented and verified locally on Windows; pending owner review
**Depends on:** None

## Goal

Make Baron Core select one canonical optional AppSec skill named security-audit, remove the active Vibe security scan path, and upgrade existing projects without deleting user-owned or locally modified data.

## Requirements

### 1. Canonical skill identity and routing

- Use security-audit as the canonical Baron skill ID and runtime asset directory name.
- Route meaningful application code tasks through a bounded focused security pass. A meaningful task changes application source, build/dependency/CI configuration, security policy, or a data-flow boundary; pure documentation or cosmetic work does not trigger a broad audit unless it touches a security-sensitive surface.
- Run a pre-implementation risk check against the planned scope and existing source, then recheck the actual changed files and connected trust boundaries before task completion. The user should not need to say “security audit” for ordinary application implementation.
- Escalate to a broader audit when the user asks for one or when Core identifies a high-risk security boundary. Core chooses scope from intent, changed paths, repository evidence, risk, and current Task State.
- Emit the selected scope, detected language set, audit mode, required validation, and any capability blocker in the normal PrepareRequestV1 / PreparePacketV1 path.
- If a finding crosses the initial focused scope but remains in the same repository and connected security boundary, expand the audit and remediation scope before deciding whether to fix it. Otherwise record the unresolved concern and coverage gap; never imply that the focused scan covered it.
- Keep the skill optional in the sense that only route-selected security guidance is loaded; do not preload the full audit corpus into every task.
- Preserve the existing requirement that security-sensitive or high-risk work receives the mandatory security-auditor gate and other Core-required quality gates.
- Keep security-audit and security-auditor distinct: the former is the audit methodology; the latter is Baron’s independent quality gate.

### 2. Retire all active Vibe routing and instructions

The implementation inventory must include, at minimum, the touchpoints observed during this targeted inspection:

- crates/baron-core/src/control_plane.rs: selection, built-in metadata, trigger reason, and supported-skill allowlist.
- crates/baron-core/src/asset_lifecycle.rs, migration.rs, and platform.rs: managed-asset, compatibility, and platform registries.
- crates/baron-adapters/src/install.rs: generated Codex/Claude indexes and skill contracts.
- assets/core/skills/vibe-security-scan/**: replace the active asset tree with the new Baron-owned audit assets.
- Related API/interface and mobile guidance that refers to Vibe as the AppSec owner.
- Core, CLI, adapter, survey, context, lifecycle, and routing tests/fixtures that expect the old name.
- Current status/architecture docs and benchmark data where claims are presented as current product behavior.

Regenerate the reference inventory during implementation; this list is not a complete source scan.

### 3. Safe update of existing installations

- A new install publishes only the security-audit managed skill.
- An update may retire the old .baron/core/skills/vibe-security-scan asset only when the existing managed baseline proves that the file is Baron-owned and unmodified.
- If the old asset was edited, ownership is ambiguous, or a transaction cannot complete, preserve it and produce a recoverable migration receipt or reviewable proposal. Do not silently delete it.
- Never remove similarly named files under user-owned .codex/skills, .claude/skills, custom skill directories, or third-party hook locations unless a verified Baron ownership manifest proves the exact path is managed.
- The change must use existing transactional update/migration facilities and preserve rollback behavior.
- Re-running update must be idempotent; interruption and retry must not leave both skills active or remove both.

### 4. License and historical references

Retain the upstream MIT license and copyright notice for any copied or adapted Cloudflare material. Keep attribution in the bundle NOTICE/LICENSE files. If only methodology is independently re-expressed, document the provenance without copying protected text.

Historical references

- The active route, runtime asset, generated instructions, and current product tests must not select or describe Vibe as the AppSec owner.
- Historical plans, release evidence, and provenance may mention the retired skill if the reference is explicitly historical and does not affect runtime.
- The retired-adapter/asset gate must use an explicit historical allowlist rather than disabling the gate.
- Permit the legacy skill ID only in exact migration/compatibility code and tests that prove safe retirement. Those references must not be used for active routing, runtime selection, or generated current-product instructions.

## Acceptance tests

- Security-sensitive requests select security-audit and retain the three mandatory Core gates.
- Ordinary non-security documentation work does not load the audit corpus.
- Existing app implementation requests in each supported language receive a focused check.
- No security task selects or reports the Vibe skill.
- New install and update publish the canonical new skill and generated indexes for Codex and Claude.
- Update fixtures cover an unchanged legacy asset, a modified legacy asset, a missing baseline, interrupted migration, rollback, and retry.
- Modified user-owned skill folders, unknown config keys, and third-party hooks remain byte-for-byte intact.
- Tracked active runtime, route, and generated guidance references are absent; exact legacy migration compatibility tests are allowlisted and prove the old skill cannot be selected.
