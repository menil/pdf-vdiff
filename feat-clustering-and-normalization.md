# AI Decision Record: Spatial Column Clustering & Reading Order Reconstruction

## Context & Goal
In PDF content streams, text operators are often stored out of logical reading order (e.g. headers after footers, alternating column lines). Comparing raw token streams directly destroys side-by-side diffing accuracy on multi-column resumes.

## Architecture & Key Decisions
1. **$O(T \log T)$ Horizontal Gap Projection**:
   - Computes horizontal intervals for all tokens and merges adjacent/overlapping intervals up to `gutter_threshold` (default 12.0 pt).
   - Identifies column bands cleanly without requiring full page segmentation heuristics.
2. **Baseline Line Grouping**:
   - Clusters tokens within each column by baseline vertical center proximity ($\Delta y \le 2.0\text{ pt}$).
   - Updates a running average baseline center to accommodate minor font baseline jitter.
3. **Canonical Reading-Order Sorting**:
   - Sorts primary by `column_index` ascending (left-to-right), secondary by line $Y$ descending (top-to-bottom in PDF coordinates), and tertiary by token $X$ ascending (left-to-right).
4. **Unicode NFKD Normalization**:
   - Decomposes ligatures (`ﬁ` -> `fi`, `ﬂ` -> `fl`) and standardizes curly quotes/dashes into ASCII equivalents for text diffing while preserving original visual bounding boxes.

## Alternatives Considered & Rejected
- *Pure Y-sorting without column partitioning*: Rejected because multi-column layouts (e.g. sidebar + main body) would interleave lines from both columns horizontally, producing nonsensical diff results.
