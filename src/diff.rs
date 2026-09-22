//! Hierarchical 2-pass sequence diff engine and highlight box unioning.

use crate::model::{DiffOpKind, HighlightSpan, PageText, Rect, TextToken};
use crate::theme::{HIGHLIGHT_PAD_X, HIGHLIGHT_PAD_Y};
use clap::ValueEnum;
use similar::{capture_diff_slices, Algorithm};

/// Diff granularity mode selectable via `--granularity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum DiffGranularity {
    /// Diff by word tokens (Default).
    #[default]
    Word,
    /// Diff by full lines.
    Line,
    /// Diff by individual characters/glyphs.
    Character,
}

/// Diff result for a single side-by-side page pair.
#[derive(Debug, Clone, PartialEq)]
pub struct PageDiffResult {
    /// 0-indexed page index.
    pub page_index: usize,
    /// Highlight spans to render on the Left (Base) pane.
    pub base_highlights: Vec<HighlightSpan>,
    /// Highlight spans to render on the Right (Tailored) pane.
    pub tailored_highlights: Vec<HighlightSpan>,
    /// True if any differences were detected on this page.
    pub has_differences: bool,
}

/// Overall document diff result across all pages.
#[derive(Debug, Clone, PartialEq)]
pub struct DocumentDiffResult {
    /// Per-page diff results.
    pub pages: Vec<PageDiffResult>,
    /// Total count of differences found across all pages.
    pub total_differences: usize,
    /// True if any differences were detected anywhere in the document.
    pub has_differences: bool,
}

/// Line representation grouping tokens on a single visual line.
///
/// Note: `line_index` and `text` are retained for debugging, logging, and future structured diff output.
#[derive(Debug, Clone)]
struct LineGroup<'a> {
    #[allow(dead_code)]
    pub line_index: usize,
    #[allow(dead_code)]
    pub text: String,
    pub normalized_text: String,
    pub tokens: &'a [TextToken],
}

/// Helper function to extract normalized line string slices from a slice of LineGroup.
fn extract_normalized_line_strings<'a>(lines: &'a [LineGroup<'a>]) -> Vec<&'a str> {
    lines.iter().map(|l| l.normalized_text.as_str()).collect()
}

/// Compares two documents page-by-page and returns structured diff highlights.
///
/// Note: `total_differences` represents the aggregate count of highlight bounding box spans
/// rendered across both base (deletions/replacements) and tailored (insertions/replacements) pages.
pub fn diff_documents(
    base_pages: &[PageText],
    tailored_pages: &[PageText],
    granularity: DiffGranularity,
) -> DocumentDiffResult {
    let max_pages = base_pages.len().max(tailored_pages.len());
    let mut page_results = Vec::with_capacity(max_pages);
    let mut total_diffs = 0;

    for i in 0..max_pages {
        let base_page = base_pages.get(i);
        let tailored_page = tailored_pages.get(i);

        let result = diff_page_pair(i, base_page, tailored_page, granularity);
        if result.has_differences {
            total_diffs += result.base_highlights.len() + result.tailored_highlights.len();
        }
        page_results.push(result);
    }

    let has_differences = total_diffs > 0;
    DocumentDiffResult {
        pages: page_results,
        total_differences: total_diffs,
        has_differences,
    }
}

/// Compares a single page pair and produces Left (Base) and Right (Tailored) highlight spans.
pub fn diff_page_pair(
    page_index: usize,
    base_page: Option<&PageText>,
    tailored_page: Option<&PageText>,
    granularity: DiffGranularity,
) -> PageDiffResult {
    let empty_tokens = Vec::new();
    let base_tokens = base_page
        .map(|p| p.tokens.as_slice())
        .unwrap_or(&empty_tokens);
    let tailored_tokens = tailored_page
        .map(|p| p.tokens.as_slice())
        .unwrap_or(&empty_tokens);

    if base_tokens.is_empty() && tailored_tokens.is_empty() {
        return PageDiffResult {
            page_index,
            base_highlights: Vec::new(),
            tailored_highlights: Vec::new(),
            has_differences: false,
        };
    }

    // Fast path: if base is missing, all tailored tokens are inserted
    if base_tokens.is_empty() {
        let highlights = merge_contiguous_highlights(tailored_tokens, DiffOpKind::Insert, false);
        return PageDiffResult {
            page_index,
            base_highlights: Vec::new(),
            tailored_highlights: highlights,
            has_differences: true,
        };
    }

    // Fast path: if tailored is missing, all base tokens are deleted
    if tailored_tokens.is_empty() {
        let highlights = merge_contiguous_highlights(base_tokens, DiffOpKind::Delete, false);
        return PageDiffResult {
            page_index,
            base_highlights: highlights,
            tailored_highlights: Vec::new(),
            has_differences: true,
        };
    }

    // Token-stream diffing
    match granularity {
        DiffGranularity::Line => diff_by_lines_only(page_index, base_tokens, tailored_tokens),
        DiffGranularity::Word | DiffGranularity::Character => {
            diff_token_stream(page_index, base_tokens, tailored_tokens)
        }
    }
}

/// Direct token-stream diffing across natural reading order.
///
/// Compares normalized token texts directly using Myers diff.
/// Tokens that shift across line wraps/reflows without changing their textual content
/// are matched as equal and produce zero highlight rectangles.
/// Only inserted tokens receive addition highlights (tailored) and deleted tokens
/// receive deletion highlights (base).
///
/// Note: Replace operations emit direct token-level deletions on the Base document
/// and insertions on the Tailored document without full-line background tint boxes,
/// ensuring that only changed text is highlighted.
fn diff_token_stream(
    page_index: usize,
    base_tokens: &[TextToken],
    tailored_tokens: &[TextToken],
) -> PageDiffResult {
    let base_words: Vec<&str> = base_tokens
        .iter()
        .map(|t| t.normalized_text.as_str())
        .collect();
    let tailored_words: Vec<&str> = tailored_tokens
        .iter()
        .map(|t| t.normalized_text.as_str())
        .collect();

    let diff_ops = capture_diff_slices(Algorithm::Myers, &base_words, &tailored_words);

    let mut base_highlights = Vec::new();
    let mut tailored_highlights = Vec::new();
    let mut has_differences = false;

    for op in diff_ops {
        match op {
            similar::DiffOp::Equal { .. } => {
                // Unchanged tokens: 0 highlights
            }
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                has_differences = true;
                let slice = &base_tokens[old_index..old_index + old_len];
                base_highlights.extend(merge_contiguous_highlights(
                    slice,
                    DiffOpKind::Delete,
                    false,
                ));
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                has_differences = true;
                let slice = &tailored_tokens[new_index..new_index + new_len];
                tailored_highlights.extend(merge_contiguous_highlights(
                    slice,
                    DiffOpKind::Insert,
                    false,
                ));
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                has_differences = true;
                // Treat Replace as token deletion on Base and token addition on Tailored (text-only diffing)
                let old_slice = &base_tokens[old_index..old_index + old_len];
                let new_slice = &tailored_tokens[new_index..new_index + new_len];
                base_highlights.extend(merge_contiguous_highlights(
                    old_slice,
                    DiffOpKind::Delete,
                    false,
                ));
                tailored_highlights.extend(merge_contiguous_highlights(
                    new_slice,
                    DiffOpKind::Insert,
                    false,
                ));
            }
        }
    }

    PageDiffResult {
        page_index,
        base_highlights,
        tailored_highlights,
        has_differences,
    }
}

/// Fallback for line-only diff granularity.
fn diff_by_lines_only(
    page_index: usize,
    base_tokens: &[TextToken],
    tailored_tokens: &[TextToken],
) -> PageDiffResult {
    let base_lines = group_tokens_by_line(base_tokens);
    let tailored_lines = group_tokens_by_line(tailored_tokens);

    let base_line_strs = extract_normalized_line_strings(&base_lines);
    let tailored_line_strs = extract_normalized_line_strings(&tailored_lines);

    let diff_ops = capture_diff_slices(Algorithm::Myers, &base_line_strs, &tailored_line_strs);

    let mut base_highlights = Vec::new();
    let mut tailored_highlights = Vec::new();
    let mut has_differences = false;

    for op in diff_ops {
        match op {
            similar::DiffOp::Equal { .. } => {}
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                has_differences = true;
                for line in &base_lines[old_index..old_index + old_len] {
                    if let Some(bounds) = compute_line_bounds(line.tokens) {
                        base_highlights.push(HighlightSpan::new(bounds, DiffOpKind::Delete, false));
                    }
                }
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                has_differences = true;
                for line in &tailored_lines[new_index..new_index + new_len] {
                    if let Some(bounds) = compute_line_bounds(line.tokens) {
                        tailored_highlights.push(HighlightSpan::new(
                            bounds,
                            DiffOpKind::Insert,
                            false,
                        ));
                    }
                }
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                has_differences = true;
                for line in &base_lines[old_index..old_index + old_len] {
                    if let Some(bounds) = compute_line_bounds(line.tokens) {
                        base_highlights.push(HighlightSpan::new(bounds, DiffOpKind::Delete, false));
                    }
                }
                for line in &tailored_lines[new_index..new_index + new_len] {
                    if let Some(bounds) = compute_line_bounds(line.tokens) {
                        tailored_highlights.push(HighlightSpan::new(
                            bounds,
                            DiffOpKind::Insert,
                            false,
                        ));
                    }
                }
            }
        }
    }

    PageDiffResult {
        page_index,
        base_highlights,
        tailored_highlights,
        has_differences,
    }
}

/// Groups tokens by `line_index` into structured LineGroup objects.
fn group_tokens_by_line(tokens: &[TextToken]) -> Vec<LineGroup<'_>> {
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut start_idx = 0;
    let mut current_line = tokens[0].line_index;

    for (i, token) in tokens.iter().enumerate() {
        if token.line_index != current_line {
            let slice = &tokens[start_idx..i];
            lines.push(build_line_group(current_line, slice));
            start_idx = i;
            current_line = token.line_index;
        }
    }

    if start_idx < tokens.len() {
        let slice = &tokens[start_idx..];
        lines.push(build_line_group(current_line, slice));
    }

    lines
}

fn build_line_group<'a>(line_index: usize, slice: &'a [TextToken]) -> LineGroup<'a> {
    let mut text = String::new();
    let mut normalized_text = String::new();

    for (i, t) in slice.iter().enumerate() {
        text.push_str(&t.text);
        normalized_text.push_str(&t.normalized_text);
        if t.trailing_space && i + 1 < slice.len() {
            text.push(' ');
            normalized_text.push(' ');
        }
    }

    LineGroup {
        line_index,
        text,
        normalized_text,
        tokens: slice,
    }
}

/// Computes the bounding box covering an entire line of tokens.
fn compute_line_bounds(tokens: &[TextToken]) -> Option<Rect> {
    tokens
        .first()
        .map(|first| {
            tokens
                .iter()
                .fold(first.bounds, |acc, t| acc.union(&t.bounds))
        })
        .map(|r| r.expand(HIGHLIGHT_PAD_X, HIGHLIGHT_PAD_Y))
}

/// Merges contiguous altered tokens (or token references) on the same line into unified highlight rectangles.
fn merge_token_highlights<T: std::borrow::Borrow<TextToken>>(
    tokens: &[T],
    op: DiffOpKind,
    is_modified_token: bool,
) -> Vec<HighlightSpan> {
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut spans = Vec::new();
    let first = tokens[0].borrow();
    let mut current_line = first.line_index;
    let mut current_box = first.bounds;

    for t in &tokens[1..] {
        let t = t.borrow();
        if t.line_index == current_line {
            current_box = current_box.union(&t.bounds);
        } else {
            spans.push(HighlightSpan::new(
                current_box.expand(HIGHLIGHT_PAD_X, HIGHLIGHT_PAD_Y),
                op,
                is_modified_token,
            ));
            current_line = t.line_index;
            current_box = t.bounds;
        }
    }

    spans.push(HighlightSpan::new(
        current_box.expand(HIGHLIGHT_PAD_X, HIGHLIGHT_PAD_Y),
        op,
        is_modified_token,
    ));

    spans
}

/// Convenience alias for merging owned token slices.
fn merge_contiguous_highlights(
    tokens: &[TextToken],
    op: DiffOpKind,
    is_modified_token: bool,
) -> Vec<HighlightSpan> {
    merge_token_highlights(tokens, op, is_modified_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_token(text: &str, x0: f32, y0: f32, x1: f32, y1: f32, line: usize) -> TextToken {
        TextToken::new(
            text.to_string(),
            text.to_string(),
            Rect::new(x0, y0, x1, y1),
            0,
            0,
            line,
            true,
        )
    }

    #[test]
    fn test_identical_documents() {
        let t1 = make_token("Hello", 50.0, 750.0, 90.0, 762.0, 0);
        let t2 = make_token("World", 95.0, 750.0, 140.0, 762.0, 0);

        let p1 = PageText::new(vec![t1.clone(), t2.clone()], 612.0, 792.0, 0);
        let p2 = PageText::new(vec![t1, t2], 612.0, 792.0, 0);

        let res = diff_documents(&[p1], &[p2], DiffGranularity::Word);
        assert!(!res.has_differences);
        assert_eq!(res.total_differences, 0);
        assert!(res.pages[0].base_highlights.is_empty());
        assert!(res.pages[0].tailored_highlights.is_empty());
    }

    #[test]
    fn test_word_insertion_and_deletion() {
        let b1 = make_token("Software", 50.0, 750.0, 100.0, 762.0, 0);
        let b2 = make_token("Engineer", 105.0, 750.0, 160.0, 762.0, 0);

        let t1 = make_token("Principal", 50.0, 750.0, 105.0, 762.0, 0);
        let t2 = make_token("Software", 110.0, 750.0, 160.0, 762.0, 0);
        let t3 = make_token("Architect", 165.0, 750.0, 220.0, 762.0, 0);

        let base_page = PageText::new(vec![b1, b2], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t1, t2, t3], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        assert!(!res.pages[0].base_highlights.is_empty());
        assert!(!res.pages[0].tailored_highlights.is_empty());
    }

    #[test]
    fn test_page_count_mismatch() {
        let b1 = make_token("Page1", 50.0, 750.0, 90.0, 762.0, 0);
        let t1 = make_token("Page1", 50.0, 750.0, 90.0, 762.0, 0);
        let t2 = make_token("Page2", 50.0, 750.0, 90.0, 762.0, 0);

        let base_pages = vec![PageText::new(vec![b1], 612.0, 792.0, 0)];
        let tailored_pages = vec![
            PageText::new(vec![t1], 612.0, 792.0, 0),
            PageText::new(vec![t2], 612.0, 792.0, 1),
        ];

        let res = diff_documents(&base_pages, &tailored_pages, DiffGranularity::Word);
        assert!(res.has_differences);
        assert_eq!(res.pages.len(), 2);
        assert!(!res.pages[0].has_differences);
        assert!(res.pages[1].has_differences);
        assert!(res.pages[1].base_highlights.is_empty());
        assert!(!res.pages[1].tailored_highlights.is_empty());
    }

    #[test]
    fn test_line_granularity_diff() {
        let b1 = make_token("Deleted Line", 50.0, 750.0, 150.0, 762.0, 0);
        let b2 = make_token("Modified Base Line", 50.0, 720.0, 200.0, 732.0, 1);

        let t1 = make_token("Modified Tailored Line", 50.0, 750.0, 210.0, 762.0, 0);
        let t2 = make_token("Inserted Line", 50.0, 720.0, 160.0, 732.0, 1);

        let base_page = PageText::new(vec![b1, b2], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t1, t2], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Line);
        assert!(res.has_differences);
        assert!(!res.pages[0].base_highlights.is_empty());
        assert!(!res.pages[0].tailored_highlights.is_empty());
    }

    #[test]
    fn test_empty_page_pairs() {
        let empty_base = PageText::new(Vec::new(), 612.0, 792.0, 0);
        let empty_tailored = PageText::new(Vec::new(), 612.0, 792.0, 0);

        let res = diff_documents(&[empty_base], &[empty_tailored], DiffGranularity::Word);
        assert!(!res.has_differences);

        // Missing base
        let t = make_token("New", 50.0, 700.0, 80.0, 712.0, 0);
        let single_tailored = PageText::new(vec![t], 612.0, 792.0, 0);
        let res_missing_base =
            diff_page_pair(0, None, Some(&single_tailored), DiffGranularity::Word);
        assert!(res_missing_base.has_differences);
        assert!(!res_missing_base.tailored_highlights.is_empty());

        // Missing tailored
        let b = make_token("Old", 50.0, 700.0, 80.0, 712.0, 0);
        let single_base = PageText::new(vec![b], 612.0, 792.0, 0);
        let res_missing_tailored =
            diff_page_pair(0, Some(&single_base), None, DiffGranularity::Word);
        assert!(res_missing_tailored.has_differences);
        assert!(!res_missing_tailored.base_highlights.is_empty());
    }

    #[test]
    fn test_character_granularity_diff() {
        let b1 = make_token("Rust", 50.0, 750.0, 100.0, 762.0, 0);
        let t1 = make_token("Rest", 50.0, 750.0, 100.0, 762.0, 0);

        let base_page = PageText::new(vec![b1], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t1], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Character);
        assert!(res.has_differences);
        assert!(!res.pages[0].base_highlights.is_empty());
        assert!(!res.pages[0].tailored_highlights.is_empty());
    }

    #[test]
    fn test_contiguous_multi_line_token_merging() {
        let t1 = make_token("Line0-A", 50.0, 750.0, 100.0, 762.0, 0);
        let t2 = make_token("Line0-B", 105.0, 750.0, 150.0, 762.0, 0);
        let t3 = make_token("Line1-A", 50.0, 720.0, 100.0, 732.0, 1);

        let merged = merge_contiguous_highlights(&[t1, t2, t3], DiffOpKind::Delete, false);
        assert_eq!(merged.len(), 2); // 1 span for line 0, 1 span for line 1
    }

    #[test]
    fn test_paragraph_reflow_insertion_no_false_positives() {
        // Base paragraph across 2 lines:
        // Line 0: "The quick brown fox"
        // Line 1: "jumps over the lazy dog"
        let b0_0 = make_token("The", 50.0, 750.0, 70.0, 762.0, 0);
        let b0_1 = make_token("quick", 75.0, 750.0, 105.0, 762.0, 0);
        let b0_2 = make_token("brown", 110.0, 750.0, 145.0, 762.0, 0);
        let b0_3 = make_token("fox", 150.0, 750.0, 170.0, 762.0, 0);
        let b1_0 = make_token("jumps", 50.0, 730.0, 85.0, 742.0, 1);
        let b1_1 = make_token("over", 90.0, 730.0, 115.0, 742.0, 1);
        let b1_2 = make_token("the", 120.0, 730.0, 140.0, 742.0, 1);
        let b1_3 = make_token("lazy", 145.0, 730.0, 170.0, 742.0, 1);
        let b1_4 = make_token("dog", 175.0, 730.0, 195.0, 742.0, 1);

        // Tailored paragraph with "very" inserted on Line 0, causing "brown" and "fox" to wrap to Line 1:
        // Line 0: "The very quick"
        // Line 1: "brown fox jumps over the lazy dog"
        let t0_0 = make_token("The", 50.0, 750.0, 70.0, 762.0, 0);
        let t0_1 = make_token("very", 75.0, 750.0, 100.0, 762.0, 0); // Inserted
        let t0_2 = make_token("quick", 105.0, 750.0, 135.0, 762.0, 0);
        let t1_0 = make_token("brown", 50.0, 730.0, 85.0, 742.0, 1); // Wrapped
        let t1_1 = make_token("fox", 90.0, 730.0, 110.0, 742.0, 1); // Wrapped
        let t1_2 = make_token("jumps", 115.0, 730.0, 150.0, 742.0, 1);
        let t1_3 = make_token("over", 155.0, 730.0, 180.0, 742.0, 1);
        let t1_4 = make_token("the", 185.0, 730.0, 205.0, 742.0, 1);
        let t1_5 = make_token("lazy", 210.0, 730.0, 235.0, 742.0, 1);
        let t1_6 = make_token("dog", 240.0, 730.0, 260.0, 742.0, 1);

        let base_page = PageText::new(
            vec![b0_0, b0_1, b0_2, b0_3, b1_0, b1_1, b1_2, b1_3, b1_4],
            612.0,
            792.0,
            0,
        );
        let tailored_page = PageText::new(
            vec![t0_0, t0_1, t0_2, t1_0, t1_1, t1_2, t1_3, t1_4, t1_5, t1_6],
            612.0,
            792.0,
            0,
        );

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base should have ZERO highlights because nothing was deleted
        assert!(
            res.pages[0].base_highlights.is_empty(),
            "Expected 0 base highlights, got: {:?}",
            res.pages[0].base_highlights
        );
        // Tailored should have exactly 1 highlight covering only "very"
        assert_eq!(
            res.pages[0].tailored_highlights.len(),
            1,
            "Expected 1 tailored highlight for 'very', got: {:?}",
            res.pages[0].tailored_highlights
        );
        assert_eq!(res.pages[0].tailored_highlights[0].op, DiffOpKind::Insert);
    }

    #[test]
    fn test_paragraph_reflow_deletion_no_false_positives() {
        // Base: Line 0 has "Software development lifecycle management", Line 1 has "practices and tooling"
        let b0_0 = make_token("Software", 50.0, 750.0, 100.0, 762.0, 0);
        let b0_1 = make_token("development", 105.0, 750.0, 170.0, 762.0, 0); // To delete
        let b0_2 = make_token("lifecycle", 175.0, 750.0, 225.0, 762.0, 0); // To delete
        let b0_3 = make_token("management", 230.0, 750.0, 300.0, 762.0, 0);
        let b1_0 = make_token("practices", 50.0, 730.0, 105.0, 742.0, 1);
        let b1_1 = make_token("and", 110.0, 730.0, 130.0, 742.0, 1);
        let b1_2 = make_token("tooling", 135.0, 730.0, 180.0, 742.0, 1);

        // Tailored: "development lifecycle" removed, "practices" pulled up to Line 0
        let t0_0 = make_token("Software", 50.0, 750.0, 100.0, 762.0, 0);
        let t0_1 = make_token("management", 105.0, 750.0, 175.0, 762.0, 0);
        let t0_2 = make_token("practices", 180.0, 750.0, 235.0, 762.0, 0); // Pulled up
        let t1_0 = make_token("and", 50.0, 730.0, 70.0, 742.0, 1);
        let t1_1 = make_token("tooling", 75.0, 730.0, 120.0, 742.0, 1);

        let base_page = PageText::new(
            vec![b0_0, b0_1, b0_2, b0_3, b1_0, b1_1, b1_2],
            612.0,
            792.0,
            0,
        );
        let tailored_page = PageText::new(vec![t0_0, t0_1, t0_2, t1_0, t1_1], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base should have exactly 1 merged highlight span covering "development lifecycle"
        assert_eq!(
            res.pages[0].base_highlights.len(),
            1,
            "Expected 1 merged base highlight, got: {:?}",
            res.pages[0].base_highlights
        );
        assert_eq!(res.pages[0].base_highlights[0].op, DiffOpKind::Delete);
        // Tailored should have 0 highlights
        assert!(
            res.pages[0].tailored_highlights.is_empty(),
            "Expected 0 tailored highlights, got: {:?}",
            res.pages[0].tailored_highlights
        );
    }

    #[test]
    fn test_inline_multiple_discrete_word_edits() {
        // Base: "The red brown fox"
        let b0 = make_token("The", 50.0, 750.0, 70.0, 762.0, 0);
        let b1 = make_token("red", 75.0, 750.0, 95.0, 762.0, 0);
        let b2 = make_token("brown", 100.0, 750.0, 135.0, 762.0, 0);
        let b3 = make_token("fox", 140.0, 750.0, 160.0, 762.0, 0);

        // Tailored: "The quick blue fox"
        let t0 = make_token("The", 50.0, 750.0, 70.0, 762.0, 0);
        let t1 = make_token("quick", 75.0, 750.0, 105.0, 762.0, 0);
        let t2 = make_token("blue", 110.0, 750.0, 135.0, 762.0, 0);
        let t3 = make_token("fox", 140.0, 750.0, 160.0, 762.0, 0);

        let base_page = PageText::new(vec![b0, b1, b2, b3], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t0, t1, t2, t3], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // "red brown" is replaced by "quick blue" -> 1 merged delete span for base, 1 merged insert span for tailored
        assert_eq!(res.pages[0].base_highlights.len(), 1);
        assert_eq!(res.pages[0].tailored_highlights.len(), 1);
    }

    #[test]
    fn test_punctuation_insertion_only_highlights_punctuation() {
        // Base: "tests"
        let b0 = make_token("tests", 50.0, 750.0, 85.0, 762.0, 0);

        // Tailored: "tests,"
        let t0 = make_token("tests", 50.0, 750.0, 85.0, 762.0, 0);
        let t1 = make_token(",", 85.5, 750.0, 89.0, 762.0, 0);

        let base_page = PageText::new(vec![b0], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t0, t1], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base should have 0 highlights since "tests" is unchanged
        assert!(res.pages[0].base_highlights.is_empty());
        // Tailored should have exactly 1 highlight covering only the comma
        assert_eq!(res.pages[0].tailored_highlights.len(), 1);
        assert_eq!(res.pages[0].tailored_highlights[0].op, DiffOpKind::Insert);
        // Verify bounding box matches comma bounds expanded with padding
        assert!((res.pages[0].tailored_highlights[0].bounds.x0 - 85.0).abs() < 0.1);
        assert!((res.pages[0].tailored_highlights[0].bounds.x1 - 89.5).abs() < 0.1);
    }

    #[test]
    fn test_punctuation_deletion_only_highlights_punctuation() {
        // Base: "tests,"
        let b0 = make_token("tests", 50.0, 750.0, 85.0, 762.0, 0);
        let b1 = make_token(",", 85.5, 750.0, 89.0, 762.0, 0);

        // Tailored: "tests"
        let t0 = make_token("tests", 50.0, 750.0, 85.0, 762.0, 0);

        let base_page = PageText::new(vec![b0, b1], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t0], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base should have exactly 1 highlight covering only the deleted comma
        assert_eq!(res.pages[0].base_highlights.len(), 1);
        assert_eq!(res.pages[0].base_highlights[0].op, DiffOpKind::Delete);
        // Tailored should have 0 highlights
        assert!(res.pages[0].tailored_highlights.is_empty());
    }

    #[test]
    fn test_punctuation_replacement_preserves_word() {
        // Base: "tests."
        let b0 = make_token("tests", 50.0, 750.0, 85.0, 762.0, 0);
        let b1 = make_token(".", 85.5, 750.0, 88.0, 762.0, 0);

        // Tailored: "tests,"
        let t0 = make_token("tests", 50.0, 750.0, 85.0, 762.0, 0);
        let t1 = make_token(",", 85.5, 750.0, 89.0, 762.0, 0);

        let base_page = PageText::new(vec![b0, b1], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t0, t1], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base highlights only "."
        assert_eq!(res.pages[0].base_highlights.len(), 1);
        assert_eq!(res.pages[0].base_highlights[0].op, DiffOpKind::Delete);
        // Tailored highlights only ","
        assert_eq!(res.pages[0].tailored_highlights.len(), 1);
        assert_eq!(res.pages[0].tailored_highlights[0].op, DiffOpKind::Insert);
    }

    #[test]
    fn test_hyphenated_and_bracketed_punctuation_edits() {
        // Base: "(hello)"
        let b0 = make_token("(", 45.0, 750.0, 49.0, 762.0, 0);
        let b1 = make_token("hello", 50.0, 750.0, 85.0, 762.0, 0);
        let b2 = make_token(")", 86.0, 750.0, 90.0, 762.0, 0);

        // Tailored: "hello"
        let t0 = make_token("hello", 50.0, 750.0, 85.0, 762.0, 0);

        let base_page = PageText::new(vec![b0, b1, b2], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t0], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base highlights "(" and ")" as 2 separate deletion spans around untouched "hello"
        assert_eq!(res.pages[0].base_highlights.len(), 2);
        assert!(res.pages[0].tailored_highlights.is_empty());
    }

    #[test]
    fn test_phrase_replacement_with_punctuation_preservation() {
        // Base: "Built scalable APIs."
        let b0 = make_token("Built", 50.0, 750.0, 85.0, 762.0, 0);
        let b1 = make_token("scalable", 90.0, 750.0, 140.0, 762.0, 0);
        let b2 = make_token("APIs", 145.0, 750.0, 180.0, 762.0, 0);
        let b3 = make_token(".", 181.0, 750.0, 184.0, 762.0, 0);

        // Tailored: "Designed distributed systems."
        let t0 = make_token("Designed", 50.0, 750.0, 105.0, 762.0, 0);
        let t1 = make_token("distributed", 110.0, 750.0, 175.0, 762.0, 0);
        let t2 = make_token("systems", 180.0, 750.0, 225.0, 762.0, 0);
        let t3 = make_token(".", 226.0, 750.0, 229.0, 762.0, 0);

        let base_page = PageText::new(vec![b0, b1, b2, b3], 612.0, 792.0, 0);
        let tailored_page = PageText::new(vec![t0, t1, t2, t3], 612.0, 792.0, 0);

        let res = diff_documents(&[base_page], &[tailored_page], DiffGranularity::Word);
        assert!(res.has_differences);
        // Base has 1 merged highlight span covering "Built scalable APIs" (excluding ".")
        assert_eq!(res.pages[0].base_highlights.len(), 1);
        // Tailored has 1 merged highlight span covering "Designed distributed systems" (excluding ".")
        assert_eq!(res.pages[0].tailored_highlights.len(), 1);
    }
}
