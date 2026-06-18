---
name: performance-optimization
description: Use when optimizing latency, runtime speed, database or query performance, bundle size, loading, rendering, caching, throughput, memory, or resource use.
license: MIT-compatible Baron-owned local guidance; attribution lives in NOTICE.md
---

# Performance Optimization

This bundled optional domain skill is for performance work. It does not replace Superpowers, tests, proof, trace scoring, or the optional `web-performance-auditor` when a task specifically asks for web performance, Core Web Vitals, loading, rendering, or browser network review.

## Core Rule

Measure before optimizing. If no measurement exists, say "potential impact" instead of pretending the bottleneck is proven.

Performance work must follow this loop:

1. Measure: establish the baseline.
2. Identify: find the bottleneck category.
3. Fix: change the smallest behavior that addresses that bottleneck.
4. Verify: measure before and after.
5. Guard: add a test, budget, monitor, or trace note so the regression is visible later.

Skipping the loop creates performance theater: code looks clever, but Baron has no proof that users or operators are better off.

## Use When

- The task says slow, latency, p95, p99, Core Web Vitals, LCP, INP, CLS, TTFB, loading, rendering, bundle, cache, throughput, CPU, memory, query, database, N+1, pagination, queue, worker, or resource usage.
- A change may increase frontend bundle size, block rendering, add expensive re-renders, fetch unbounded records, introduce N+1 queries, or add synchronous heavy work.
- Monitoring, tests, traces, or users report degraded performance.
- A high-risk feature touches checkout, auth, payment, search, dashboards, exports, uploads, realtime flows, or large datasets.
- A Product Harness story has a proof gap around load time, response time, scalability, or resource use.

Do not use this skill for cosmetic-only work unless it affects loading, rendering, interaction latency, resource usage, or production reliability.

## Baron Contract

- Superpowers remains the workflow owner. This skill only supplies performance judgment.
- Never fabricate metrics, Lighthouse scores, profile traces, query plans, bundle sizes, or user impact.
- Check platform focus first: web, backend, mobile, desktop, CLI/tool, data, cloud, library, or unknown.
- Use the smallest useful measurement. A local timing, query log, bundle report, smoke benchmark, or profiler trace is better than a large speculative investigation.
- Prefer user-visible and operator-visible wins over micro-optimizations.
- Record proof through Baron proof/trace when the task claims improvement.
- If the task is web-facing, route to optional `web-performance-auditor` as an advisory agent, but keep mandatory gates limited to Baron core gates.
- If evidence is missing, output "unknown" or "potential impact", not a confident claim.

## What To Measure First

| Symptom | First Measurement | Likely Area |
| --- | --- | --- |
| First page load feels slow | Network waterfall, TTFB, LCP resource, bundle size | server response, assets, render blocking |
| Interaction feels laggy | browser performance trace, long tasks, re-render count | JavaScript, layout, expensive state updates |
| Layout jumps | CLS attribution, image/font/content dimensions | visual stability |
| API endpoint is slow | request timing, database query timing, trace span | database, network, external service, CPU |
| Dashboard/list is slow | record count, pagination, query count, render cost | unbounded fetch, N+1, DOM size |
| Memory grows over time | heap/profile snapshots, cache size, retained references | leak, unbounded cache, long-lived objects |
| Intermittent latency | p95/p99, lock contention, queue depth, retries | concurrency, external dependency, saturation |
| Build or startup is slow | build profile, dependency graph, cold start timing | tooling, bundle, dynamic imports, I/O |

## Core Web Vitals Reference

Use these as current practical targets for web tasks, then verify against the project requirements and current official guidance when release-critical:

| Metric | Good | Needs Improvement | Poor | Meaning |
| --- | --- | --- | --- | --- |
| LCP | <= 2.5s | <= 4.0s | > 4.0s | main content load speed |
| INP | <= 200ms | <= 500ms | > 500ms | interaction responsiveness |
| CLS | <= 0.1 | <= 0.25 | > 0.25 | visual stability |

Use field data when available. Lab data is useful for reproducible debugging, but field data shows what real users experience.

## Investigation Flow

### 1. Baseline

Record what is known:

- user-visible symptom or explicit performance requirement
- environment: local, CI, staging, production, device/network class
- measurement command or artifact
- current number, if available
- what is unknown

If there is no baseline, the first useful action is usually to create one.

### 2. Classify The Bottleneck

Pick the smallest matching category:

- Database: N+1 queries, missing indexes, unbounded scans, bad joins, over-fetching, transaction locks.
- Network/API: waterfall requests, repeated calls, large payloads, no compression, slow external service, no timeout.
- Frontend loading: oversized bundle, render-blocking JS/CSS, unoptimized images/fonts, late LCP discovery.
- Frontend rendering: long tasks, excessive re-renders, huge DOM, layout thrashing, expensive derived state.
- Caching: missing cache, wrong TTL, stale invalidation, cache stampede, unbounded cache growth.
- CPU/memory: synchronous heavy work, regex backtracking, serialization overhead, leaks, large in-memory arrays.
- Concurrency: lock contention, racey retries, queue backlog, thread or connection pool exhaustion.
- Tooling/build: slow bundler config, expensive type generation, unnecessary transpilation, broad file scans.

Do not optimize a category just because it is fashionable. Match the symptom to evidence.

### 3. Fix Common Anti-Patterns

#### Backend And Database

- N+1 query: batch, join, include, preload, or use a data loader pattern.
- Unbounded list endpoint: add limit, pagination, cursor, projection, and stable ordering.
- Missing index: confirm query shape first, then add an index that matches filters and sort order.
- Large payload: return only needed fields, compress where appropriate, stream large exports.
- Slow external dependency: add timeout, retry policy, circuit breaker, cache, or async job.
- Expensive sync work in request path: move to worker, precompute, cache, or stream.
- Transaction contention: shorten transaction scope, avoid user I/O inside transactions, check lock order.

#### Frontend And Web

- Slow LCP: prioritize the LCP resource, avoid lazy-loading above-the-fold hero assets, reduce TTFB, remove render blockers.
- Poor INP: break long tasks, reduce JS work on interaction, avoid heavy synchronous validation, virtualize large lists.
- High CLS: set dimensions for media, reserve space for async content, stabilize fonts, avoid layout-affecting animations.
- Large bundle: split routes, lazy-load heavy rarely used modules, remove unused dependencies, inspect the actual bundle.
- Re-render churn: stabilize props only where profiling shows cost, memoize expensive calculations, avoid global state fanout.
- Request waterfall: parallelize independent data, move critical data server-side where appropriate, avoid duplicate fetching.

#### Tooling, Desktop, CLI, Mobile

- Broad file scans: respect ignore files, cap traversal, cache file metadata.
- Slow cold start: lazy-load non-critical modules, defer network calls, precompute static indexes.
- Excessive memory: stream large files, avoid collecting entire datasets, bound caches.
- Mobile jank: reduce main-thread work, batch UI updates, profile on representative devices.

## Performance Budget

Use project-specific budgets when they exist. If none exist, propose a budget rather than silently optimizing forever.

Common starting points:

- Web initial JS: keep it intentionally bounded and watch gzipped size.
- Critical CSS and fonts: keep above-the-fold resources small and predictable.
- Images: resize and format for the rendered size, especially above the fold.
- API response: define target p95 for critical endpoints.
- Query count: list pages should not scale queries linearly with displayed rows.
- Memory: define maximum retained cache size or eviction policy.
- Build/startup: define acceptable local and CI timing for the project.

Budgets are not truth by themselves. They are guardrails that make regressions visible.

## Verification

Before claiming performance improved:

- Show before and after measurement, or say why only static risk was addressed.
- Name the bottleneck that was fixed.
- Show the command, report, trace, query plan, bundle output, monitor, or benchmark used.
- Confirm behavior tests still pass.
- Confirm proof/trace was recorded if the task is meaningful or high-risk.
- For web work, mention LCP/INP/CLS status if relevant.
- For backend/database work, mention query count, query time, response time, or resource usage if relevant.
- For cache changes, mention invalidation and stale-data risk.
- For concurrency changes, mention race, lock, retry, and saturation risk.

## Output Contract

Return performance findings in this shape:

1. Status: measured improvement, measured regression, static risk, or unknown.
2. Baseline: number/artifact if available; otherwise say missing.
3. Bottleneck: one primary category and evidence.
4. Change: smallest fix that addresses the bottleneck.
5. Verification: exact command/artifact and before/after result.
6. Guard: test, budget, monitor, trace, or backlog item to prevent regression.
7. Remaining gaps: what Baron still cannot prove.

## Red Flags

- "It should be faster" without measurement.
- Optimizing code that is not on the hot path.
- Adding memoization/cache without invalidation or size limits.
- Replacing readable code with clever code for no measured gain.
- Fetching all records from a table or API endpoint.
- New data access inside loops.
- Large images or fonts with no dimensions or loading strategy.
- React memoization everywhere without profiler evidence.
- Huge bundle growth with no bundle report.
- Removing tests because they slow the build instead of profiling the test bottleneck.
- Treating local fast machine results as user proof.

## Common Rationalizations

| Rationalization | Reality |
| --- | --- |
| "This optimization is obvious." | If it matters, it can be measured or at least labeled as static risk. |
| "We'll optimize later." | Performance debt compounds on hot paths; defer only with an explicit backlog/proof gap. |
| "It is fast locally." | Local speed is not representative of real users, production data, or shared infrastructure. |
| "The framework handles it." | Frameworks do not prevent N+1 queries, oversized bundles, slow dependencies, or bad cache policy. |
| "Caching will fix it." | Caching without invalidation, limits, and stampede control creates correctness and reliability bugs. |
| "Micro-optimizing proves care." | Baron values measured user and operational impact, not cleverness. |
