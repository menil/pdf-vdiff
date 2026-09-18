//! Canvas layout geometry, coordinate mapping, and pane origin transformations.

use crate::model::{HighlightSpan, Rect};
use crate::theme::{DEFAULT_GUTTER_WIDTH, DEFAULT_HEADER_HEIGHT};

/// Width and height of a PDF page in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageDimensions {
    pub width: f32,
    pub height: f32,
}

impl PageDimensions {
    /// Creates a new `PageDimensions` instance.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Standard US Letter dimensions (612 x 792 pt).
    pub fn us_letter() -> Self {
        Self::new(612.0, 792.0)
    }

    /// Standard A4 dimensions (595.28 x 841.89 pt).
    pub fn a4() -> Self {
        Self::new(595.28, 841.89)
    }
}

/// Calculated composite landscape canvas layout for a side-by-side page pair.
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasLayout {
    /// Total width of the composite canvas in PDF points.
    pub total_width: f32,
    /// Total height of the composite canvas in PDF points.
    pub total_height: f32,
    /// Height allocated to the top header banner.
    pub header_height: f32,
    /// Gutter width in points between left and right viewports.
    pub gutter_width: f32,
    /// Origin (X0, Y0) of the Left (Base) viewport in canvas coordinates.
    pub base_pane_origin: (f32, f32),
    /// Origin (X0, Y0) of the Right (Tailored) viewport in canvas coordinates.
    pub tailored_pane_origin: (f32, f32),
    /// Bounding box of the Left (Base) page on the canvas.
    pub base_viewport: Rect,
    /// Bounding box of the Right (Tailored) page on the canvas.
    pub tailored_viewport: Rect,
    /// Bounding box of the top header banner.
    pub header_bounds: Rect,
    /// X coordinate of the vertical gutter dividing line.
    pub gutter_line_x: f32,
}

impl CanvasLayout {
    /// Computes the complete composite canvas layout for a page pair.
    ///
    /// Invariants:
    /// - Dimensions: $W_{out} = W_b + W_t + \text{Gutter}$, $H_{out} = \max(H_b, H_t) + H_{header}$
    /// - Top-Alignment: Source pages are top-aligned directly below the header banner ($Y = \max(H_b, H_t)$)
    /// - Bottom-Left Origin: $(X_0, Y_0) = (0, \max(H_b, H_t) - H_b)$ for Left, $(W_b + \text{Gutter}, \max(H_b, H_t) - H_t)$ for Right
    pub fn compute(
        base_dim: Option<PageDimensions>,
        tailored_dim: Option<PageDimensions>,
        gutter_width: f32,
        header_height: f32,
    ) -> Self {
        // Fallback to the other document's size if one side is missing, or US Letter if both missing
        let default_dim = base_dim
            .or(tailored_dim)
            .unwrap_or_else(PageDimensions::us_letter);
        let b = base_dim.unwrap_or(default_dim);
        let t = tailored_dim.unwrap_or(default_dim);

        let max_content_height = b.height.max(t.height);
        let total_width = b.width + t.width + gutter_width;
        let total_height = max_content_height + header_height;

        // In PDF coordinates with bottom-left origin, top alignment directly below the header means:
        // Page Top Y = max_content_height
        // Origin Y = max_content_height - Page Height
        let base_y0 = max_content_height - b.height;
        let tailored_y0 = max_content_height - t.height;

        let base_x0 = 0.0;
        let tailored_x0 = b.width + gutter_width;

        let base_viewport = Rect::new(base_x0, base_y0, base_x0 + b.width, base_y0 + b.height);
        let tailored_viewport = Rect::new(
            tailored_x0,
            tailored_y0,
            tailored_x0 + t.width,
            tailored_y0 + t.height,
        );

        let header_bounds = Rect::new(0.0, max_content_height, total_width, total_height);

        // Position the vertical gutter divider line at the exact midpoint (gutter_width / 2.0)
        // between the base (left) and tailored (right) page viewports.
        let gutter_line_x = b.width + (gutter_width / 2.0);

        Self {
            total_width,
            total_height,
            header_height,
            gutter_width,
            base_pane_origin: (base_x0, base_y0),
            tailored_pane_origin: (tailored_x0, tailored_y0),
            base_viewport,
            tailored_viewport,
            header_bounds,
            gutter_line_x,
        }
    }

    /// Computes layout with default gutter and header settings.
    pub fn compute_default(
        base_dim: Option<PageDimensions>,
        tailored_dim: Option<PageDimensions>,
    ) -> Self {
        Self::compute(
            base_dim,
            tailored_dim,
            DEFAULT_GUTTER_WIDTH,
            DEFAULT_HEADER_HEIGHT,
        )
    }

    /// Projects a rectangle from Left (Base) source page space into composite canvas space.
    pub fn project_base_rect(&self, r: Rect) -> Rect {
        Rect::new(
            r.x0 + self.base_pane_origin.0,
            r.y0 + self.base_pane_origin.1,
            r.x1 + self.base_pane_origin.0,
            r.y1 + self.base_pane_origin.1,
        )
    }

    /// Projects a rectangle from Right (Tailored) source page space into composite canvas space.
    pub fn project_tailored_rect(&self, r: Rect) -> Rect {
        Rect::new(
            r.x0 + self.tailored_pane_origin.0,
            r.y0 + self.tailored_pane_origin.1,
            r.x1 + self.tailored_pane_origin.0,
            r.y1 + self.tailored_pane_origin.1,
        )
    }

    /// Projects a Left (Base) highlight span into canvas space.
    pub fn project_base_highlight(&self, span: &HighlightSpan) -> HighlightSpan {
        HighlightSpan::new(
            self.project_base_rect(span.bounds),
            span.op,
            span.is_modified_token,
        )
    }

    /// Projects a Right (Tailored) highlight span into canvas space.
    pub fn project_tailored_highlight(&self, span: &HighlightSpan) -> HighlightSpan {
        HighlightSpan::new(
            self.project_tailored_rect(span.bounds),
            span.op,
            span.is_modified_token,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DiffOpKind;

    #[test]
    fn test_equal_page_dimensions() {
        let dim = PageDimensions::new(600.0, 800.0);
        let layout = CanvasLayout::compute(Some(dim), Some(dim), 20.0, 40.0);

        assert_eq!(layout.total_width, 1220.0); // 600 + 600 + 20
        assert_eq!(layout.total_height, 840.0); // 800 + 40
        assert_eq!(layout.base_pane_origin, (0.0, 0.0));
        assert_eq!(layout.tailored_pane_origin, (620.0, 0.0));
        assert_eq!(layout.gutter_line_x, 610.0);
        assert_eq!(layout.header_bounds, Rect::new(0.0, 800.0, 1220.0, 840.0));
    }

    #[test]
    fn test_unequal_page_dimensions_top_alignment() {
        let base = PageDimensions::new(600.0, 700.0);
        let tailored = PageDimensions::new(600.0, 800.0); // Taller page
        let layout = CanvasLayout::compute(Some(base), Some(tailored), 20.0, 30.0);

        assert_eq!(layout.total_height, 830.0); // 800 + 30
                                                // Top alignment invariant:
                                                // Base top must be at Y = 800.0 (below header) -> Base Y0 = 800 - 700 = 100.0
        assert_eq!(layout.base_pane_origin, (0.0, 100.0));
        assert_eq!(layout.tailored_pane_origin, (620.0, 0.0));

        let local_rect = Rect::new(50.0, 600.0, 150.0, 650.0);
        let projected = layout.project_base_rect(local_rect);
        assert_eq!(projected, Rect::new(50.0, 700.0, 150.0, 750.0));
    }

    #[test]
    fn test_highlight_projection() {
        let dim = PageDimensions::us_letter();
        let layout = CanvasLayout::compute_default(Some(dim), Some(dim));

        let span = HighlightSpan::new(Rect::new(10.0, 20.0, 30.0, 40.0), DiffOpKind::Insert, false);

        let proj = layout.project_tailored_highlight(&span);
        assert_eq!(proj.bounds.x0, 10.0 + 612.0 + DEFAULT_GUTTER_WIDTH);
        assert_eq!(proj.op, DiffOpKind::Insert);
    }
}
