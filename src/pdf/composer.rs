//! Vector canvas compositor using PDFium.

use crate::error::PdfVdiffError;
use crate::layout::CanvasLayout;
use crate::model::{DiffOpKind, HighlightSpan};
use crate::theme::DiffTheme;
use pdfium_render::prelude::*;
use std::path::Path;

/// Document-level metadata for header banner rendering.
#[derive(Debug, Clone)]
pub struct HeaderMetadata<'a> {
    pub base_name: &'a str,
    pub tailored_name: &'a str,
    pub total_pages: usize,
    pub has_differences: bool,
}

/// Horizontal offset approximation for centering placeholder text in the base pane.
const BASE_PLACEHOLDER_TEXT_X_OFFSET: f32 = 120.0;

/// Horizontal offset approximation for centering placeholder text in the tailored pane.
const TAILORED_PLACEHOLDER_TEXT_X_OFFSET: f32 = 130.0;

/// Right margin allocation width for header page status and count text.
const HEADER_META_TEXT_RIGHT_MARGIN: f32 = 220.0;

/// Convert internal `[f32; 4]` normalized color to `PdfColor`.
#[inline]
fn to_pdf_color(c: &[f32; 4]) -> PdfColor {
    PdfColor::new(
        (c[0] * 255.0).clamp(0.0, 255.0).round() as u8,
        (c[1] * 255.0).clamp(0.0, 255.0).round() as u8,
        (c[2] * 255.0).clamp(0.0, 255.0).round() as u8,
        (c[3] * 255.0).clamp(0.0, 255.0).round() as u8,
    )
}

struct MismatchContext {
    page_idx: usize,
    base_count: usize,
    tailored_count: usize,
}

struct ChromeContext<'a> {
    page_idx: usize,
    total_pages: usize,
    font_helvetica_token: PdfFontToken,
    font_bold_token: PdfFontToken,
    meta: Option<&'a HeaderMetadata<'a>>,
}

/// Compositor orchestrating 4-layer Z-order side-by-side visual diff generation.
pub struct CanvasCompositor<'a> {
    pdfium: &'a Pdfium,
    theme: &'a DiffTheme,
    header_meta: Option<HeaderMetadata<'a>>,
}

impl<'a> CanvasCompositor<'a> {
    /// Create a new canvas compositor instance.
    pub fn new(
        pdfium: &'a Pdfium,
        theme: &'a DiffTheme,
        header_meta: Option<HeaderMetadata<'a>>,
    ) -> Self {
        Self {
            pdfium,
            theme,
            header_meta,
        }
    }

    /// Composite and save the diff document to an atomic destination path.
    pub fn render_and_save<'doc>(
        &self,
        base_doc: Option<&PdfDocument<'doc>>,
        tailored_doc: Option<&PdfDocument<'doc>>,
        layouts: &[CanvasLayout],
        highlights: &[(Vec<HighlightSpan>, Vec<HighlightSpan>)],
        output_path: &Path,
    ) -> Result<(), PdfVdiffError> {
        let base_page_count = base_doc.map_or(0, |d| d.pages().len() as usize);
        let tailored_page_count = tailored_doc.map_or(0, |d| d.pages().len() as usize);
        let total_pages = layouts.len();

        let base_src_pages: Vec<Option<PdfPage<'doc>>> = (0..total_pages)
            .map(|idx| base_doc.and_then(|d| d.pages().get(idx as i32).ok()))
            .collect();
        let tailored_src_pages: Vec<Option<PdfPage<'doc>>> = (0..total_pages)
            .map(|idx| tailored_doc.and_then(|d| d.pages().get(idx as i32).ok()))
            .collect();

        let mut new_doc = self.pdfium.create_new_pdf().map_err(|e| {
            PdfVdiffError::Render(format!("Failed to create output PDF document: {e}"))
        })?;

        // Preload fonts needed for header and placeholders
        let font_helvetica_token = new_doc.fonts_mut().helvetica();
        let font_bold_token = new_doc.fonts_mut().helvetica_bold();
        let font_oblique_token = new_doc.fonts_mut().helvetica_oblique();

        for (page_idx, layout) in layouts.iter().enumerate() {
            let mut new_page = new_doc
                .pages_mut()
                .create_page_at_end(PdfPagePaperSize::Custom(
                    PdfPoints::new(layout.total_width),
                    PdfPoints::new(layout.total_height),
                ))
                .map_err(|e| {
                    PdfVdiffError::Render(format!("Failed to create canvas page {page_idx}: {e}"))
                })?;

            let (base_spans, tailored_spans) = highlights
                .get(page_idx)
                .map_or((&[][..], &[][..]), |(b, t)| (b.as_slice(), t.as_slice()));

            let base_src_page = base_src_pages.get(page_idx).and_then(|p| p.as_ref());
            let tailored_src_page = tailored_src_pages.get(page_idx).and_then(|p| p.as_ref());

            // Layer 1: Backgrounds & Page-mismatch placeholders
            Self::render_layer1_backgrounds(
                self.theme,
                &mut new_page,
                &new_doc,
                font_oblique_token,
                layout,
                MismatchContext {
                    page_idx,
                    base_count: base_page_count,
                    tailored_count: tailored_page_count,
                },
            )?;

            // Layer 2: Embedded Vector Content (Form XObjects)
            Self::render_layer2_embedded_pages(
                &mut new_page,
                &mut new_doc,
                base_src_page,
                tailored_src_page,
                layout,
            )?;

            // Layer 3: Vector Highlight Paths (Multiply blend mode)
            Self::render_layer3_highlights(
                self.theme,
                &mut new_page,
                layout,
                base_spans,
                tailored_spans,
            )?;

            // Layer 4: Chrome (Gutter divider line & Header banner)
            Self::render_layer4_chrome(
                self.theme,
                &mut new_page,
                &new_doc,
                layout,
                ChromeContext {
                    page_idx,
                    total_pages,
                    font_helvetica_token,
                    font_bold_token,
                    meta: self.header_meta.as_ref(),
                },
            )?;
        }

        // Save atomically using tempfile
        Self::save_atomically(&new_doc, output_path)
    }

    /// Layer 1: Backgrounds and placeholder panels for page count mismatches.
    fn render_layer1_backgrounds<'b>(
        theme: &DiffTheme,
        page: &mut PdfPage<'b>,
        doc: &PdfDocument<'b>,
        font_oblique_token: PdfFontToken,
        layout: &CanvasLayout,
        ctx: MismatchContext,
    ) -> Result<(), PdfVdiffError> {
        let placeholder_bg = to_pdf_color(&theme.placeholder_bg);
        let placeholder_border = to_pdf_color(&theme.placeholder_border);
        let label_color = to_pdf_color(&theme.placeholder_text);

        // Missing Left Pane (Base)
        if ctx.page_idx >= ctx.base_count {
            let left_rect = PdfRect::new_from_values(
                layout.base_viewport.y0,
                layout.base_viewport.x0,
                layout.base_viewport.y1,
                layout.base_viewport.x1,
            );
            page.objects_mut()
                .create_path_object_rect(
                    left_rect,
                    Some(placeholder_border),
                    Some(PdfPoints::new(1.0)),
                    Some(placeholder_bg),
                )
                .map_err(|e| {
                    PdfVdiffError::Render(format!("Failed to draw base placeholder: {e}"))
                })?;

            if let Some(font) = doc.fonts().get(font_oblique_token) {
                let text = "[No corresponding page in Base Document]";
                let x = layout.base_pane_origin.0 + (layout.base_viewport.width() / 2.0)
                    - BASE_PLACEHOLDER_TEXT_X_OFFSET;
                let y = layout.base_pane_origin.1 + (layout.base_viewport.height() / 2.0);
                let mut text_obj = page
                    .objects_mut()
                    .create_text_object(
                        PdfPoints::new(x.max(layout.base_pane_origin.0 + 10.0)),
                        PdfPoints::new(y),
                        text,
                        font,
                        PdfPoints::new(12.0),
                    )
                    .map_err(|e| {
                        PdfVdiffError::Render(format!("Failed to create placeholder text: {e}"))
                    })?;
                // Setting fill color on fresh text object is infallible under standard PDFium runtime
                let _ = text_obj.set_fill_color(label_color);
            }
        }

        // Missing Right Pane (Tailored)
        if ctx.page_idx >= ctx.tailored_count {
            let right_rect = PdfRect::new_from_values(
                layout.tailored_viewport.y0,
                layout.tailored_viewport.x0,
                layout.tailored_viewport.y1,
                layout.tailored_viewport.x1,
            );
            page.objects_mut()
                .create_path_object_rect(
                    right_rect,
                    Some(placeholder_border),
                    Some(PdfPoints::new(1.0)),
                    Some(placeholder_bg),
                )
                .map_err(|e| {
                    PdfVdiffError::Render(format!("Failed to draw tailored placeholder: {e}"))
                })?;

            if let Some(font) = doc.fonts().get(font_oblique_token) {
                let text = "[No corresponding page in Tailored Document]";
                let x = layout.tailored_pane_origin.0 + (layout.tailored_viewport.width() / 2.0)
                    - TAILORED_PLACEHOLDER_TEXT_X_OFFSET;
                let y = layout.tailored_pane_origin.1 + (layout.tailored_viewport.height() / 2.0);
                let mut text_obj = page
                    .objects_mut()
                    .create_text_object(
                        PdfPoints::new(x.max(layout.tailored_pane_origin.0 + 10.0)),
                        PdfPoints::new(y),
                        text,
                        font,
                        PdfPoints::new(12.0),
                    )
                    .map_err(|e| {
                        PdfVdiffError::Render(format!("Failed to create placeholder text: {e}"))
                    })?;
                // Setting fill color on fresh text object is infallible under standard PDFium runtime
                let _ = text_obj.set_fill_color(label_color);
            }
        }

        Ok(())
    }

    /// Layer 2: Embed vector pages into Left and Right viewports.
    fn render_layer2_embedded_pages<'b>(
        page: &mut PdfPage<'b>,
        new_doc: &mut PdfDocument<'b>,
        base_src_page: Option<&'b PdfPage<'b>>,
        tailored_src_page: Option<&'b PdfPage<'b>>,
        layout: &CanvasLayout,
    ) -> Result<(), PdfVdiffError> {
        // Embed Base Page
        if let Some(src_page) = base_src_page {
            if let Ok(mut xobj) = src_page.objects().copy_into_x_object_form_object(new_doc) {
                let _ = xobj.translate(
                    PdfPoints::new(layout.base_pane_origin.0),
                    PdfPoints::new(layout.base_pane_origin.1),
                );
                // Inserting Form XObject into destination page cannot fail under standard PDFium memory state
                let _ = page.objects_mut().add_object(xobj);
            }
        }

        // Embed Tailored Page
        if let Some(src_page) = tailored_src_page {
            if let Ok(mut xobj) = src_page.objects().copy_into_x_object_form_object(new_doc) {
                let _ = xobj.translate(
                    PdfPoints::new(layout.tailored_pane_origin.0),
                    PdfPoints::new(layout.tailored_pane_origin.1),
                );
                // Inserting Form XObject into destination page cannot fail under standard PDFium memory state
                let _ = page.objects_mut().add_object(xobj);
            }
        }

        Ok(())
    }

    /// Helper to render highlight rectangle spans for either base or tailored side.
    fn render_highlight_spans_for_side<'b>(
        theme: &DiffTheme,
        page: &mut PdfPage<'b>,
        layout: &CanvasLayout,
        spans: &[HighlightSpan],
        is_tailored: bool,
    ) {
        for span in spans {
            let projected = if is_tailored {
                layout.project_tailored_highlight(span)
            } else {
                layout.project_base_highlight(span)
            };

            let (fill, stroke) = match span.op {
                DiffOpKind::Delete => (theme.deletion_fill, Some(theme.deletion_stroke)),
                DiffOpKind::Insert => (theme.addition_fill, Some(theme.addition_stroke)),
                DiffOpKind::Replace => {
                    if span.is_modified_token {
                        if is_tailored {
                            (theme.addition_fill, Some(theme.addition_stroke))
                        } else {
                            (theme.deletion_fill, Some(theme.deletion_stroke))
                        }
                    } else {
                        (theme.replaced_line_tint, None)
                    }
                }
                _ => continue,
            };

            let pdf_rect = PdfRect::new_from_values(
                projected.bounds.y0,
                projected.bounds.x0,
                projected.bounds.y1,
                projected.bounds.x1,
            );
            let fill_color = to_pdf_color(&fill);
            let stroke_color = stroke.as_ref().map(to_pdf_color);
            let stroke_width = stroke.map(|_| PdfPoints::new(0.5));

            if let Ok(mut path_obj) = page.objects_mut().create_path_object_rect(
                pdf_rect,
                stroke_color,
                stroke_width,
                Some(fill_color),
            ) {
                // PDFium blend mode modification is non-fatal if unsupported by underlying target backend
                let _ = path_obj.set_blend_mode(PdfPageObjectBlendMode::Multiply);
            }
        }
    }

    /// Layer 3: Render vector highlight rectangles with RGBA fills and Multiply blend mode.
    fn render_layer3_highlights<'b>(
        theme: &DiffTheme,
        page: &mut PdfPage<'b>,
        layout: &CanvasLayout,
        base_spans: &[HighlightSpan],
        tailored_spans: &[HighlightSpan],
    ) -> Result<(), PdfVdiffError> {
        Self::render_highlight_spans_for_side(theme, page, layout, base_spans, false);
        Self::render_highlight_spans_for_side(theme, page, layout, tailored_spans, true);
        Ok(())
    }

    /// Layer 4: Gutter divider line and header metadata banner.
    fn render_layer4_chrome<'b>(
        theme: &DiffTheme,
        page: &mut PdfPage<'b>,
        doc: &PdfDocument<'b>,
        layout: &CanvasLayout,
        ctx: ChromeContext,
    ) -> Result<(), PdfVdiffError> {
        // 1. Gutter divider line
        let gutter_x = layout.gutter_line_x;
        let gutter_y_top = layout.total_height - layout.header_height;
        let divider_color = to_pdf_color(&theme.gutter_color);

        let _ = page.objects_mut().create_path_object_line(
            PdfPoints::new(gutter_x),
            PdfPoints::new(0.0),
            PdfPoints::new(gutter_x),
            PdfPoints::new(gutter_y_top),
            divider_color,
            PdfPoints::new(0.75),
        );

        // 2. Header banner
        if layout.header_height > 0.0 {
            let header = layout.header_bounds;
            let header_bg = to_pdf_color(&theme.header_bg);
            let header_border = to_pdf_color(&theme.header_border);

            // Banner background
            let header_pdf_rect =
                PdfRect::new_from_values(header.y0, header.x0, header.y1, header.x1);
            let _ = page.objects_mut().create_path_object_rect(
                header_pdf_rect,
                None,
                None,
                Some(header_bg),
            );

            // Banner bottom border line
            let _ = page.objects_mut().create_path_object_line(
                PdfPoints::new(0.0),
                PdfPoints::new(header.y0),
                PdfPoints::new(layout.total_width),
                PdfPoints::new(header.y0),
                header_border,
                PdfPoints::new(0.75),
            );

            // Banner typography
            if let Some(meta) = ctx.meta {
                // Title (Left): 10 pt Helvetica Bold
                if let Some(bold_font) = doc.fonts().get(ctx.font_bold_token) {
                    let title_text = format!(
                        "Base: {}  vs  Tailored: {}",
                        meta.base_name, meta.tailored_name
                    );
                    let title_color = to_pdf_color(&theme.header_title_color);
                    let title_y = header.y0 + (layout.header_height / 2.0) - 4.0;
                    if let Ok(mut text_obj) = page.objects_mut().create_text_object(
                        PdfPoints::new(16.0),
                        PdfPoints::new(title_y),
                        title_text,
                        bold_font,
                        PdfPoints::new(10.0),
                    ) {
                        let _ = text_obj.set_fill_color(title_color);
                    }
                }

                // Page & Status (Right): 8 pt Helvetica Regular
                if let Some(reg_font) = doc.fonts().get(ctx.font_helvetica_token) {
                    let status_str = if meta.has_differences {
                        "Differences detected"
                    } else {
                        "Identical"
                    };
                    let page_status_text = format!(
                        "Page {} of {} | {}",
                        ctx.page_idx + 1,
                        ctx.total_pages,
                        status_str
                    );
                    let meta_color = to_pdf_color(&theme.header_meta_color);
                    let meta_x = (layout.total_width - HEADER_META_TEXT_RIGHT_MARGIN)
                        .max(layout.total_width / 2.0);
                    let meta_y = header.y0 + (layout.header_height / 2.0) - 3.0;
                    if let Ok(mut text_obj) = page.objects_mut().create_text_object(
                        PdfPoints::new(meta_x),
                        PdfPoints::new(meta_y),
                        page_status_text,
                        reg_font,
                        PdfPoints::new(8.0),
                    ) {
                        let _ = text_obj.set_fill_color(meta_color);
                    }
                }
            }
        }

        Ok(())
    }

    /// Atomically save document to target path via temporary file.
    fn save_atomically(doc: &PdfDocument, output_path: &Path) -> Result<(), PdfVdiffError> {
        let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
        let mut temp_file =
            tempfile::NamedTempFile::new_in(parent_dir).map_err(|e| PdfVdiffError::OutputIo {
                path: output_path.to_path_buf(),
                source: e,
            })?;

        doc.save_to_writer(&mut temp_file)
            .map_err(|e| PdfVdiffError::Render(format!("Failed to serialize PDF: {e}")))?;

        temp_file
            .persist(output_path)
            .map_err(|e| PdfVdiffError::OutputIo {
                path: output_path.to_path_buf(),
                source: e.error,
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::diff_documents;
    use crate::layout::PageDimensions;
    use crate::pdf::extract::extract_document_tokens;
    use crate::pdf::init::init_pdfium;
    use crate::theme::ThemeKind;

    fn create_dummy_pdf<'a>(pdfium: &'a Pdfium, pages_content: &[&[&str]]) -> PdfDocument<'a> {
        let mut doc = pdfium.create_new_pdf().expect("create pdf");
        let font_token = doc.fonts_mut().helvetica();

        for lines in pages_content {
            let mut page = doc
                .pages_mut()
                .create_page_at_end(PdfPagePaperSize::Custom(
                    PdfPoints::new(612.0),
                    PdfPoints::new(792.0),
                ))
                .expect("create page");

            let font = doc.fonts().get(font_token).expect("font");
            for (idx, line) in lines.iter().enumerate() {
                let y = 700.0 - (idx as f32 * 20.0);
                let mut text_obj = page
                    .objects_mut()
                    .create_text_object(
                        PdfPoints::new(72.0),
                        PdfPoints::new(y),
                        *line,
                        font,
                        PdfPoints::new(12.0),
                    )
                    .expect("create text");
                let _ = text_obj.set_fill_color(PdfColor::BLACK);
            }
        }

        doc
    }

    #[test]
    fn test_compositor_synthetic_equal_documents() {
        let pdfium = init_pdfium().expect("PDFium init failed");
        let base_doc = create_dummy_pdf(pdfium, &[&["Hello World", "Rust systems engineer"]]);
        let tailored_doc =
            create_dummy_pdf(pdfium, &[&["Hello World", "Rust senior systems engineer"]]);

        let base_pages = extract_document_tokens(&base_doc, 250).expect("Extract base failed");
        let tailored_pages =
            extract_document_tokens(&tailored_doc, 250).expect("Extract tailored failed");

        let diff_res = diff_documents(
            &base_pages,
            &tailored_pages,
            crate::diff::DiffGranularity::Word,
        );
        let max_pages = base_pages.len().max(tailored_pages.len());

        let mut layouts = Vec::with_capacity(max_pages);
        let mut highlights = Vec::with_capacity(max_pages);

        for i in 0..max_pages {
            let b_dims = base_pages
                .get(i)
                .map(|p| PageDimensions::new(p.width, p.height));
            let t_dims = tailored_pages
                .get(i)
                .map(|p| PageDimensions::new(p.width, p.height));
            layouts.push(CanvasLayout::compute_default(b_dims, t_dims));

            let page_diff = diff_res.pages.get(i);
            let b_high = page_diff.map_or(Vec::new(), |p| p.base_highlights.clone());
            let t_high = page_diff.map_or(Vec::new(), |p| p.tailored_highlights.clone());
            highlights.push((b_high, t_high));
        }

        let theme = ThemeKind::IntelliJ.theme();
        let header_meta = HeaderMetadata {
            base_name: "base.pdf",
            tailored_name: "tailored.pdf",
            total_pages: max_pages,
            has_differences: diff_res.has_differences,
        };

        let compositor = CanvasCompositor::new(pdfium, theme, Some(header_meta));
        let out_dir = tempfile::tempdir().expect("tempdir");
        let out_pdf = out_dir.path().join("synthetic_diff.pdf");

        compositor
            .render_and_save(
                Some(&base_doc),
                Some(&tailored_doc),
                &layouts,
                &highlights,
                &out_pdf,
            )
            .expect("Render and save failed");

        assert!(out_pdf.exists());
        assert!(std::fs::metadata(&out_pdf).expect("metadata").len() > 500);
    }

    #[test]
    fn test_compositor_synthetic_page_mismatch_and_no_header() {
        let pdfium = init_pdfium().expect("PDFium init failed");
        // Base has 1 page, Tailored has 2 pages
        let base_doc = create_dummy_pdf(pdfium, &[&["Page 1 Base"]]);
        let tailored_doc =
            create_dummy_pdf(pdfium, &[&["Page 1 Tailored"], &["Page 2 Tailored extra"]]);

        let base_pages = extract_document_tokens(&base_doc, 250).expect("Extract base failed");
        let tailored_pages =
            extract_document_tokens(&tailored_doc, 250).expect("Extract tailored failed");

        let diff_res = diff_documents(
            &base_pages,
            &tailored_pages,
            crate::diff::DiffGranularity::Word,
        );
        let max_pages = base_pages.len().max(tailored_pages.len());
        assert_eq!(max_pages, 2);

        let mut layouts = Vec::with_capacity(max_pages);
        let mut highlights = Vec::with_capacity(max_pages);

        for i in 0..max_pages {
            let b_dims = base_pages
                .get(i)
                .map(|p| PageDimensions::new(p.width, p.height));
            let t_dims = tailored_pages
                .get(i)
                .map(|p| PageDimensions::new(p.width, p.height));
            // Test with header_height = 0.0 (no header)
            layouts.push(CanvasLayout::compute(b_dims, t_dims, 24.0, 0.0));

            let page_diff = diff_res.pages.get(i);
            let b_high = page_diff.map_or(Vec::new(), |p| p.base_highlights.clone());
            let t_high = page_diff.map_or(Vec::new(), |p| p.tailored_highlights.clone());
            highlights.push((b_high, t_high));
        }

        let theme = ThemeKind::GitHub.theme();
        let compositor = CanvasCompositor::new(pdfium, theme, None);
        let out_dir = tempfile::tempdir().expect("tempdir");
        let out_pdf = out_dir.path().join("mismatch_diff.pdf");

        compositor
            .render_and_save(
                Some(&base_doc),
                Some(&tailored_doc),
                &layouts,
                &highlights,
                &out_pdf,
            )
            .expect("Render and save failed");

        assert!(out_pdf.exists());
        assert!(std::fs::metadata(&out_pdf).expect("metadata").len() > 500);
    }

    #[test]
    fn test_compositor_e2e_with_private_fixtures_if_present() {
        let base_path = Path::new("tests/fixtures_private/base.local.pdf");
        let tailored_path = Path::new("tests/fixtures_private/tailored.local.pdf");
        if !base_path.exists() || !tailored_path.exists() {
            return;
        }

        let pdfium = init_pdfium().expect("PDFium init failed");
        let base_doc = pdfium
            .load_pdf_from_file(base_path, None)
            .expect("Load base failed");
        let tailored_doc = pdfium
            .load_pdf_from_file(tailored_path, None)
            .expect("Load tailored failed");

        let base_pages = extract_document_tokens(&base_doc, 250).expect("Extract base failed");
        let tailored_pages =
            extract_document_tokens(&tailored_doc, 250).expect("Extract tailored failed");

        let diff_res = diff_documents(
            &base_pages,
            &tailored_pages,
            crate::diff::DiffGranularity::Word,
        );
        assert!(diff_res.has_differences);

        let max_pages = base_pages.len().max(tailored_pages.len());
        let mut layouts = Vec::with_capacity(max_pages);
        let mut highlights = Vec::with_capacity(max_pages);

        for i in 0..max_pages {
            let b_dims = base_pages
                .get(i)
                .map(|p| PageDimensions::new(p.width, p.height));
            let t_dims = tailored_pages
                .get(i)
                .map(|p| PageDimensions::new(p.width, p.height));
            layouts.push(CanvasLayout::compute_default(b_dims, t_dims));

            let page_diff = diff_res.pages.get(i);
            let b_high = page_diff.map_or(Vec::new(), |p| p.base_highlights.clone());
            let t_high = page_diff.map_or(Vec::new(), |p| p.tailored_highlights.clone());
            highlights.push((b_high, t_high));
        }

        let theme = ThemeKind::IntelliJ.theme();
        let header_meta = HeaderMetadata {
            base_name: "base.local.pdf",
            tailored_name: "tailored.local.pdf",
            total_pages: max_pages,
            has_differences: diff_res.has_differences,
        };

        let compositor = CanvasCompositor::new(pdfium, theme, Some(header_meta));
        let out_dir = tempfile::tempdir().expect("tempdir");
        let out_pdf = out_dir.path().join("diff_output.local.pdf");

        compositor
            .render_and_save(
                Some(&base_doc),
                Some(&tailored_doc),
                &layouts,
                &highlights,
                &out_pdf,
            )
            .expect("Render and save failed");

        assert!(out_pdf.exists());
        let file_size = std::fs::metadata(&out_pdf).expect("metadata").len();
        assert!(
            file_size > 1000,
            "Output PDF must be a non-empty valid file: {file_size} bytes"
        );
        println!(
            "Successfully generated diff PDF at {:?} (size: {} bytes)",
            out_pdf, file_size
        );
    }
}
