---
name: frontend-design
description: Use when building, styling, or refining frontend interfaces, pages, components, layouts, dashboards, browser-facing product flows, responsive behavior, accessibility, interaction states, or visual polish.
license: Apache-2.0 compatible local guidance; see LICENSE.txt and NOTICE.md
---

# Frontend Design

This bundled optional domain skill guides frontend/UI work. It is lazy-loaded only when `.codex/skills/INDEX.md` routes a matching task here.

## Baron Contract

- Superpowers remains the workflow authority for planning, TDD, debugging, review, and verification.
- This skill is frontend-specific guidance, not a replacement for Superpowers or the 3 core subagents.
- Follow `AGENTS.md`, `.codex/INDEX.md`, `.codex/skills/INDEX.md`, and `.codex/agents/INDEX.md` first.
- Do not override `code-reviewer`, `security-auditor`, or `test-engineer`; route quality, security, and verification gates to those core agents when needed.
- Use repo files, product copy, design tokens, screenshots, and existing UI patterns as evidence. Mark unknown product facts as unknown.

## Use When

Use this when the task asks for frontend implementation, UI design, visual refinement, component styling, page layout, responsive behavior, accessibility, interaction states, dashboard/product screens, or browser-facing product flows.

Do not use it for backend-only, database-only, CLI-only, cloud-only, or pure security work unless a visible frontend surface is part of the task.

## Select The Right Frontend Mode

Choose the least disruptive mode that matches the request and the evidence.

| Mode | Use when | Boundary |
| --- | --- | --- |
| `audit` | Review an existing screen, flow, or component. | Preserve the current product structure and report evidence-backed findings. |
| `refine` | Improve a known surface without changing its product job. | Preserve information architecture, user flows, and established design language. |
| `redesign` | The user explicitly asks for a redesign, or verified evidence shows the current structure cannot serve its job. | Record the approved scope and the evidence for changing structure before editing. |

Do not turn an audit or refinement into a redesign because a new composition
looks more interesting. If product intent, brand evidence, or user constraints
are absent, mark them unknown rather than inventing them.

## Load Only The Needed Reference

This skill is intentionally narrow. Load one or more references only when the
task needs them; do not read every reference for a small CSS or copy change.

- New composition or an approved redesign: read `references/brief-fingerprint.md`.
- Visual refinement, dashboard, landing page, or component redesign: read
  `references/anti-template-gates.md`.
- Responsive layout, stateful flow, form, checkout, or accessibility work: read
  `references/responsive-state-proof.md`.

## Design Thinking

Before coding frontend UI, identify the product context and choose a deliberate aesthetic direction:

- Purpose: what user workflow does this screen help?
- Audience: who uses it, how often, and under what pressure?
- Tone: choose a coherent direction such as calm focus, editorial clarity, friendly learning, refined minimalism, operational density, or playful exploration.
- Constraints: framework, performance, accessibility, content density, responsive layout, and nearby implementation conventions.
- Differentiation: identify the product signal that makes the interface feel designed for this project, not generic.

Before changing a visual composition, write a short brief fingerprint from
repository and user evidence. It must state the product job, audience or
interaction pressure, one justified macrostructure, three product-specific
signals, and generic defaults rejected for this surface. Preserve existing
information architecture unless the selected mode and evidence allow a
redesign.

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

## Baron Design Quality Gate (Changed-Surface)

Run this bounded gate before final response for meaningful frontend work. Check
only the changed surface and the directly related flow; do not turn it into a
whole-product redesign or an unrelated visual scan.

- Reject interchangeable card grids, decorative gradient/blob themes, generic
  dark dashboards, or oversized marketing typography when they do not follow
  product evidence.
- Check that containers express information hierarchy rather than repeating
  rounded panels without purpose.
- Do not introduce arbitrary imagery, iconography, or invented product copy.
- Treat a theme-only swap that leaves a weak composition unchanged as an
  unresolved design issue.
- Check overflow and clipping with long content at a narrow and a wide viewport
  before claiming responsive coverage.
- Check contrast, visible focus, reduced motion, long content, and mobile
  overflow for the changed interaction when relevant.
- Record each check as `observed`, `not applicable with reason`, or `not
  verified`. Never translate missing evidence into a passing claim.

## Verification

- Run the smallest useful browser, DOM, screenshot, component, build, or smoke check that proves the UI behavior.
- If the app cannot run locally, report that as unknown and give the best static evidence from files read.
- For responsive work, check at least mobile and desktop constraints or name the missing viewport proof.
- For interaction work, verify loading, empty, error, disabled, hover/focus, and long-text states when relevant.
- For accessibility work, verify labels, focus path, keyboard reachability, semantic roles, and contrast risks when practical.
- For visual work, compare against existing design tokens and nearby screens instead of inventing a disconnected style.
- For performance-sensitive screens, do not claim faster rendering without measurement; label static concerns as potential impact.
- Record proof and trace evidence through Baron when the frontend task is meaningful or medium/high risk.
- Use only `observed`, `not applicable with reason`, or `not verified` for
  responsive and interaction evidence. A viewport or state that was not
  checked remains `not verified`.

## Output Contract

When reporting frontend work, include:

- what UI surface changed or was reviewed
- selected frontend mode and the evidence that justified it
- files touched or inspected
- accessibility, responsive, and interaction-state risks
- brief fingerprint, anti-template, and responsive/state evidence when those
  checks applied
- browser/screenshot/smoke verification performed, or why it could not run
- remaining gaps that should go to `code-reviewer`, `security-auditor`, or `test-engineer`
