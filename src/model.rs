//! Core data models and geometric types for `pdf-vdiff`.

/// 2D Bounding box in PDF user-space points with bottom-left origin.
/// Invariant: x0 <= x1 and y0 <= y1 (enforced by construction in constructor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x0: f32, // Left
    pub y0: f32, // Bottom
    pub x1: f32, // Right
    pub y1: f32, // Top
}

impl Rect {
    /// Creates a new bounding box, ensuring `x0 <= x1` and `y0 <= y1`.
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    /// Returns the width of the bounding box.
    pub fn width(&self) -> f32 {
        (self.x1 - self.x0).max(0.0)
    }

    /// Returns the height of the bounding box.
    pub fn height(&self) -> f32 {
        (self.y1 - self.y0).max(0.0)
    }

    /// Computes the minimal bounding box containing both `self` and `other`.
    pub fn union(&self, other: &Rect) -> Rect {
        Rect {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    /// Expands the bounding box by `pad_x` horizontally and `pad_y` vertically.
    pub fn expand(&self, pad_x: f32, pad_y: f32) -> Rect {
        Rect {
            x0: self.x0 - pad_x,
            y0: self.y0 - pad_y,
            x1: self.x1 + pad_x,
            y1: self.y1 + pad_y,
        }
    }

    /// Checks if a point is within this bounding box.
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x0 && x <= self.x1 && y >= self.y0 && y <= self.y1
    }

    /// Checks if this rectangle overlaps with another rectangle.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && self.y0 < other.y1 && self.y1 > other.y0
    }
}

/// Extracted text token with geometric and structural metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct TextToken {
    /// Raw extracted text of the token.
    pub text: String,
    /// NFKD normalized text used for sequence matching and diffing.
    pub normalized_text: String,
    /// CropBox-normalized bounding box in PDF user points.
    pub bounds: Rect,
    /// 0-indexed source page index.
    pub page_index: usize,
    /// Spatial column partition index.
    pub column_index: usize,
    /// Visual baseline line index within the column.
    pub line_index: usize,
    /// True if whitespace originally followed this token in the document.
    pub trailing_space: bool,
}

impl TextToken {
    /// Constructs a new `TextToken`.
    pub fn new(
        text: String,
        normalized_text: String,
        bounds: Rect,
        page_index: usize,
        column_index: usize,
        line_index: usize,
        trailing_space: bool,
    ) -> Self {
        Self {
            text,
            normalized_text,
            bounds,
            page_index,
            column_index,
            line_index,
            trailing_space,
        }
    }
}

/// Category of a diff operation on text or lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffOpKind {
    /// Content is identical across both documents.
    Equal,
    /// Content is deleted from the Base document.
    Delete,
    /// Content is inserted into the Tailored document.
    Insert,
    /// Content is modified/replaced between documents.
    Replace,
}

/// Highlight rectangle segment rendered on the composite canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct HighlightSpan {
    /// Canvas-projected bounding box.
    pub bounds: Rect,
    /// Kind of diff operation.
    pub op: DiffOpKind,
    /// True if this highlight marks a specific altered word in a modified line.
    pub is_modified_token: bool,
}

impl HighlightSpan {
    /// Constructs a new `HighlightSpan`.
    pub fn new(bounds: Rect, op: DiffOpKind, is_modified_token: bool) -> Self {
        Self {
            bounds,
            op,
            is_modified_token,
        }
    }
}

/// Structured representation of all text extracted from a single PDF page.
#[derive(Debug, Clone, PartialEq)]
pub struct PageText {
    /// Extracted tokens sorted in natural visual reading order.
    pub tokens: Vec<TextToken>,
    /// Page width in PDF points.
    pub width: f32,
    /// Page height in PDF points.
    pub height: f32,
    /// 0-indexed page number.
    pub page_index: usize,
}

impl PageText {
    /// Constructs a new `PageText`.
    pub fn new(tokens: Vec<TextToken>, width: f32, height: f32, page_index: usize) -> Self {
        Self {
            tokens,
            width,
            height,
            page_index,
        }
    }

    /// Returns true if the page contains zero text tokens.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_math() {
        let r1 = Rect::new(10.0, 20.0, 50.0, 60.0);
        assert_eq!(r1.width(), 40.0);
        assert_eq!(r1.height(), 40.0);
        assert!(r1.contains_point(30.0, 40.0));
        assert!(!r1.contains_point(5.0, 40.0));

        let r2 = Rect::new(40.0, 50.0, 80.0, 90.0);
        assert!(r1.overlaps(&r2));

        let u = r1.union(&r2);
        assert_eq!(u, Rect::new(10.0, 20.0, 80.0, 90.0));

        let exp = r1.expand(0.5, 1.0);
        assert_eq!(exp, Rect::new(9.5, 19.0, 50.5, 61.0));
    }

    #[test]
    fn test_rect_new_inverted_coordinates() {
        let r = Rect::new(50.0, 60.0, 10.0, 20.0);
        assert_eq!(
            r,
            Rect {
                x0: 10.0,
                y0: 20.0,
                x1: 50.0,
                y1: 60.0,
            }
        );
        assert_eq!(r.width(), 40.0);
        assert_eq!(r.height(), 40.0);
    }

    #[test]
    fn test_token_and_page_text() {
        let rect = Rect::new(0.0, 0.0, 100.0, 20.0);
        let token = TextToken::new(
            "Hello".to_string(),
            "Hello".to_string(),
            rect,
            0,
            0,
            0,
            true,
        );
        assert_eq!(token.text, "Hello");
        assert!(token.trailing_space);

        let page = PageText::new(vec![token], 612.0, 792.0, 0);
        assert!(!page.is_empty());
        assert_eq!(page.tokens.len(), 1);
    }
}
