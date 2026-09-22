//! Unicode normalization, spatial column clustering, and reading-order reconstruction.

use crate::model::TextToken;
use crate::theme::{
    DEFAULT_BASELINE_TOLERANCE, DEFAULT_COLUMN_GUTTER_THRESHOLD, MIN_BASELINE_GLYPH_HEIGHT,
};
use unicode_normalization::UnicodeNormalization;

/// Normalizes text using Unicode NFKD and decomposes standard typographical ligatures and quotes.
pub fn normalize_token_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    for ch in input.nfkd() {
        match ch {
            // Typographical quotes to ASCII
            '“' | '”' | '″' => normalized.push('"'),
            '‘' | '’' | '′' | '`' => normalized.push('\''),
            // Typographical dashes to ASCII hyphen
            '—' | '–' | '―' | '−' => normalized.push('-'),
            // Non-breaking and special spaces to standard ASCII space
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2009}' | '\u{202F}' => normalized.push(' '),
            other => normalized.push(other),
        }
    }
    normalized
}

/// A line cluster containing tokens sharing a common visual baseline within a column.
#[derive(Debug, Clone)]
struct LineCluster {
    y_center: f32,
    y0: f32,
    y1: f32,
    tokens: Vec<usize>, // Token indices
}

#[inline]
fn token_matches_line(line: &LineCluster, t: &TextToken, baseline_tolerance: f32) -> bool {
    let y_center = (t.bounds.y0 + t.bounds.y1) / 2.0;
    if (line.y_center - y_center).abs() <= baseline_tolerance {
        return true;
    }
    // Check baseline proximity and vertical overlap for punctuation / sub-height glyphs
    let same_baseline = (line.y0 - t.bounds.y0).abs() <= baseline_tolerance * 1.5;
    let overlaps = t.bounds.y0 <= line.y1 && t.bounds.y1 >= line.y0;
    same_baseline || overlaps
}

/// Clusters and sorts text tokens on a page into natural visual reading order.
///
/// Pipeline:
/// 1. Partitions tokens into columns using horizontal gap detection ($O(T \log T)$).
/// 2. Within each column, clusters tokens into visual lines based on baseline vertical tolerance.
/// 3. Sorts columns left-to-right, lines top-to-bottom ($Y$ descending in PDF coordinates),
///    and tokens within each line left-to-right ($X$ ascending).
/// 4. Updates `column_index` and `line_index` on each `TextToken`.
pub fn cluster_and_sort_tokens(
    tokens: &mut [TextToken],
    gutter_threshold: f32,
    baseline_tolerance: f32,
) {
    if tokens.is_empty() {
        return;
    }

    // Step 1: Normalize all token text strings for diffing
    for token in tokens.iter_mut() {
        token.normalized_text = normalize_token_text(&token.text);
    }

    // Step 2: Detect vertical columns via horizontal gap projection
    let column_assignments = detect_columns(tokens, gutter_threshold);
    for (idx, &col) in column_assignments.iter().enumerate() {
        tokens[idx].column_index = col;
    }

    // Step 3: Group and sort per column using bucketed column indices
    let max_column = tokens.iter().map(|t| t.column_index).max().unwrap_or(0);
    let mut column_buckets: Vec<Vec<usize>> = vec![Vec::new(); max_column + 1];
    for (idx, token) in tokens.iter().enumerate() {
        column_buckets[token.column_index].push(idx);
    }

    let mut global_sorted_tokens = Vec::with_capacity(tokens.len());
    let mut global_line_counter = 0;

    for col_token_indices in column_buckets {
        if col_token_indices.is_empty() {
            continue;
        }

        // Cluster tokens in this column into lines
        let mut lines = cluster_column_lines(tokens, &col_token_indices, baseline_tolerance);

        // Sort lines top-to-bottom (Y descending in PDF user space)
        lines.sort_by(|a, b| {
            b.y_center
                .partial_cmp(&a.y_center)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Within each line, sort tokens left-to-right (X ascending)
        for line in lines.iter_mut() {
            line.tokens.sort_by(|&i, &j| {
                tokens[i]
                    .bounds
                    .x0
                    .partial_cmp(&tokens[j].bounds.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for &token_idx in &line.tokens {
                tokens[token_idx].line_index = global_line_counter;
                global_sorted_tokens.push(tokens[token_idx].clone());
            }
            global_line_counter += 1;
        }
    }

    // Move sorted tokens back into target slice in-place without slice reallocation
    for (dst, src) in tokens.iter_mut().zip(global_sorted_tokens) {
        *dst = src;
    }
}

/// Boundary tolerance in points to handle sub-pixel glyph rounding against column bands.
const COLUMN_EDGE_TOLERANCE: f32 = 1.0;

/// Detects distinct horizontal columns by finding horizontal intervals separated by gaps > `gutter_threshold`.
fn detect_columns(tokens: &[TextToken], gutter_threshold: f32) -> Vec<usize> {
    if tokens.is_empty() {
        return Vec::new();
    }

    // Extract horizontal intervals [x0, x1]
    let mut x_intervals: Vec<(f32, f32)> =
        tokens.iter().map(|t| (t.bounds.x0, t.bounds.x1)).collect();
    x_intervals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // Merge overlapping/near intervals to find solid column bands
    let mut column_bands: Vec<(f32, f32)> = Vec::new();
    for (x0, x1) in x_intervals {
        if let Some(last) = column_bands.last_mut() {
            if x0 <= last.1 + gutter_threshold {
                last.1 = last.1.max(x1);
                continue;
            }
        }
        column_bands.push((x0, x1));
    }

    // Assign column index based on which band contains the token's center X
    tokens
        .iter()
        .map(|token| {
            let x_center = (token.bounds.x0 + token.bounds.x1) / 2.0;
            column_bands
                .iter()
                .position(|band| {
                    x_center >= band.0 - COLUMN_EDGE_TOLERANCE
                        && x_center <= band.1 + COLUMN_EDGE_TOLERANCE
                })
                .unwrap_or(0)
        })
        .collect()
}

/// Groups tokens within a single column into visual lines based on vertical baseline center proximity.
fn cluster_column_lines(
    tokens: &[TextToken],
    col_token_indices: &[usize],
    baseline_tolerance: f32,
) -> Vec<LineCluster> {
    let mut lines: Vec<LineCluster> = Vec::new();

    for &token_idx in col_token_indices {
        let t = &tokens[token_idx];
        let y_center = (t.bounds.y0 + t.bounds.y1) / 2.0;

        // Check the most recently modified line first (fast path for reading-order runs)
        let matching_line = if let Some(last) = lines.last_mut() {
            if token_matches_line(last, t, baseline_tolerance) {
                Some(last)
            } else {
                lines
                    .iter_mut()
                    .rev()
                    .skip(1)
                    .find(|line| token_matches_line(line, t, baseline_tolerance))
            }
        } else {
            None
        };

        if let Some(line) = matching_line {
            line.tokens.push(token_idx);
            line.y0 = line.y0.min(t.bounds.y0);
            line.y1 = line.y1.max(t.bounds.y1);
            if t.bounds.height() > MIN_BASELINE_GLYPH_HEIGHT {
                let count = line.tokens.len() as f32;
                line.y_center = ((line.y_center * (count - 1.0)) + y_center) / count;
            }
        } else {
            lines.push(LineCluster {
                y_center,
                y0: t.bounds.y0,
                y1: t.bounds.y1,
                tokens: vec![token_idx],
            });
        }
    }

    lines
}

/// Convenience function to cluster tokens with default thresholds.
pub fn cluster_tokens_default(tokens: &mut [TextToken]) {
    cluster_and_sort_tokens(
        tokens,
        DEFAULT_COLUMN_GUTTER_THRESHOLD,
        DEFAULT_BASELINE_TOLERANCE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Rect;

    #[test]
    fn test_unicode_normalization() {
        // Ligatures
        assert_eq!(normalize_token_text("ﬁle"), "file");
        assert_eq!(normalize_token_text("ﬂow"), "flow");
        assert_eq!(normalize_token_text("oﬃce"), "office");

        // Quotes and dashes
        assert_eq!(normalize_token_text("“smart quotes”"), "\"smart quotes\"");
        assert_eq!(normalize_token_text("‘single’"), "'single'");
        assert_eq!(normalize_token_text("word—break"), "word-break");
    }

    #[test]
    fn test_single_column_sorting() {
        // 3 tokens out of order in stream
        let mut tokens = vec![
            TextToken::new(
                "Line2".to_string(),
                "Line2".to_string(),
                Rect::new(50.0, 700.0, 100.0, 712.0),
                0,
                0,
                0,
                true,
            ),
            TextToken::new(
                "World".to_string(),
                "World".to_string(),
                Rect::new(90.0, 750.0, 130.0, 762.0),
                0,
                0,
                0,
                false,
            ),
            TextToken::new(
                "Hello".to_string(),
                "Hello".to_string(),
                Rect::new(50.0, 750.0, 85.0, 762.0),
                0,
                0,
                0,
                true,
            ),
        ];

        cluster_tokens_default(&mut tokens);

        assert_eq!(tokens[0].text, "Hello");
        assert_eq!(tokens[0].line_index, 0);

        assert_eq!(tokens[1].text, "World");
        assert_eq!(tokens[1].line_index, 0);

        assert_eq!(tokens[2].text, "Line2");
        assert_eq!(tokens[2].line_index, 1);
    }

    #[test]
    fn test_multi_column_sorting() {
        // Sidebar (Left column: x in [50, 150]) and Main Body (Right column: x in [200, 500])
        // Stream order is deliberately interleaved: Right Col Line 1, Left Col Line 1, Right Col Line 2, Left Col Line 2
        let mut tokens = vec![
            TextToken::new(
                "Right-1".to_string(),
                "Right-1".to_string(),
                Rect::new(200.0, 750.0, 250.0, 762.0),
                0,
                0,
                0,
                true,
            ),
            TextToken::new(
                "Left-1".to_string(),
                "Left-1".to_string(),
                Rect::new(50.0, 750.0, 90.0, 762.0),
                0,
                0,
                0,
                true,
            ),
            TextToken::new(
                "Right-2".to_string(),
                "Right-2".to_string(),
                Rect::new(200.0, 700.0, 250.0, 712.0),
                0,
                0,
                0,
                true,
            ),
            TextToken::new(
                "Left-2".to_string(),
                "Left-2".to_string(),
                Rect::new(50.0, 700.0, 90.0, 712.0),
                0,
                0,
                0,
                true,
            ),
        ];

        cluster_tokens_default(&mut tokens);

        // Should be sorted: Left column first (top to bottom), then Right column (top to bottom)
        assert_eq!(tokens[0].text, "Left-1");
        assert_eq!(tokens[0].column_index, 0);

        assert_eq!(tokens[1].text, "Left-2");
        assert_eq!(tokens[1].column_index, 0);

        assert_eq!(tokens[2].text, "Right-1");
        assert_eq!(tokens[2].column_index, 1);

        assert_eq!(tokens[3].text, "Right-2");
        assert_eq!(tokens[3].column_index, 1);
    }

    #[test]
    fn test_empty_tokens() {
        let mut tokens = Vec::new();
        cluster_tokens_default(&mut tokens);
        assert!(tokens.is_empty());
    }
}
