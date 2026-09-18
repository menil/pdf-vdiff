//! CLI entrypoint for `pdf-vdiff`.

use clap::Parser;
use pdf_vdiff::cli::{path_file_name_or, CliArgs};
use pdf_vdiff::diff::diff_documents;
use pdf_vdiff::error::PdfVdiffError;
use pdf_vdiff::layout::{CanvasLayout, PageDimensions};
use pdf_vdiff::pdf::{extract_document_tokens, init_pdfium, CanvasCompositor, HeaderMetadata};
use std::process::ExitCode;

/// Core execution routine for CLI invocation.
pub fn run(args: CliArgs) -> Result<i32, PdfVdiffError> {
    // 1. Pre-flight input existence checks
    if !args.base_pdf.exists() {
        return Err(PdfVdiffError::FileNotFound(args.base_pdf));
    }
    if !args.tailored_pdf.exists() {
        return Err(PdfVdiffError::FileNotFound(args.tailored_pdf));
    }

    let output_path = args.resolve_output_path();
    if output_path.exists() && !args.force {
        return Err(PdfVdiffError::OutputFileExists(output_path));
    }

    if args.verbose {
        eprintln!("Comparing: {:?} vs {:?}", args.base_pdf, args.tailored_pdf);
        eprintln!(
            "Theme: {:?}, Granularity: {:?}",
            args.theme, args.granularity
        );
    }

    // 2. Initialize PDFium engine
    let pdfium = init_pdfium()?;

    // 3. Load input documents
    let base_doc = pdfium
        .load_pdf_from_file(&args.base_pdf, None)
        .map_err(|e| PdfVdiffError::PdfOpen {
            path: args.base_pdf.clone(),
            reason: e.to_string(),
        })?;

    let tailored_doc = pdfium
        .load_pdf_from_file(&args.tailored_pdf, None)
        .map_err(|e| PdfVdiffError::PdfOpen {
            path: args.tailored_pdf.clone(),
            reason: e.to_string(),
        })?;

    // 4. Extract tokens from documents
    let base_pages = extract_document_tokens(&base_doc, args.max_pages)?;
    let tailored_pages = extract_document_tokens(&tailored_doc, args.max_pages)?;

    let total_base_tokens: usize = base_pages.iter().map(|p| p.tokens.len()).sum();
    let total_tailored_tokens: usize = tailored_pages.iter().map(|p| p.tokens.len()).sum();

    if total_base_tokens == 0 && total_tailored_tokens == 0 {
        return Err(PdfVdiffError::NoTextLayer(args.base_pdf));
    }

    // 5. Compute hierarchical sequence diff
    let diff_result = diff_documents(&base_pages, &tailored_pages, args.granularity);

    let max_pages = base_pages.len().max(tailored_pages.len());
    let mut layouts = Vec::with_capacity(max_pages);
    let mut highlights = Vec::with_capacity(max_pages);

    let header_height = args.effective_header_height();

    for i in 0..max_pages {
        let b_dims = base_pages
            .get(i)
            .map(|p| PageDimensions::new(p.width, p.height));
        let t_dims = tailored_pages
            .get(i)
            .map(|p| PageDimensions::new(p.width, p.height));
        layouts.push(CanvasLayout::compute(
            b_dims,
            t_dims,
            args.gutter_width,
            header_height,
        ));

        let page_diff = diff_result.pages.get(i);
        let (b_high, t_high) = page_diff
            .map(|p| (p.base_highlights.clone(), p.tailored_highlights.clone()))
            .unwrap_or_default();
        highlights.push((b_high, t_high));
    }

    // 6. Setup compositor and render 4-layer vector canvas
    let base_name = path_file_name_or(&args.base_pdf, "Base");
    let tailored_name = path_file_name_or(&args.tailored_pdf, "Tailored");

    let header_meta = HeaderMetadata {
        base_name,
        tailored_name,
        total_pages: max_pages,
        has_differences: diff_result.has_differences,
    };

    let theme = args.theme.theme();
    let compositor = CanvasCompositor::new(
        pdfium,
        theme,
        if args.no_header {
            None
        } else {
            Some(header_meta)
        },
    );

    compositor.render_and_save(
        Some(&base_doc),
        Some(&tailored_doc),
        &layouts,
        &highlights,
        &output_path,
    )?;

    if args.verbose {
        eprintln!("Successfully wrote visual diff to {:?}", output_path);
    }

    // 7. Launch non-fatal system viewer if requested
    if args.open {
        if let Err(e) = open::that(&output_path) {
            eprintln!(
                "Warning: Failed to open system viewer for {:?}: {}",
                output_path, e
            );
        }
    }

    // 8. Return exit code contract: 1 for diffs found, 0 for identical
    if diff_result.has_differences {
        println!(
            "Differences detected between {:?} and {:?}",
            args.base_pdf, args.tailored_pdf
        );
        Ok(1)
    } else {
        println!(
            "Documents {:?} and {:?} are identical.",
            args.base_pdf, args.tailored_pdf
        );
        Ok(0)
    }
}

fn main() -> ExitCode {
    let args = CliArgs::parse();
    match run(args) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(1) => ExitCode::from(1),
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pdf_vdiff::theme::ThemeKind;
    use pdfium_render::prelude::*;
    use std::path::PathBuf;

    fn write_test_pdf(path: &std::path::Path, lines: &[&str]) {
        let pdfium = init_pdfium().expect("init pdfium");
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

        doc.save_to_file(path).expect("save pdf");
    }

    fn write_empty_page_pdf(path: &std::path::Path) {
        let pdfium = init_pdfium().expect("init pdfium");
        let mut doc = pdfium.create_new_pdf().expect("create pdf");
        doc.pages_mut()
            .create_page_at_end(PdfPagePaperSize::Custom(
                PdfPoints::new(612.0),
                PdfPoints::new(792.0),
            ))
            .expect("create page");
        doc.save_to_file(path).expect("save empty pdf");
    }

    #[test]
    fn test_run_missing_input_file() {
        let args = CliArgs {
            base_pdf: PathBuf::from("non_existent_base_file_12345.pdf"),
            tailored_pdf: PathBuf::from("non_existent_tailored_file_12345.pdf"),
            output: None,
            force: false,
            open: false,
            theme: ThemeKind::IntelliJ,
            granularity: pdf_vdiff::diff::DiffGranularity::Word,
            gutter_width: 24.0,
            no_header: false,
            max_pages: 250,
            verbose: false,
        };

        let res = run(args);
        assert!(res.is_err());
        match res.unwrap_err() {
            PdfVdiffError::FileNotFound(p) => {
                assert_eq!(p, PathBuf::from("non_existent_base_file_12345.pdf"));
            }
            other => panic!("Expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_run_output_exists_without_force() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let base = temp_dir.path().join("base.pdf");
        let tailored = temp_dir.path().join("tailored.pdf");
        let output = temp_dir.path().join("output.pdf");

        std::fs::write(&base, b"dummy base").expect("write base");
        std::fs::write(&tailored, b"dummy tailored").expect("write tailored");
        std::fs::write(&output, b"existing output").expect("write output");

        let args = CliArgs {
            base_pdf: base,
            tailored_pdf: tailored,
            output: Some(output.clone()),
            force: false,
            open: false,
            theme: ThemeKind::IntelliJ,
            granularity: pdf_vdiff::diff::DiffGranularity::Word,
            gutter_width: 24.0,
            no_header: false,
            max_pages: 250,
            verbose: false,
        };

        let res = run(args);
        assert!(res.is_err());
        match res.unwrap_err() {
            PdfVdiffError::OutputFileExists(p) => {
                assert_eq!(p, output);
            }
            other => panic!("Expected OutputFileExists, got {:?}", other),
        }
    }

    #[test]
    fn test_run_success_with_differences() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let base = temp_dir.path().join("base.pdf");
        let tailored = temp_dir.path().join("tailored.pdf");
        let output = temp_dir.path().join("diff_out.pdf");

        write_test_pdf(&base, &["Senior Systems Architect", "Rust and C++"]);
        write_test_pdf(&tailored, &["Lead Systems Architect", "Rust, C++, and Go"]);

        let args = CliArgs {
            base_pdf: base,
            tailored_pdf: tailored,
            output: Some(output.clone()),
            force: true,
            open: false,
            theme: ThemeKind::GitHub,
            granularity: pdf_vdiff::diff::DiffGranularity::Word,
            gutter_width: 24.0,
            no_header: false,
            max_pages: 250,
            verbose: true,
        };

        let res = run(args);
        assert_eq!(res.unwrap(), 1);
        assert!(output.exists());
    }

    #[test]
    fn test_run_success_identical_documents() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let base = temp_dir.path().join("base.pdf");
        let tailored = temp_dir.path().join("tailored.pdf");
        let output = temp_dir.path().join("identical_diff.pdf");

        write_test_pdf(&base, &["Identical Document Content"]);
        write_test_pdf(&tailored, &["Identical Document Content"]);

        let args = CliArgs {
            base_pdf: base,
            tailored_pdf: tailored,
            output: Some(output.clone()),
            force: true,
            open: false,
            theme: ThemeKind::Classic,
            granularity: pdf_vdiff::diff::DiffGranularity::Line,
            gutter_width: 20.0,
            no_header: true,
            max_pages: 50,
            verbose: false,
        };

        let res = run(args);
        assert_eq!(res.unwrap(), 0);
        assert!(output.exists());
    }

    #[test]
    fn test_run_no_text_layer_error() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let base = temp_dir.path().join("empty_base.pdf");
        let tailored = temp_dir.path().join("empty_tailored.pdf");
        let output = temp_dir.path().join("empty_diff.pdf");

        write_empty_page_pdf(&base);
        write_empty_page_pdf(&tailored);

        let args = CliArgs {
            base_pdf: base.clone(),
            tailored_pdf: tailored,
            output: Some(output),
            force: true,
            open: false,
            theme: ThemeKind::HighContrast,
            granularity: pdf_vdiff::diff::DiffGranularity::Character,
            gutter_width: 24.0,
            no_header: false,
            max_pages: 250,
            verbose: false,
        };

        let res = run(args);
        assert!(res.is_err());
        match res.unwrap_err() {
            PdfVdiffError::NoTextLayer(p) => assert_eq!(p, base),
            other => panic!("Expected NoTextLayer, got {:?}", other),
        }
    }

    #[test]
    fn test_run_invalid_pdf_content() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let base = temp_dir.path().join("corrupted.pdf");
        let tailored = temp_dir.path().join("valid.pdf");

        std::fs::write(&base, b"not a valid pdf header").expect("write corrupt");
        write_test_pdf(&tailored, &["Valid content"]);

        let args = CliArgs {
            base_pdf: base.clone(),
            tailored_pdf: tailored,
            output: None,
            force: true,
            open: false,
            theme: ThemeKind::IntelliJ,
            granularity: pdf_vdiff::diff::DiffGranularity::Word,
            gutter_width: 24.0,
            no_header: false,
            max_pages: 250,
            verbose: false,
        };

        let res = run(args);
        assert!(res.is_err());
        match res.unwrap_err() {
            PdfVdiffError::PdfOpen { path, .. } => assert_eq!(path, base),
            other => panic!("Expected PdfOpen error, got {:?}", other),
        }
    }
}
