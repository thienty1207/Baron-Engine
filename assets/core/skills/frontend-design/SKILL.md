---
name: frontend-design
description: Use when building, styling, or refining frontend interfaces, pages, components, layouts, dashboards, browser-facing product flows, responsive behavior, accessibility, interaction states, or visual polish.
license: Apache-2.0; adapted from anthropics/skills frontend-design
---

# Frontend Design

This bundled optional domain skill guides frontend/UI work. It is lazy-loaded only when `.codex/skills/INDEX.md` routes a matching task here.

## Compatibility Guardrails

- Superpowers remains the workflow authority for planning, TDD, debugging, review, and verification.
- This skill is frontend-specific guidance, not a replacement for Superpowers or the 3 core subagents.
- Follow `AGENTS.md`, `.codex/INDEX.md`, `.codex/skills/INDEX.md`, and `.codex/agents/INDEX.md` first.
- Do not override `code-reviewer`, `security-auditor`, or `test-engineer`; route quality, security, and verification gates to those core agents when needed.
- Use repo files, product copy, design tokens, screenshots, and existing UI patterns as evidence. Mark unknown product facts as unknown.

## When To Use

Use this when the task asks for frontend implementation, UI design, visual refinement, component styling, page layout, responsive behavior, accessibility, interaction states, dashboard/product screens, or browser-facing product flows.

Do not use it for backend-only, database-only, CLI-only, cloud-only, or pure security work unless a visible frontend surface is part of the task.

## Design Thinking

Before coding frontend UI, identify the product context and choose a deliberate aesthetic direction:

- Purpose: what user workflow does this screen help?
- Audience: who uses it, how often, and under what pressure?
- Tone: choose a coherent direction such as calm focus, editorial clarity, friendly learning, refined minimalism, operational density, or playful exploration.
- Constraints: framework, performance, accessibility, content density, responsive layout, and nearby implementation conventions.
- Differentiation: identify the product signal that makes the interface feel designed for this project, not generic.

## Implementation Standard

Build real working UI code that is:

- functional, responsive, and accessible
- visually intentional without becoming noisy
- aligned with existing repo conventions and design tokens
- refined in spacing, typography, color, states, motion, and hierarchy
- verified with the smallest useful browser, screenshot, or smoke check when the app can run locally
- honest about performance: if no browser/Lighthouse/devtools evidence exists, describe performance concerns as potential impact, not measured fact

## Frontend Aesthetics Guidelines

- Typography: choose fonts and scale that fit the product personality; avoid accidental default styling.
- Color and theme: use meaningful contrast and accents; prefer project tokens/CSS variables when present.
- Motion: use animation for useful transitions and feedback, not constant decoration.
- Spatial composition: use hierarchy, rhythm, responsive constraints, and layout variety.
- Visual details: add imagery, icons, texture, depth, or illustration only when it helps the user understand or act.
- Density: operational tools should stay scan-friendly; marketing surfaces can be more expressive.

## Avoid

- generic AI-looking UI
- repeated purple-gradient-on-white styling
- indistinct SaaS cards without product character
- decorative complexity that weakens readability
- visible explanation text that describes the UI instead of serving the product
- layout shifts, overlapping text, or responsive states that break button/card/container boundaries
- stack-mismatched advice such as recommending Next.js-only patterns for Vue/Svelte/vanilla projects without repo evidence

## Quality Rubric

Before final response, check:

- Correctness: the UI satisfies the requested workflow, not only the screenshot.
- Accessibility: keyboard, focus, labels, contrast, and reduced-motion risks are addressed or named.
- Responsiveness: mobile, tablet, and desktop layout constraints are stable.
- Interaction state: loading, empty, error, disabled, hover/focus, and long-text states are handled when relevant.
- Performance: images, fonts, heavy components, route loading, and unnecessary re-render risks are identified.
- Baron evidence: changed files, browser/smoke proof, remaining core-agent gates, and trace/proof gaps are clear.

## Output Contract

When reporting frontend work, include:

- what UI surface changed or was reviewed
- files touched or inspected
- accessibility, responsive, and interaction-state risks
- browser/screenshot/smoke verification performed, or why it could not run
- remaining gaps that should go to `code-reviewer`, `security-auditor`, or `test-engineer`

## Attribution

Adapted from Anthropic's `frontend-design` skill under Apache-2.0. Additional frontend-quality rubric ideas are informed by MIT-licensed `addyosmani/agent-skills`, rewritten as Baron-native optional guidance. See `LICENSE.txt` in this folder.
