//! PDF rendering, text extraction, and vector composition module.

pub mod composer;
pub mod extract;
pub mod init;

pub use composer::{CanvasCompositor, HeaderMetadata};
pub use extract::{extract_document_tokens, extract_page_tokens};
pub use init::init_pdfium;
