# AI Decision Record: Hierarchical Two-Pass Sequence Diff Engine

## Context & Goal
Diffing large unstructured token streams directly with Myers sequence matching causes $O(N^2)$ quadratic slowdowns on restructured documents and produces visually noisy diffs when paragraphs change.

## Architecture & Key Decisions
1. **Hierarchical 2-Pass Architecture**:
   - **Pass 1 (Line Level)**: Performs fast Myers sequence alignment on normalized line strings ($O(L \cdot D_L)$), identifying unchanged lines, deleted lines, inserted lines, and replaced line blocks.
   - **Pass 2 (Token Level)**: For replaced lines only, performs token-level Myers diffing to highlight specific altered words.
2. **Line Tint & Altered Word Separation**:
   - Replaced lines receive a subtle background tint (`DiffOpKind::Replace`).
   - Specifically altered words receive distinct deletion/insertion highlights.
3. **Contiguous Highlight Unioning**:
   - Merges adjacent deleted/inserted tokens on the same line into a single bounding box $[x_{\min}, y_{\min}, x_{\max}, y_{\max}]$ expanded with padding (`HIGHLIGHT_PAD_X`, `HIGHLIGHT_PAD_Y`), eliminating vertical seam artifacts.
4. **Page-Pair Scoping**:
   - Diffing operates independently on page pairs $(P_{base}[i], P_{tailored}[i])$ with empty-slice fallback for missing pages on mismatched page counts.

## Alternatives Considered & Rejected
- *Unbounded Token Diffing*: Rejected due to $O(N^2)$ complexity and noisy inter-word diffs on multi-line paragraph edits.
