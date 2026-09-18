//! Typed domain errors for `pdf-vdiff`.

use std::path::PathBuf;

/// Comprehensive error type for all core pdf-vdiff operations.
#[derive(thiserror::Error, Debug)]
pub enum PdfVdiffError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Output file '{0}' already exists. Use --force to overwrite.")]
    OutputFileExists(PathBuf),

    #[error("Failed to open PDF at '{path}': {reason}")]
    PdfOpen { path: PathBuf, reason: String },

    #[error("PDF '{0}' is password-protected. Please provide an unlocked document.")]
    PasswordProtected(PathBuf),

    #[error("PDF '{0}' contains no extractable text layer. Text-based diffing requires OCR text.")]
    NoTextLayer(PathBuf),

    #[error("Page count ({0}) exceeds maximum allowed limit ({1}). Override with --max-pages.")]
    PageCountExceeded(usize, usize),

    #[error("Invalid page dimensions in '{path}': {reason}")]
    InvalidDimensions { path: PathBuf, reason: String },

    #[error("Output IO error at '{path}': {source}")]
    OutputIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Rendering engine error: {0}")]
    Render(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PdfVdiffError::FileNotFound(PathBuf::from("test.pdf"));
        assert_eq!(err.to_string(), "File not found: test.pdf");

        let err = PdfVdiffError::OutputFileExists(PathBuf::from("out.pdf"));
        assert!(err.to_string().contains("--force"));

        let err = PdfVdiffError::PageCountExceeded(300, 250);
        assert!(err.to_string().contains("300"));
    }
}
