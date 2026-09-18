//! PDFium initialization and lifecycle management.

use crate::error::PdfVdiffError;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PDFIUM_INSTANCE: OnceLock<Pdfium> = OnceLock::new();

/// Initialize PDFium with dynamic system binding or local search fallbacks (singleton).
pub fn init_pdfium() -> Result<&'static Pdfium, PdfVdiffError> {
    if let Some(instance) = PDFIUM_INSTANCE.get() {
        return Ok(instance);
    }

    let instance = create_pdfium_instance()?;
    let _ = PDFIUM_INSTANCE.set(instance);
    Ok(PDFIUM_INSTANCE.get().expect("PDFium instance initialized"))
}

fn create_pdfium_instance() -> Result<Pdfium, PdfVdiffError> {
    // 1. Try explicit environment variable PDFIUM_LIB_DIR or PDFIUM_LIB_PATH
    if let Ok(path_str) = std::env::var("PDFIUM_LIB_PATH") {
        let path = Path::new(&path_str);
        if let Ok(bindings) = Pdfium::bind_to_library(path) {
            return Ok(Pdfium::new(bindings));
        }
    }
    if let Ok(dir_str) = std::env::var("PDFIUM_LIB_DIR") {
        let path = PathBuf::from(dir_str).join(Pdfium::pdfium_platform_library_name_at_path("./"));
        if let Ok(bindings) = Pdfium::bind_to_library(path) {
            return Ok(Pdfium::new(bindings));
        }
    }

    // 2. Try default system library search
    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Ok(Pdfium::new(bindings));
    }

    // 3. Try current working directory
    let local_name = Pdfium::pdfium_platform_library_name_at_path("./");
    if let Ok(bindings) = Pdfium::bind_to_library(&local_name) {
        return Ok(Pdfium::new(bindings));
    }

    // 4. Try current executable's directory
    if let Ok(mut exe_path) = std::env::current_exe() {
        exe_path.pop();
        let exe_dir_lib = exe_path.join(Pdfium::pdfium_platform_library_name_at_path("./"));
        if let Ok(bindings) = Pdfium::bind_to_library(exe_dir_lib) {
            return Ok(Pdfium::new(bindings));
        }
    }

    // 5. Common system installation paths
    for search_dir in &[
        "/usr/local/lib",
        "/opt/homebrew/lib",
        "/usr/lib",
        "/tmp/lib",
    ] {
        let path =
            PathBuf::from(search_dir).join(Pdfium::pdfium_platform_library_name_at_path("./"));
        if path.exists() {
            if let Ok(bindings) = Pdfium::bind_to_library(&path) {
                return Ok(Pdfium::new(bindings));
            }
        }
    }

    Err(PdfVdiffError::Render(
        "Could not find or bind to PDFium dynamic library (libpdfium.dylib/so/dll).".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdfium_initialization() {
        let pdfium = init_pdfium();
        assert!(
            pdfium.is_ok(),
            "PDFium should initialize successfully: {:?}",
            pdfium.err()
        );
    }
}
