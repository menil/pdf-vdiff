//! CLI argument definitions and validation.

use crate::diff::DiffGranularity;
use crate::theme::{ThemeKind, DEFAULT_GUTTER_WIDTH, DEFAULT_HEADER_HEIGHT};
use clap::Parser;
use std::path::{Path, PathBuf};

/// CLI arguments for `pdf-vdiff`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "pdf-vdiff",
    author,
    version,
    about = "Side-by-side visual PDF diffing tool preserving vector fidelity and searchability",
    long_about = "A fast, standalone CLI tool that takes two PDF documents (e.g. base and tailored resumes) and produces a single side-by-side landscape PDF with IntelliJ/GitHub-style visual diff highlighting."
)]
pub struct CliArgs {
    /// Path to the original / base PDF document
    #[arg(value_name = "BASE_PDF")]
    pub base_pdf: PathBuf,

    /// Path to the modified / tailored PDF document
    #[arg(value_name = "TAILORED_PDF")]
    pub tailored_pdf: PathBuf,

    /// Target output PDF file path [default: <base_stem>_vs_<tailored_stem>_diff.pdf]
    #[arg(short = 'o', long = "output", value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Overwrite destination output file if it already exists
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Automatically open the generated diff in the system default PDF viewer
    #[arg(long = "open")]
    pub open: bool,

    /// Color palette theme (intellij, github, classic, high-contrast)
    #[arg(long = "theme", value_enum, default_value_t = ThemeKind::IntelliJ)]
    pub theme: ThemeKind,

    /// Diff granularity mode (word, line, character)
    #[arg(long = "granularity", value_enum, default_value_t = DiffGranularity::Word)]
    pub granularity: DiffGranularity,

    /// Spacing in points between left and right pages
    #[arg(long = "gutter-width", default_value_t = DEFAULT_GUTTER_WIDTH)]
    pub gutter_width: f32,

    /// Suppress the top header/metadata banner
    #[arg(long = "no-header")]
    pub no_header: bool,

    /// Maximum page count threshold to prevent unbounded processing
    #[arg(long = "max-pages", default_value_t = 250)]
    pub max_pages: usize,

    /// Enable verbose structural logging (omits sensitive raw text PII)
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}

/// Extract the file stem as string or return fallback if not present or invalid UTF-8.
#[inline]
pub fn path_stem_or<'a>(path: &'a Path, fallback: &'a str) -> &'a str {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(fallback)
}

/// Extract the file name as string or return fallback if not present or invalid UTF-8.
#[inline]
pub fn path_file_name_or<'a>(path: &'a Path, fallback: &'a str) -> &'a str {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(fallback)
}

impl CliArgs {
    /// Resolves the effective output path, defaulting to `<base_stem>_vs_<tailored_stem>_diff.pdf`.
    pub fn resolve_output_path(&self) -> PathBuf {
        if let Some(ref path) = self.output {
            path.clone()
        } else {
            let base_stem = path_stem_or(&self.base_pdf, "base");
            let tailored_stem = path_stem_or(&self.tailored_pdf, "tailored");
            PathBuf::from(format!("{}_vs_{}_diff.pdf", base_stem, tailored_stem))
        }
    }

    /// Header height in points taking `--no-header` flag into account.
    pub fn effective_header_height(&self) -> f32 {
        if self.no_header {
            0.0
        } else {
            DEFAULT_HEADER_HEIGHT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_output_path_resolution() {
        let args = CliArgs {
            base_pdf: PathBuf::from("path/to/my_resume_v1.pdf"),
            tailored_pdf: PathBuf::from("other/path/my_resume_v2.pdf"),
            output: None,
            force: false,
            open: false,
            theme: ThemeKind::IntelliJ,
            granularity: DiffGranularity::Word,
            gutter_width: DEFAULT_GUTTER_WIDTH,
            no_header: false,
            max_pages: 250,
            verbose: false,
        };

        assert_eq!(
            args.resolve_output_path(),
            PathBuf::from("my_resume_v1_vs_my_resume_v2_diff.pdf")
        );
        assert_eq!(args.effective_header_height(), DEFAULT_HEADER_HEIGHT);
    }

    #[test]
    fn test_cli_custom_output_path_and_no_header() {
        let args = CliArgs {
            base_pdf: PathBuf::from("base.pdf"),
            tailored_pdf: PathBuf::from("tailored.pdf"),
            output: Some(PathBuf::from("custom_out.pdf")),
            force: true,
            open: true,
            theme: ThemeKind::GitHub,
            granularity: DiffGranularity::Line,
            gutter_width: 30.0,
            no_header: true,
            max_pages: 100,
            verbose: true,
        };

        assert_eq!(args.resolve_output_path(), PathBuf::from("custom_out.pdf"));
        assert_eq!(args.effective_header_height(), 0.0);
    }

    #[test]
    fn test_path_helpers() {
        let p = Path::new("some/nested/file.pdf");
        assert_eq!(path_stem_or(p, "fallback"), "file");
        assert_eq!(path_file_name_or(p, "fallback"), "file.pdf");

        let empty = Path::new("");
        assert_eq!(path_stem_or(empty, "fallback"), "fallback");
        assert_eq!(path_file_name_or(empty, "fallback"), "fallback");
    }
}
