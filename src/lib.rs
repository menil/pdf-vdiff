//! `pdf_vdiff` core library.
//! Provides pure-Rust geometry, token clustering, and diffing engines.

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
