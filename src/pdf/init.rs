//! PDFium initialization and lifecycle management.

use crate::error::PdfVdiffError;
use pdfium_render::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static PDFIUM_INSTANCE: OnceLock<Pdfium> = OnceLock::new();

/// Initialize PDFium with dynamic system binding or local search fallbacks (singleton).
///
/// # Errors
///
/// Returns [`PdfVdiffError::Render`] if no compatible PDFium dynamic library (`libpdfium.dylib`,
/// `libpdfium.so`, or `pdfium.dll`) can be located or dynamically bound across environment
/// variables, system loader paths, the executable directory, or standard OS directories.
pub fn init_pdfium() -> Result<&'static Pdfium, PdfVdiffError> {
    if let Some(instance) = PDFIUM_INSTANCE.get() {
        return Ok(instance);
    }

    let instance = create_pdfium_instance()?;
    let _ = PDFIUM_INSTANCE.set(instance);
    Ok(PDFIUM_INSTANCE.get().expect("PDFium instance initialized"))
}

fn create_pdfium_instance() -> Result<Pdfium, PdfVdiffError> {
    // 1. Explicit environment overrides take highest precedence to allow CI,
    // Nix wrappers, and containerized deployments to pin custom library builds.
    if let Ok(path_str) = std::env::var("PDFIUM_LIB_PATH") {
        let path = Path::new(&path_str);
        if path.is_file() {
            if let Ok(bindings) = Pdfium::bind_to_library(path) {
                return Ok(Pdfium::new(bindings));
            }
        }
    }
    if let Ok(dir_str) = std::env::var("PDFIUM_LIB_DIR") {
        let path = PathBuf::from(dir_str).join(Pdfium::pdfium_platform_library_name_at_path("./"));
        if path.is_file() {
            if let Ok(bindings) = Pdfium::bind_to_library(path) {
                return Ok(Pdfium::new(bindings));
            }
        }
    }

    // 2. Standard system dynamic linker search paths (e.g., ld.so.cache, dyld).
    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Ok(Pdfium::new(bindings));
    }

    // 3. Executable's parent directory: supports portable standalone binary distributions
    // where `libpdfium` is bundled alongside `pdf-vdiff` inside release archives.
    // Canonicalize the executable path to follow symlinks when invoked via symlinked PATH.
    if let Ok(exe_path) = std::env::current_exe() {
        let resolved_exe = std::fs::canonicalize(&exe_path).unwrap_or(exe_path);
        if let Some(exe_dir) = resolved_exe.parent() {
            let exe_dir_lib = exe_dir.join(Pdfium::pdfium_platform_library_name_at_path("./"));
            if exe_dir_lib.is_file() {
                if let Ok(bindings) = Pdfium::bind_to_library(exe_dir_lib) {
                    return Ok(Pdfium::new(bindings));
                }
            }
        }
    }

    // 4. Common OS package manager and FHS directories as fallbacks:
    // - /opt/homebrew/lib: Apple Silicon macOS Homebrew prefix
    // - /usr/local/lib: Intel macOS Homebrew and standard Linux local installs
    // - /usr/lib: Linux system libraries
    // - /usr/lib64: RedHat / Fedora / CentOS 64-bit multiarch
    // - /usr/lib/x86_64-linux-gnu: Debian / Ubuntu x86_64 multiarch
    // - /usr/lib/aarch64-linux-gnu: Debian / Ubuntu ARM64 multiarch
    // (Note: /tmp/lib is deliberately omitted to prevent untrusted library planting CWE-426).
    for search_dir in &[
        "/opt/homebrew/lib",
        "/usr/local/lib",
        "/usr/lib",
        "/usr/lib64",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
    ] {
        let path =
            PathBuf::from(search_dir).join(Pdfium::pdfium_platform_library_name_at_path("./"));
        if path.is_file() {
            if let Ok(bindings) = Pdfium::bind_to_library(&path) {
                return Ok(Pdfium::new(bindings));
            }
        }
    }

    // 5. Current working directory fallback for local developer checkouts.
    // Checked after executable and system directories to prevent DLL planting vulnerabilities.
    let local_name = Pdfium::pdfium_platform_library_name_at_path("./");
    let local_path = Path::new(&local_name);
    if local_path.is_file() {
        if let Ok(bindings) = Pdfium::bind_to_library(local_path) {
            return Ok(Pdfium::new(bindings));
        }
    }

    Err(PdfVdiffError::Render(
        "Could not find or bind to PDFium dynamic library (libpdfium.dylib/so/dll).\n\
        To resolve this:\n\
          - Download a self-contained release archive from https://github.com/menil/pdf-vdiff/releases\n\
          - Or install PDFium to your system (e.g. `brew install bblanchon/pdfium/pdfium-binaries` on macOS)\n\
          - Or set the PDFIUM_LIB_PATH / PDFIUM_LIB_DIR environment variable to your libpdfium location\n\
          - Or run via Nix: `nix run github:menil/pdf-vdiff`"
            .to_string(),
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
