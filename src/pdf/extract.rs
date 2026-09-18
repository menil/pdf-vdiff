//! PDFium text extraction adapter.

use crate::error::PdfVdiffError;
use crate::model::{PageText, Rect, TextToken};
use pdfium_render::prelude::*;

/// Extract tokens from a document using PDFium.
pub fn extract_document_tokens(
    document: &PdfDocument,
    max_pages: usize,
) -> Result<Vec<PageText>, PdfVdiffError> {
    let page_count = document.pages().len() as usize;
    if page_count > max_pages {
        return Err(PdfVdiffError::PageCountExceeded(page_count, max_pages));
    }

    let mut pages = Vec::with_capacity(page_count);
    for (page_idx, page) in document.pages().iter().enumerate() {
        let page_text = extract_page_tokens(&page, page_idx)?;
        pages.push(page_text);
    }
    Ok(pages)
}

/// Determine if two adjacent single-character segments should be coalesced into a single word token.
#[inline]
pub fn should_coalesce_tokens(prev: &TextToken, token: &TextToken) -> bool {
    let same_baseline = (prev.bounds.y0 - token.bounds.y0).abs() <= 1.5;
    let adjacent_x =
        token.bounds.x0 >= prev.bounds.x1 - 1.0 && token.bounds.x0 <= prev.bounds.x1 + 3.0;
    !prev.trailing_space
        && same_baseline
        && adjacent_x
        && prev.text.chars().count() == 1
        && token.text.chars().count() == 1
}

/// Extract tokens from a single PDFium page.
pub fn extract_page_tokens(page: &PdfPage, page_index: usize) -> Result<PageText, PdfVdiffError> {
    let width = page.width().value;
    let height = page.height().value;

    let text_page = page.text().map_err(|e| {
        PdfVdiffError::Render(format!(
            "Failed to access text layer for page {page_index}: {e}"
        ))
    })?;

    let mut raw_tokens = Vec::new();

    // Iterate over segments (words / chunks)
    for segment in text_page.segments().iter() {
        let text = segment.text();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        let bounds = segment.bounds();
        let x0 = bounds.left().value;
        let y0 = bounds.bottom().value;
        let x1 = bounds.right().value;
        let y1 = bounds.top().value;

        // Ensure valid Rect invariant x0 <= x1 and y0 <= y1
        let min_x = x0.min(x1);
        let max_x = x0.max(x1);
        let min_y = y0.min(y1);
        let max_y = y0.max(y1);

        let rect = Rect::new(min_x, min_y, max_x, max_y);
        let trailing_space = text.ends_with(char::is_whitespace);

        raw_tokens.push(TextToken {
            text: trimmed.to_string(),
            normalized_text: crate::cluster::normalize_token_text(trimmed),
            bounds: rect,
            page_index,
            column_index: 0,
            line_index: 0,
            trailing_space,
        });
    }

    // Coalesce single-character segments into words if they are directly adjacent on the same baseline
    let mut tokens: Vec<TextToken> = Vec::with_capacity(raw_tokens.len());
    for token in raw_tokens {
        if let Some(prev) = tokens.last_mut() {
            if should_coalesce_tokens(prev, &token) {
                prev.text.push_str(&token.text);
                prev.normalized_text.push_str(&token.normalized_text);
                prev.bounds = prev.bounds.union(&token.bounds);
                prev.trailing_space = token.trailing_space;
                continue;
            }
        }
        tokens.push(token);
    }

    Ok(PageText {
        page_index,
        width,
        height,
        tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::init::init_pdfium;
    use std::path::Path;

    #[test]
    fn test_extract_synthetic_document() {
        let pdfium = init_pdfium().expect("PDFium init failed");
        let mut doc = pdfium.create_new_pdf().expect("create pdf");
        let font_token = doc.fonts_mut().helvetica();

        let mut page = doc
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::Custom(
                PdfPoints::new(612.0),
                PdfPoints::new(792.0),
            ))
            .expect("create page");

        let font = doc.fonts().get(font_token).expect("font");
        let mut text_obj = page
            .objects_mut()
            .create_text_object(
                PdfPoints::new(72.0),
                PdfPoints::new(700.0),
                "Software Engineer Resume",
                font,
                PdfPoints::new(14.0),
            )
            .expect("create text");
        let _ = text_obj.set_fill_color(PdfColor::BLACK);

        let extracted = extract_document_tokens(&doc, 250).expect("extract");
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].width, 612.0);
        assert_eq!(extracted[0].height, 792.0);
        assert!(!extracted[0].tokens.is_empty());
    }

    #[test]
    fn test_extract_page_count_exceeded() {
        let pdfium = init_pdfium().expect("PDFium init failed");
        let mut doc = pdfium.create_new_pdf().expect("create pdf");

        for _ in 0..3 {
            let _ = doc
                .pages_mut()
                .create_page_at_end(PdfPagePaperSize::Custom(
                    PdfPoints::new(612.0),
                    PdfPoints::new(792.0),
                ))
                .expect("create page");
        }

        let err = extract_document_tokens(&doc, 2).expect_err("should exceed limit");
        match err {
            PdfVdiffError::PageCountExceeded(count, max) => {
                assert_eq!(count, 3);
                assert_eq!(max, 2);
            }
            _ => panic!("Expected PageCountExceeded, got {:?}", err),
        }
    }

    #[test]
    fn test_should_coalesce_tokens() {
        let t1 = TextToken {
            text: "A".to_string(),
            normalized_text: "a".to_string(),
            bounds: Rect::new(10.0, 100.0, 18.0, 110.0),
            page_index: 0,
            column_index: 0,
            line_index: 0,
            trailing_space: false,
        };
        let t2 = TextToken {
            text: "B".to_string(),
            normalized_text: "b".to_string(),
            bounds: Rect::new(18.5, 100.2, 26.0, 110.0),
            page_index: 0,
            column_index: 0,
            line_index: 0,
            trailing_space: false,
        };
        assert!(should_coalesce_tokens(&t1, &t2));

        let mut t1_spaced = t1.clone();
        t1_spaced.trailing_space = true;
        assert!(!should_coalesce_tokens(&t1_spaced, &t2));

        let mut t2_far = t2.clone();
        t2_far.bounds = Rect::new(30.0, 100.0, 38.0, 110.0);
        assert!(!should_coalesce_tokens(&t1, &t2_far));

        let mut t2_diff_line = t2.clone();
        t2_diff_line.bounds = Rect::new(18.5, 120.0, 26.0, 130.0);
        assert!(!should_coalesce_tokens(&t1, &t2_diff_line));
    }

    #[test]
    fn test_extract_from_private_fixture_if_present() {
        let private_base = Path::new("tests/fixtures_private/base.local.pdf");
        if !private_base.exists() {
            return;
        }

        let pdfium = init_pdfium().expect("PDFium init failed");
        let doc = pdfium
            .load_pdf_from_file(private_base, None)
            .expect("Load PDF failed");
        let pages = extract_document_tokens(&doc, 250).expect("Extraction failed");

        assert!(!pages.is_empty());
        assert!(!pages[0].tokens.is_empty());
        println!("Page 0 coalesced tokens count: {}", pages[0].tokens.len());
    }
}
