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

    // Hierarchical diffing
    match granularity {
        DiffGranularity::Line => diff_by_lines_only(page_index, base_tokens, tailored_tokens),
        DiffGranularity::Word | DiffGranularity::Character => {
            diff_hierarchical(page_index, base_tokens, tailored_tokens)
        }
    }
}

/// Hierarchical two-pass diff: line-level matching followed by fine-grained token diff.
fn diff_hierarchical(
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
            similar::DiffOp::Equal { .. } => {
                // Unchanged lines: 0 highlights
            }
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                has_differences = true;
                for line in &base_lines[old_index..old_index + old_len] {
                    base_highlights.extend(merge_contiguous_highlights(
                        line.tokens,
                        DiffOpKind::Delete,
                        false,
                    ));
                }
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                has_differences = true;
                for line in &tailored_lines[new_index..new_index + new_len] {
                    tailored_highlights.extend(merge_contiguous_highlights(
                        line.tokens,
                        DiffOpKind::Insert,
                        false,
                    ));
                }
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                has_differences = true;
                let old_slice = &base_lines[old_index..old_index + old_len];
                let new_slice = &tailored_lines[new_index..new_index + new_len];

                // Pass 2: Token-level fine-grained diff across modified line blocks
                diff_modified_line_blocks(
                    old_slice,
                    new_slice,
                    &mut base_highlights,
                    &mut tailored_highlights,
                );
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

/// Computes token-level differences within modified line blocks and adds line tint + word highlights.
fn diff_modified_line_blocks(
    old_lines: &[LineGroup],
    new_lines: &[LineGroup],
    base_highlights: &mut Vec<HighlightSpan>,
    tailored_highlights: &mut Vec<HighlightSpan>,
) {
    // 1. Add subtle background tint across the modified lines
    for line in old_lines {
        if let Some(bounds) = compute_line_bounds(line.tokens) {
            base_highlights.push(HighlightSpan::new(bounds, DiffOpKind::Replace, false));
        }
    }
    for line in new_lines {
        if let Some(bounds) = compute_line_bounds(line.tokens) {
            tailored_highlights.push(HighlightSpan::new(bounds, DiffOpKind::Replace, false));
        }
    }

    // 2. Fine-grained word token diff
    let old_tokens: Vec<&TextToken> = old_lines.iter().flat_map(|l| l.tokens.iter()).collect();
    let new_tokens: Vec<&TextToken> = new_lines.iter().flat_map(|l| l.tokens.iter()).collect();

    let old_words: Vec<&str> = old_tokens
        .iter()
        .map(|t| t.normalized_text.as_str())
        .collect();
    let new_words: Vec<&str> = new_tokens
        .iter()
        .map(|t| t.normalized_text.as_str())
        .collect();

    let token_diffs = capture_diff_slices(Algorithm::Myers, &old_words, &new_words);

    for token_op in token_diffs {
        match token_op {
            similar::DiffOp::Equal { .. } => {}
            similar::DiffOp::Delete {
                old_index, old_len, ..
            } => {
                let deleted_slice = &old_tokens[old_index..old_index + old_len];
                base_highlights.extend(merge_token_ref_highlights(
                    deleted_slice,
                    DiffOpKind::Delete,
                    true,
                ));
            }
            similar::DiffOp::Insert {
                new_index, new_len, ..
            } => {
                let inserted_slice = &new_tokens[new_index..new_index + new_len];
                tailored_highlights.extend(merge_token_ref_highlights(
                    inserted_slice,
                    DiffOpKind::Insert,
                    true,
                ));
            }
            similar::DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let deleted_slice = &old_tokens[old_index..old_index + old_len];
                let inserted_slice = &new_tokens[new_index..new_index + new_len];

                base_highlights.extend(merge_token_ref_highlights(
                    deleted_slice,
                    DiffOpKind::Delete,
                    true,
                ));
                tailored_highlights.extend(merge_token_ref_highlights(
                    inserted_slice,
                    DiffOpKind::Insert,
                    true,
                ));
            }
        }
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

/// Convenience alias for merging borrowed token slices.
fn merge_token_ref_highlights(
    tokens: &[&TextToken],
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
}
