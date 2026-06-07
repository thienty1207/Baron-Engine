---
name: vibe-security-scan
description: Use when Baron routes defensive security review for auth, API, secrets, .env, Supabase/RLS, uploads, payments, dependencies, CORS, JWT, rate limits, access control, tenant, admin, or permission work.
---

# Vibe Security Scan Blueprint

Bundled optional defensive security skill.

This skill should be lazy-loaded only when a task matches security-sensitive
work. It must stay defensive-only. It complements the `security-auditor` core
agent but does not replace Superpowers or Baron proof/trace gates.
