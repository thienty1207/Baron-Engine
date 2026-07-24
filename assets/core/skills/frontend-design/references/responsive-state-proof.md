# Responsive And State Proof

Use this evidence matrix for meaningful responsive or interaction work. Check
only states relevant to the changed flow. Each row must be `observed`, `not
applicable with reason`, or `not verified`; do not use a passing label for an
unobserved viewport or state.

| Surface | What to check | Status | Evidence or reason |
| --- | --- | --- | --- |
| Narrow viewport | Content order, action reachability, long labels, overflow, and touch targets. |  |  |
| Wide viewport | Stable hierarchy, readable line length, and no unused layout expansion. |  |  |
| Long content | Long labels, dense lists, translated text, and dynamic content growth. |  |  |
| Loading | Pending feedback, disabled actions, and layout stability. |  |  |
| Empty | Clear next action and no misleading empty success state. |  |  |
| Error | Recoverable message, preserved input where appropriate, and usable retry. |  |  |
| Disabled | Reason, accessibility semantics, and no hidden required action. |  |  |
| Keyboard and focus | Logical focus order, visible focus, labels, and keyboard reachability. |  |  |
| Reduced motion | No essential action depends on animation; motion can be reduced. |  |  |

When a browser or device cannot be run locally, preserve that limitation as
`not verified` and cite the static repository evidence that was available.
