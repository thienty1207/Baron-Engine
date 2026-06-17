---
name: performance-optimization
description: Use when optimizing latency, runtime speed, database or query performance, bundle size, loading, rendering, caching, throughput, memory, or resource use.
license: MIT-inspired guidance; Baron-native adaptation
---

# Performance Optimization

This bundled optional domain skill is for performance work. It does not replace Superpowers, tests, proof, trace scoring, or the optional `web-performance-auditor` when a web performance audit is specifically needed.

## Use When

- the task mentions slow, latency, performance, bundle, loading, rendering, cache, throughput, memory, CPU, database/query speed, or resource usage
- reviewing a change that could create N+1 queries, unbounded work, heavy frontend rendering, or inefficient network behavior
- interpreting provided performance artifacts

Do not use for cosmetic-only work unless it affects loading/rendering behavior.

## Baron Contract

- Never fabricate metrics. If no measurement exists, label findings as potential impact.
- Prefer the smallest measurement that can prove the claim.
- Check current platform focus before recommending web, backend, mobile, desktop, or CLI-specific fixes.
- Keep optimizations tied to user-facing or operational value; avoid micro-optimizations without evidence.

## Checklist

- What is the performance target or symptom?
- Is there measured evidence, or only static risk?
- What is the likely bottleneck: database, network, CPU, rendering, bundle, I/O, caching, or concurrency?
- What fix changes behavior least?
- What command, benchmark, smoke run, Lighthouse report, query plan, or profile proves improvement?

## Output Contract

- Measured or potential-impact status
- Bottleneck hypothesis
- Evidence source
- Recommended fix
- Verification command/artifact
- Remaining proof/trace gaps

## Attribution

Inspired by MIT-licensed performance ideas from `addyosmani/agent-skills`, rewritten as Baron-native optional guidance.
