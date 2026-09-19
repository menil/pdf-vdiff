# AI Decision Record: Fine-Grained Punctuation Diff Highlighting

## Context & Problem Statement
In PDF visual diffing, slight punctuation changes (such as adding a trailing comma `tests` -> `tests,` or swapping a period `tests.` -> `tests,`) previously caused the entire adjacent word to be marked as deleted/inserted and highlighted across both panes. This created unnecessary visual noise and obscured the precise location of punctuation-only modifications.

## Architectural Decisions
1. **Discrete Punctuation Tokenization (`src/pdf/extract.rs`)**:
   - Introduced `is_punctuation(c: char) -> bool` covering ASCII punctuation (`c.is_ascii_punctuation()`), Unicode General Punctuation (`\u{2000}`..=`\u{206F}`), and common typographical symbols (`«`, `»`, `“`, `”`, `—`, `–`, `•`, etc.).
   - Updated `extract_page_tokens` to flush the active word buffer upon encountering punctuation and emit punctuation glyphs as separate `TextToken` entries with exact individual bounding boxes.
   - Maintained exact `trailing_space` flags across words and punctuation tokens to ensure correct line reconstruction and column layout detection.

2. **Punctuation-Aware Line Clustering (`src/cluster.rs`)**:
   - Updated `LineCluster` to track vertical spans `[y0, y1]` alongside line centers.
   - Added `token_matches_line` to ensure small-height glyphs (commas with descenders, periods without ascenders, quotes/apostrophes at cap-height) are clustered onto the same visual line as their adjacent words rather than being segregated into distinct lines due to center vertical offset.
   - Constrained `y_center` running updates to tokens with `height() > 4.0 pt` so tiny punctuation glyphs do not skew the baseline center of the line.

3. **Myers Diff & Highlight Merging Preservation (`src/diff.rs`)**:
   - Myers diff compares `normalized_text`, correctly matching identical words while isolating punctuation additions, deletions, or swaps.
   - Adjacent multi-token edits (e.g. replacing a full phrase with trailing punctuation) continue to merge cleanly into a single contiguous bounding box via `merge_token_highlights`.

## Alternatives Considered & Rejected
- **Sub-string / Character-Level Bounding Box Slicing in Diff Engine**: Computing sub-string bounding box projections from monolithic word tokens during diffing was rejected because it requires font metrics / character width estimation outside of PDFium's exact glyph layout engine. Extracting exact per-character glyph boxes directly at the PDFium layer guarantees 100% vector layout fidelity.
- **Always Exploding Every Character into a Token**: Tokenizing every single character by default was rejected for word-granularity diffing because it increases token count and diff comparison overhead without benefit for intact words, whereas punctuation-separated word tokens provide the optimal balance of performance and precision.

## Verification & Validation
- Added 7 new unit tests in `src/pdf/extract.rs` and `src/diff.rs` covering punctuation detection, extraction, additions, deletions, swaps, and phrase preservation.
- Reached **92.01% overall line coverage** across the workspace.
- Validated via `nix-shell --run "just validate"` and regenerated `example/resume_diff.pdf`.
