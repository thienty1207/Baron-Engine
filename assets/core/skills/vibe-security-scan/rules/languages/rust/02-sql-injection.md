---
id: SQL-INJECTION
severity_max: CRITICAL
applies_to: rust
---

# Rust SQL Injection

## Intent

Rust code can still build unsafe SQL when L1 user input reaches `format!`, `push_str`, or string concatenation before a raw query sink.

## Search patterns

Use these as leads, then read surrounding code:

```text
sqlx::query\s*\(\s*&?format!
sqlx::query_as\s*\(\s*&?format!
diesel::sql_query\s*\(\s*format!
Statement::from_string
raw_sql|query_raw|execute_raw
format!\s*\(\s*".*(SELECT|UPDATE|DELETE|INSERT)
```

## Flag when

- route/query/body/path/header input reaches SQL text construction
- dynamic `ORDER BY`, table, or column names are not allowlisted
- raw SQL is executed without parameter binding

## Do not flag when

- `sqlx::query!("SELECT ... WHERE id = $1", id)` or bind-style APIs are used correctly
- dynamic identifiers come from a closed enum or constant allowlist
- ORM filters bind values instead of interpolating SQL strings

## Fix recommendation

Use SQLx macros or `.bind()`, Diesel query builders, or explicit allowlists for identifiers that cannot be parameterized.
