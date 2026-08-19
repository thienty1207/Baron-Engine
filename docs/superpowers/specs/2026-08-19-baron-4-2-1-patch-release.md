# Baron 4.2.1 Patch Release Decision

## Decision

Publish Baron `4.2.1` as a packaging and release-truth patch. The original
`v4.2.0` tag was created before the Reasonix adapter commits, so its native
binary cannot expose the Reasonix command surface even though the current
source and README can. Baron `4.2.1` packages the already-reviewed adapter in
the public binary without changing the intelligence engine or memory model.

## In scope

- bump the workspace and certification release identity from `4.2.0` to
  `4.2.1`;
- keep `baron init --reasonix`, `baron context --reasonix`, and the root
  `baron --reasonix` shortcut in the released binary;
- publish native archives, raw update candidates, checksums, manifest, and
  installers through the existing immutable release workflow;
- synchronize README, release guide, changelog, status, JSON, architecture,
  and build-log truth;
- verify the new Windows binary hash so a narrowly scoped local WDAC policy can
  be regenerated after installation.

## Out of scope

- no Baron 4.3 release;
- no intelligence, memory, Wiki, CodeGraph, Vault, or fallback redesign;
- no change to user project files or Vault data;
- no broad Windows policy relaxation or path-wide allow rule.

## Release gate

The release is complete only when the exact `4.2.1` source commit is on
`origin/main`, the native matrix and release workflow pass, the public
`releases/latest` installer reports `baron 4.2.1`, and that binary accepts
`baron --reasonix` and `baron init --reasonix`. A new binary hash must be
recorded separately from the old 4.2.0 WDAC exception.
