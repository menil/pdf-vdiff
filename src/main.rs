//! CLI entrypoint for `pdf-vdiff`.

use std::process::ExitCode;

fn main() -> ExitCode {
    println!("pdf-vdiff v{}", pdf_vdiff::version());
    ExitCode::SUCCESS
}
