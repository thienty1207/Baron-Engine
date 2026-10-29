# SPEC-08-02 — Seven-Language Security Coverage

**Parent:** [SPEC-08-00](SPEC-08-00-REPLACEMENT-OVERVIEW.md)<br>
**Status:** Implementation and static contracts are locally verified on isolated branch `codex/spec-08-02-seven-language-coverage` (2026-10-28); SPEC-08-03 now supplies workflow/schema validation, but classifier behavior against fixtures remains unavailable until SPEC-08-05 enables safe execution; not committed or pushed
**Depends on:** SPEC-08-01 and SPEC-08-05

## Goal

Make the new Baron security-audit route useful in the languages the user expects to build products with, rather than treating a generic checklist as proof of language coverage.

## Required languages

| Language | Minimum repository evidence |
|---|---|
| Python | .py, pyproject.toml, requirements/lock files, or project manifests |
| C++ | .cc, .cpp, .cxx, .h, .hh, .hpp, .hxx, and build manifests such as CMake |
| Java | .java, Maven/Gradle files, or JVM build manifests |
| Rust | .rs, Cargo.toml, and Cargo.lock |
| Go | .go, go.mod, and go.sum |
| TypeScript | .ts, .tsx, .mts, .cts, package.json, and lockfiles |
| C# | .cs, .csproj, .sln, and NuGet manifests |

Detection must account for monorepos and mixed-language projects. One detected language must never suppress another. Generated, vendored, fixture, and build-output directories are excluded only by an explicit recorded scope rule.

Manifest and source evidence is read through Core's safe regular-file opener. Per-file and aggregate byte limits apply while streaming from the same opened handle; an unreadable or over-limit path remains an explicit coverage gap. Source-based framework signals are derived from parsed import targets after language-aware comment removal, not from comments or similar identifiers.

C/C++ header files such as .h are classified from build manifests, compilation databases, and consuming translation units. If the repository does not establish whether a shared header is compiled as C or C++, record the ambiguity and cover each relevant language interpretation without double-counting the file.

## Coverage contract

For every in-scope language, the audit records:

- detected language and confidence/evidence;
- framework and build-system signals when present;
- source paths and lifecycle surfaces included in scope;
- language/domain rules loaded;
- attack classes reviewed;
- checks actually run and their evidence;
- unsupported patterns, unreadable paths, missing tools, and other gaps.

A language is not “covered” merely because its extension was detected or its checklist was loaded.

## Minimum shared attack classes

Every required language needs checks for:

- trust boundaries, input validation, injection, and unsafe output rendering;
- authentication, authorization, ownership/tenant isolation, and object-level access;
- secrets, sessions, tokens, cookies, JWT handling, and cryptographic misuse;
- SQL/query construction, deserialization, serialization, and unsafe parsing;
- file paths, uploads, archive extraction, external URL fetching, and SSRF;
- command/process execution, logging/error disclosure, CORS/CSRF where relevant;
- rate limits, resource exhaustion, race conditions, dependency and build supply-chain risk;
- deployment/configuration mistakes that materially weaken a code-enforced control.

Language-specific rules must be added for idioms not captured by the shared list, including C++ memory/lifetime/bounds issues, Rust unsafe and FFI boundaries, Java and C# framework guards, Go concurrency and process boundaries, Python dynamic loading/deserialization, and TypeScript browser/Node trust boundaries.

## Measurable fixture baseline

Organize the shared coverage into these testable families: input/injection/output; authentication/authorization/ownership; secrets/session/cryptography; file/path/upload/outbound-network; serialization/process execution; concurrency/resource exhaustion; and dependency/build/deployment configuration.

For every language and every family marked applicable, provide at least one vulnerable positive fixture and one safe-pattern/negative fixture. A family may be marked not applicable only when its absence is supported by the fixture or repository model; do not leave an empty matrix cell. Add a language-specific positive/safe pair for each native-only rule family, including C++ memory-safety and Rust unsafe/FFI cases. The matrix is a minimum regression baseline, not a completeness guarantee.

Framework-specific coverage should be loaded from repository evidence rather than guessed. Initial fixtures should include representative web/backend stacks where they are supported by source evidence, such as Django/FastAPI/Flask, Spring/Jakarta, ASP.NET Core, common Go HTTP handlers, Rust web/data crates, and Node/TypeScript web frameworks. This list is a starting matrix, not a claim that every framework is already supported. The detector must preserve `framework_detection_catalog_non_exhaustive_requires_validation` for each in-scope detected language; a missing known signal is not evidence that no framework is present. Unknown framework or construct evidence remains `needs_validation` rather than falling back silently to language-only rules.

## Authoring and maintenance rules

- Keep a shared taxonomy for finding categories and severity; language rules add source-to-sink detail without inventing conflicting severity policy.
- Each rule states the risky source/operation, the protected boundary, the safe code pattern, common false positives, and the evidence needed to confirm a finding.
- Rules are lazy-loaded by detected language and relevant features. Do not read every language corpus for a single-language task.
- Rules and fixtures are local Baron assets so the normal workflow is usable offline.
- Optional ecosystem scanners may add evidence only when Capability Registry says they actually executed. They are not prerequisites and are never silently installed.
- If a framework or construct has no reliable rule, retain the gap as needs_validation; do not silently map it to a weaker language pack.

## Acceptance tests

For each of the seven languages:

1. A detector fixture routes to the correct language pack.
2. The fixture matrix covers every applicable shared baseline family; every rule classified Critical or High by the finding severity policy has a vulnerable positive fixture and a safe-pattern/negative fixture.
3. Negative and safe-pattern fixtures carry an explicit `must_not_be_confirmed_from_keyword_alone` expected outcome. SPEC-08-02 validates this as fixture metadata only; executable classifier behavior must be verified in SPEC-08-03 before claiming this acceptance item is behaviorally proven.
4. Mixed-language fixtures produce separate coverage ledger units.
5. Generated/vendor paths are excluded only when the scope records the reason.
6. Unsupported language/framework evidence remains visible as a coverage gap.
7. The route and report use the same canonical finding taxonomy and do not claim full coverage from detection alone.

Each vulnerable/safe fixture pair includes an `expected_outcome`: a positive is a candidate requiring source validation, and a safe case must not be confirmed from keyword presence alone. The asset-contract test validates those labels and fixture structure as static data. SPEC-08-03 validates workflow and finding schemas but does not execute a classifier or fixtures. These labels do not establish that a current classifier behaves correctly or that real applications are vulnerability-free; classifier behavior remains unverified until SPEC-08-05 enables safe execution.
