//! `pdf_vdiff` core library.
//! Provides pure-Rust geometry, token clustering, diffing, and formatting models.

pub mod cluster;
pub mod error;
pub mod model;
pub mod theme;

pub use cluster::{cluster_and_sort_tokens, cluster_tokens_default, normalize_token_text};
pub use error::PdfVdiffError;
pub use model::{DiffOpKind, HighlightSpan, PageText, Rect, TextToken};
pub use theme::{DiffTheme, ThemeKind};

/// Returns the library version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
