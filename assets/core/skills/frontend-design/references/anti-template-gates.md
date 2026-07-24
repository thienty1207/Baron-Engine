# Anti-Template Gates

Run this bounded audit only against the changed UI surface and its directly
affected interaction. It is a quality gate, not permission to rewrite unrelated
screens.

Record each row as `observed`, `corrected`, `not applicable with reason`, or
`not verified`. A missing check is not a pass.

| Gate | Look for | Evidence to record |
| --- | --- | --- |
| Composition | An interchangeable SaaS card grid or repeated panels that do not express information hierarchy. | Why the chosen hierarchy fits the product job. |
| Decoration | Decorative gradients, floating blobs, generic dark dashboards, or visual effects that hide the product. | Existing token, content, or workflow evidence for the chosen visual treatment. |
| Typography | Oversized marketing typography inside an operational tool or density that harms scanning. | Relevant text scale and reading or action priority. |
| Containers | Rounded containers repeated without a distinct role, grouping rule, or action hierarchy. | Container roles and the relationship between groups. |
| Assets and copy | Arbitrary iconography, stock imagery, or invented product copy. | Repository or user source for assets and product wording. |
| Theme-only change | New colors applied to the same weak composition without a workflow improvement. | The actual usability or structure improvement. |
| Accessible interaction | Low contrast, hidden focus, inaccessible action, or ignored reduced motion. | Observed focus, contrast, semantic, and motion behavior. |
| Mobile behavior | Overflow, clipped long labels, compressed actions, or touch targets that fail on narrow layouts. | Narrow viewport and long-content evidence. |

If a gate finds a problem, either correct it or report the remaining issue with
its impact. Do not claim visual polish solely because the theme changed.
