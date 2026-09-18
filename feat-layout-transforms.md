# AI Decision Record: Canvas Layout Geometry and Top-Alignment Transforms

## Context & Goal
Provide exact mathematical coordinate transformations for side-by-side landscape canvas composition under an optional header banner across equal and unequal page dimensions.

## Architecture & Key Decisions
1. **Dimension Formulas**:
   - $W_{out} = W_b + W_t + \text{Gutter}$
   - $H_{out} = \max(H_b, H_t) + H_{header}$
2. **Top-Alignment Invariant Under Header**:
   - Source pages sit directly below the header banner ($Y = \max(H_b, H_t)$).
   - In PDF bottom-left coordinates:
     - Left (Base) origin: $(0, \max(H_b, H_t) - H_b)$
     - Right (Tailored) origin: $(W_b + \text{Gutter}, \max(H_b, H_t) - H_t)$
   - Guarantees that when page heights differ (e.g. US Letter vs A4), top margins align cleanly across panes.
3. **Projection Mapping**:
   - `project_base_rect` and `project_tailored_rect` project local token and highlight coordinates directly into canvas space.

## Alternatives Considered & Rejected
- *Vertical Center Alignment*: Rejected because centering causes vertical baseline offsets between lines across the left and right pages.
