//! CLI argument definitions and validation.

use crate::diff::DiffGranularity;
use crate::theme::{ThemeKind, DEFAULT_GUTTER_WIDTH, DEFAULT_HEADER_HEIGHT};
use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use std::path::{Path, PathBuf};

/// CLI arguments for `pdf-vdiff`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "pdf-vdiff",
    version,
    about = "Side-by-side visual PDF diffing tool preserving vector fidelity and searchability",
    long_about = "A fast, standalone CLI tool that takes two PDF documents (e.g. base and tailored resumes) and produces a single side-by-side landscape PDF with IntelliJ/GitHub-style visual diff highlighting."
)]
pub struct CliArgs {
    /// Path to the original / base PDF document
    #[arg(
        value_name = "BASE_PDF",
        required_unless_present_any = ["generate_completions", "generate_man"]
    )]
    pub base_pdf: Option<PathBuf>,

    /// Path to the modified / tailored PDF document
    #[arg(
        value_name = "TAILORED_PDF",
        required_unless_present_any = ["generate_completions", "generate_man"]
    )]
    pub tailored_pdf: Option<PathBuf>,

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

    /// Generate shell completion script to stdout (bash, zsh, fish, elvish, powershell)
    #[arg(long = "generate-completions", value_name = "SHELL", value_enum)]
    pub generate_completions: Option<Shell>,

    /// Generate roff man page (Section 1) to stdout
    #[arg(long = "generate-man")]
    pub generate_man: bool,
}

/// Extract the file stem from an optional path or return fallback if absent or invalid UTF-8.
#[inline]
pub fn opt_path_stem_or<'a>(path: Option<&'a Path>, fallback: &'a str) -> &'a str {
    path.and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
        .unwrap_or(fallback)
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
            let base_stem = opt_path_stem_or(self.base_pdf.as_deref(), "base");
            let tailored_stem = opt_path_stem_or(self.tailored_pdf.as_deref(), "tailored");
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

/// Render the roff man page (Section 1) for `pdf-vdiff` to the given writer.
pub fn render_man_page<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    clap_mangen::Man::new(CliArgs::command()).render(writer)
}

/// Render shell completion script for `pdf-vdiff` to the given writer.
pub fn render_completions<W: std::io::Write>(shell: Shell, writer: &mut W) -> std::io::Result<()> {
    let mut cmd = CliArgs::command();
    clap_complete::generate(shell, &mut cmd, "pdf-vdiff", writer);
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_output_path_resolution() {
        let args = CliArgs {
            base_pdf: Some(PathBuf::from("path/to/my_resume_v1.pdf")),
            tailored_pdf: Some(PathBuf::from("other/path/my_resume_v2.pdf")),
            output: None,
            force: false,
            open: false,
            theme: ThemeKind::IntelliJ,
            granularity: DiffGranularity::Word,
            gutter_width: DEFAULT_GUTTER_WIDTH,
            no_header: false,
            max_pages: 250,
            verbose: false,
            generate_completions: None,
            generate_man: false,
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
            base_pdf: Some(PathBuf::from("base.pdf")),
            tailored_pdf: Some(PathBuf::from("tailored.pdf")),
            output: Some(PathBuf::from("custom_out.pdf")),
            force: true,
            open: true,
            theme: ThemeKind::GitHub,
            granularity: DiffGranularity::Line,
            gutter_width: 30.0,
            no_header: true,
            max_pages: 100,
            verbose: true,
            generate_completions: None,
            generate_man: false,
        };

        assert_eq!(args.resolve_output_path(), PathBuf::from("custom_out.pdf"));
        assert_eq!(args.effective_header_height(), 0.0);
    }

    #[test]
    fn test_cli_asset_generation_methods() {
        let mut man_buf = Vec::new();
        render_man_page(&mut man_buf).expect("render man page");
        let man_str = String::from_utf8(man_buf).expect("utf-8 man page");
        assert!(man_str.contains(".TH pdf-vdiff 1"));
        assert!(man_str.contains("pdf\\-vdiff"));

        for shell in [
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::Elvish,
            Shell::PowerShell,
        ] {
            let mut comp_buf = Vec::new();
            render_completions(shell, &mut comp_buf).unwrap();
            let comp_str = String::from_utf8(comp_buf).expect("utf-8 completion");
            assert!(!comp_str.is_empty());
            assert!(comp_str.contains("pdf-vdiff"));
        }
    }

    #[test]
    fn test_path_helpers() {
        let p = Path::new("some/nested/file.pdf");
        assert_eq!(path_stem_or(p, "fallback"), "file");
        assert_eq!(path_file_name_or(p, "fallback"), "file.pdf");
        assert_eq!(opt_path_stem_or(Some(p), "fallback"), "file");

        let empty = Path::new("");
        assert_eq!(path_stem_or(empty, "fallback"), "fallback");
        assert_eq!(path_file_name_or(empty, "fallback"), "fallback");
        assert_eq!(opt_path_stem_or(Some(empty), "fallback"), "fallback");
        assert_eq!(opt_path_stem_or(None, "fallback"), "fallback");
    }
}
