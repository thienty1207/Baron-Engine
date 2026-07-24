---
name: superpowers
description: Baron workflow core for planning, TDD, debugging, review, and verification.
---

# Superpowers

Superpowers is Baron's only workflow core.

Read the narrow sub-skill that matches the current workflow stage. Start with
`using-superpowers/SKILL.md`. Do not recursively load every sub-skill, and do
not act from a remembered summary when the selected sub-skill is available.

Baron provides memory, context, plan, harness, proof, and trace state.
Superpowers provides the discipline used to act on that state.

## Baron Integration Contract

The vendored sub-skills are the detailed authority. These short rules protect
the 6.2 workflow boundaries most likely to be lost during a long session:

- Resolve a plan-scoped SDD workspace before using a progress ledger. Never
  reuse the old flat ledger or a ledger naming another plan.
- If different plan paths have the same basename and resolve to one workspace,
  stop before writing and give the new plan a unique basename. Never overwrite
  or migrate the other plan's ledger.
- Rounds 1-3 resume the original implementer, followed by a scoped independent
  re-review.
- Rounds 4-5 use a fresh implementer on a more capable model, followed by the
  same scoped re-review.
- After round 5, stop the fix loop, adjudicate every open finding, and route the
  result. Do not continue an unbounded loop or claim completion.
- Text presence alone is not test proof. Test observable behavior with an
  independent expected result and confirm the test fails when production
  behavior is deliberately broken.
- Keep Baron plan, proof, trace, and Vault state synchronized around the
  Superpowers workflow. Neither system replaces the other.
