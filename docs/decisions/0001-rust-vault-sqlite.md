# 0001 - Rust Engine, Vault Source, SQLite Accelerator

Status: accepted
Date: 2026-06-08

## Decision

Baron will use Rust as the primary engine language.

Markdown in the vault is the durable source of truth. SQLite/cache/index files
are accelerators only.

## Rationale

Rust gives Baron a fast, portable, dependency-light engine that can handle large
repos and large vaults. Markdown keeps memory inspectable by humans and agents.
SQLite gives query speed without taking ownership of truth.

## Consequences

- Baron can ship as a standalone binary.
- Baron can support Windows-first workflows.
- The engine must be careful to rebuild indexes from Markdown.
- Database corruption must not destroy memory.
