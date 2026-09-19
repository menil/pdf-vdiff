//! Synthetic test fixture generator and size validation suite.
//!
//! Generates reproducible, lightweight (< 15 KB) synthetic PDF fixtures
//! covering critical visual diffing edge cases: identical documents, inline
//! word/line edits, multi-column reading orders, asymmetric page counts,
//! and scanned/image-only PDFs lacking text layers.

use pdf_vdiff::pdf::init_pdfium;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};

/// Returns the path to the committed synthetic test fixtures directory (`tests/fixtures`).
fn get_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Creates a new PDF document with `num_pages` using standard US Letter dimensions (612 x 792 pt).
fn create_pdf_with_pages<'a, F>(
    pdfium: &'a Pdfium,
    num_pages: usize,
    mut draw_page: F,
) -> PdfDocument<'a>
where
    F: FnMut(usize, &mut PdfPage<'a>, &PdfDocument<'a>, PdfFontToken),
{
    let mut doc = pdfium.create_new_pdf().expect("create new pdf");
    let helvetica = doc.fonts_mut().helvetica();

    for page_idx in 0..num_pages {
        let mut page = doc
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::Custom(
                PdfPoints::new(612.0),
                PdfPoints::new(792.0),
            ))
            .expect("create page");
        draw_page(page_idx, &mut page, &doc, helvetica);
    }

    doc
}

/// Draws a single-line text object at `(x, y)` in PDF coordinate space (origin at bottom-left).
fn add_text(
    page: &mut PdfPage,
    doc: &PdfDocument,
    font_token: PdfFontToken,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
) {
    let font = doc.fonts().get(font_token).expect("helvetica font");
    let mut obj = page
        .objects_mut()
        .create_text_object(
            PdfPoints::new(x),
            PdfPoints::new(y),
            text,
            font,
            PdfPoints::new(font_size),
        )
        .expect("create text object");
    let _ = obj.set_fill_color(PdfColor::BLACK);
}

/// Renders a synthetic document via `draw_page` and writes the resulting PDF to `dir/filename`.
fn create_and_save_fixture<'a, F>(
    dir: &Path,
    filename: &str,
    pdfium: &'a Pdfium,
    num_pages: usize,
    draw_page: F,
) where
    F: FnMut(usize, &mut PdfPage<'a>, &PdfDocument<'a>, PdfFontToken),
{
    let doc = create_pdf_with_pages(pdfium, num_pages, draw_page);
    doc.save_to_file(&dir.join(filename))
        .unwrap_or_else(|e| panic!("failed to save {filename}: {e}"));
}

/// Generates all standard synthetic test fixtures into the specified target directory.
///
/// Fixtures generated:
/// - `identical_base.pdf` / `identical_target.pdf`: Baseline identity check (diff engine reports zero changes, exit code 0).
/// - `edit_base.pdf` / `edit_target.pdf`: Inline word substitutions, multi-word insertions, bullet point additions, and deletions.
/// - `multicolumn_base.pdf` / `multicolumn_target.pdf`: Multi-column spatial layout clustering and reading order verification.
/// - `page_mismatch_base.pdf` / `page_mismatch_target.pdf`: Unequal page counts (2 pages vs 1 page) and compositor rendering.
/// - `no_text_image.pdf`: Zero-text vector rectangle triggering `PdfVdiffError::NoTextLayer`.
pub fn generate_all_fixtures_to_dir(dir: &Path) {
    std::fs::create_dir_all(dir).expect("create fixtures dir");
    let pdfium = init_pdfium().expect("init pdfium");

    // 1. Identical Documents: Verifies identical baseline content results in no highlights and exit code 0.
    create_and_save_fixture(
        dir,
        "identical_base.pdf",
        pdfium,
        1,
        |_, page, doc, font| {
            add_text(page, doc, font, "Person A", 72.0, 720.0, 18.0);
            add_text(
                page,
                doc,
                font,
                "Senior Systems Engineer",
                72.0,
                695.0,
                13.0,
            );
            add_text(
                page,
                doc,
                font,
                "Led distributed systems architecture and Rust infrastructure.",
                72.0,
                670.0,
                11.0,
            );
        },
    );

    create_and_save_fixture(
        dir,
        "identical_target.pdf",
        pdfium,
        1,
        |_, page, doc, font| {
            add_text(page, doc, font, "Person A", 72.0, 720.0, 18.0);
            add_text(
                page,
                doc,
                font,
                "Senior Systems Engineer",
                72.0,
                695.0,
                13.0,
            );
            add_text(
                page,
                doc,
                font,
                "Led distributed systems architecture and Rust infrastructure.",
                72.0,
                670.0,
                11.0,
            );
        },
    );

    // 2. Edit Base and Target
    create_and_save_fixture(dir, "edit_base.pdf", pdfium, 1, |_, page, doc, font| {
        add_text(page, doc, font, "Person A", 72.0, 720.0, 18.0);
        add_text(
            page,
            doc,
            font,
            "Senior Systems Engineer",
            72.0,
            695.0,
            13.0,
        );
        add_text(
            page,
            doc,
            font,
            "Led distributed systems architecture and Rust infrastructure.",
            72.0,
            670.0,
            11.0,
        );
        add_text(
            page,
            doc,
            font,
            "Proficient in C++, Python, and legacy bash scripts.",
            72.0,
            645.0,
            11.0,
        );
        add_text(
            page,
            doc,
            font,
            "5 years experience at Company A.",
            72.0,
            620.0,
            11.0,
        );
    });

    create_and_save_fixture(dir, "edit_target.pdf", pdfium, 1, |_, page, doc, font| {
        add_text(page, doc, font, "Person A", 72.0, 720.0, 18.0);
        add_text(page, doc, font, "Staff Systems Engineer", 72.0, 695.0, 13.0);
        add_text(
            page,
            doc,
            font,
            "Led distributed systems architecture, cloud orchestration, and Rust infrastructure.",
            72.0,
            670.0,
            11.0,
        );
        add_text(
            page,
            doc,
            font,
            "Proficient in Rust, Go, Python, and Kubernetes.",
            72.0,
            645.0,
            11.0,
        );
        add_text(
            page,
            doc,
            font,
            "6 years experience at Company A.",
            72.0,
            620.0,
            11.0,
        );
        add_text(
            page,
            doc,
            font,
            "Published open-source tools for visual verification.",
            72.0,
            595.0,
            11.0,
        );
    });

    // 3. Multi-Column Base and Target
    create_and_save_fixture(
        dir,
        "multicolumn_base.pdf",
        pdfium,
        1,
        |_, page, doc, font| {
            // Left Column (x: 72..240)
            add_text(page, doc, font, "Skills & Languages", 72.0, 720.0, 14.0);
            add_text(page, doc, font, "Rust, Go, TypeScript", 72.0, 695.0, 11.0);
            add_text(
                page,
                doc,
                font,
                "Linux, Distributed Systems",
                72.0,
                675.0,
                11.0,
            );

            // Right Column (x: 300..540)
            add_text(page, doc, font, "Work Experience", 300.0, 720.0, 14.0);
            add_text(
                page,
                doc,
                font,
                "Staff Engineer at Company B",
                300.0,
                695.0,
                11.0,
            );
            add_text(
                page,
                doc,
                font,
                "Engineered low-latency pipelines.",
                300.0,
                675.0,
                11.0,
            );
        },
    );

    create_and_save_fixture(
        dir,
        "multicolumn_target.pdf",
        pdfium,
        1,
        |_, page, doc, font| {
            // Left Column
            add_text(page, doc, font, "Skills & Languages", 72.0, 720.0, 14.0);
            add_text(
                page,
                doc,
                font,
                "Rust, Go, TypeScript, C++",
                72.0,
                695.0,
                11.0,
            );
            add_text(
                page,
                doc,
                font,
                "Linux Kernel, Distributed Systems",
                72.0,
                675.0,
                11.0,
            );

            // Right Column
            add_text(page, doc, font, "Work Experience", 300.0, 720.0, 14.0);
            add_text(
                page,
                doc,
                font,
                "Principal Engineer at Company B",
                300.0,
                695.0,
                11.0,
            );
            add_text(
                page,
                doc,
                font,
                "Engineered zero-downtime streaming pipelines.",
                300.0,
                675.0,
                11.0,
            );
        },
    );

    // 4. Page Count Mismatch Base (2 pages) and Target (1 page)
    create_and_save_fixture(
        dir,
        "page_mismatch_base.pdf",
        pdfium,
        2,
        |page_idx, page, doc, font| {
            if page_idx == 0 {
                add_text(page, doc, font, "Person A - Page 1", 72.0, 720.0, 16.0);
                add_text(
                    page,
                    doc,
                    font,
                    "Detailed experience overview.",
                    72.0,
                    690.0,
                    12.0,
                );
            } else {
                add_text(page, doc, font, "Person A - Page 2", 72.0, 720.0, 16.0);
                add_text(
                    page,
                    doc,
                    font,
                    "Publications & Patents list.",
                    72.0,
                    690.0,
                    12.0,
                );
            }
        },
    );

    create_and_save_fixture(
        dir,
        "page_mismatch_target.pdf",
        pdfium,
        1,
        |_, page, doc, font| {
            add_text(
                page,
                doc,
                font,
                "Person A - Page 1 Condensed",
                72.0,
                720.0,
                16.0,
            );
            add_text(
                page,
                doc,
                font,
                "Summarized experience and publications.",
                72.0,
                690.0,
                12.0,
            );
        },
    );

    // 5. Scanned / Zero-Text PDF
    create_and_save_fixture(dir, "no_text_image.pdf", pdfium, 1, |_, page, _, _| {
        let _ = page.objects_mut().create_path_object_rect(
            PdfRect::new(
                PdfPoints::new(72.0),
                PdfPoints::new(72.0),
                PdfPoints::new(172.0),
                PdfPoints::new(172.0),
            ),
            None,
            None,
            Some(PdfColor::new(200, 200, 200, 255)),
        );
    });
}

/// Generates all standard synthetic fixtures directly into `tests/fixtures/`.
pub fn generate_all_fixtures() {
    generate_all_fixtures_to_dir(&get_fixtures_dir());
}

/// Verifies that all fixtures generate in an isolated temporary directory and satisfy the < 15 KB size limit.
#[test]
fn test_generate_fixtures_and_verify_sizes() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    generate_all_fixtures_to_dir(temp_dir.path());
    let fixtures_dir = get_fixtures_dir();

    let expected_files = [
        "identical_base.pdf",
        "identical_target.pdf",
        "edit_base.pdf",
        "edit_target.pdf",
        "multicolumn_base.pdf",
        "multicolumn_target.pdf",
        "page_mismatch_base.pdf",
        "page_mismatch_target.pdf",
        "no_text_image.pdf",
    ];

    for file in &expected_files {
        let path = temp_dir.path().join(file);
        assert!(path.exists(), "Missing fixture file in temp: {:?}", path);
        let size = std::fs::metadata(&path).expect("metadata").len();
        // Specs state each synthetic fixture must be lightweight (< 15 KB)
        assert!(
            size > 100 && size < 15_000,
            "Fixture {:?} size {} bytes outside expected range (100B - 15KB)",
            file,
            size
        );

        // Also verify committed fixture in tests/fixtures/
        let committed_path = fixtures_dir.join(file);
        assert!(
            committed_path.exists(),
            "Missing committed fixture: {:?}",
            committed_path
        );
        let committed_size = std::fs::metadata(&committed_path).expect("metadata").len();
        assert!(
            committed_size > 100 && committed_size < 15_000,
            "Committed fixture {:?} size {} bytes outside expected range (100B - 15KB)",
            file,
            committed_size
        );
    }
}

/// Manual utility test to regenerate all committed fixtures into `tests/fixtures/`.
/// Run explicitly with `cargo test -- --ignored test_regenerate_committed_fixtures`.
#[test]
#[ignore]
fn test_regenerate_committed_fixtures() {
    generate_all_fixtures();
}
