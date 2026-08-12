# Small Review Workflow

Use this workflow for a repository or scope with at most 20 primary-language
files, 30 total source files, and a normal review window. The workflow is
deterministic, read-only, and bounded; it does not create a report directory or
temporary workspace unless the user explicitly requests a saved artifact.

## Inputs

Before entering this file, resolve and state:

- `repo_path`: the authorized repository root and its Baron project identity.
- `scope`: the requested files or directories, canonicalized and inside the root.
- `output_language`: Vietnamese by default, or English when requested.
- `primary_language`: the detected language, or `unknown` when not detected.
- `overlay`: the matching language rules, or none.
- `capabilities`: optional tools that are actually registered and executable.

Do not use undeclared variables, implicit shell state, or references to missing
numbered steps. If a path or language is unknown, record it as unknown and
continue with the generic rules where safe.

## S1 - Load rules lazily

1. Read the generic rule metadata needed for the detected surface. Load all 21
   canonical rules only for an explicitly requested full scan.
2. Load the selected language overlay only when it matches the primary language.
3. Record the rule IDs and overlay revision in the review notes. Never invent a
   new rule ID to make a finding fit.

## S2 - Review each file

For each in-scope text file, skip binaries, generated output, vendored code,
ignored secrets, and files outside the authorized root. For files over 5,000
lines, use bounded search-then-read windows rather than loading the whole file.

For each applicable rule:

1. Search for leads with `rg` or the equivalent read-only search tool.
2. Read the complete surrounding function, route, configuration, and relevant
   source-to-sink path before deciding that a lead is a finding.
3. Apply the data-flow classification in
   `../references/data-flow-classification.md`.
4. Record confirmed findings with file, line, canonical rule ID, severity,
   evidence, impact, safe abuse path, fix, and verification step.
5. Record skipped checks, false positives, assumptions, and unknowns separately.

## S3 - Cross-file checks

Perform only the cross-file checks relevant to the detected stack: ownership
and authorization around object references, global CSRF middleware, dependency
lockfiles, package typosquatting, upload validation, URL allowlists, and secret
reachability. Do not fetch or install anything unless the user explicitly asks
and Baron capability policy allows it.

## S4 - Verdict and output

Use the severity policy from `SKILL.md`. Print the report in the requested
language with findings, passed checks, skipped checks, unknowns, commands, and
residual risks. Do not claim a tool ran without its execution receipt. A saved
file is optional and must be requested by the user; if requested, write only to
the resolved path they approved and include its hash in the receipt.

## S5 - Verification checklist

- Every finding has a concrete file and line or an explicit configuration path.
- Every finding maps to one of the 21 canonical rule IDs.
- Counts by severity equal the number of findings.
- No secret, token, cookie, raw malicious sample, or sensitive personal data is
  copied into the report or Baron memory.
- The report states what was not reviewed and whether `security-auditor` still
  needs to perform the independent gate.
- The scan leaves the repository, `.gitignore`, Vault, and caches unchanged by
  default.
