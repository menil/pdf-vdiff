# AI Decision Record: Technical Specification for `pdf-vdiff`

## Context & Goal
`pdf-vdiff` is a standalone Rust CLI tool that compares two PDF documents (e.g., base and tailored resumes) and generates a single side-by-side landscape PDF with IntelliJ-style visual diff highlighting while preserving 100% vector fidelity and text searchability.

## Architecture & Key Decisions

1. **Dual-Target Crate Architecture (`lib.rs` + `main.rs`)**:
   - Decouples core geometric algorithms, spatial clustering, sequence diffing, and formatting (`pdf_vdiff` library) from native PDFium C++ FFI bindings and CLI binary logic (`pdf-vdiff`).
   - Ensures 100% pure-Rust unit testability to meet the strict **85% minimum code coverage** threshold (`cargo llvm-cov`).

2. **Spatial Column Clustering ($O(T \log T)$)**:
   - PDF content streams often encode text out of natural visual reading order.
   - Horizontal gap histogram partitioning detects vertical column gutters, and vertical baseline grouping ($\Delta y \le 2.0\text{ pt}$) reconstructs natural reading order for multi-column resumes.

3. **Hierarchical Two-Pass Sequence Diffing**:
   - **Pass 1 (Line-level)**: Myers sequence matching identifies coarse matching vs modified lines ($O(L \cdot D_L)$).
   - **Pass 2 (Token-level)**: Myers token diff isolates altered words only within modified lines.
   - Prevents $O(N^2)$ quadratic slowdowns on large document comparisons.

4. **Vector Path Highlight Compositing (vs. PDF Annotations)**:
   - Highlights rendered as native `PdfPagePathObject` vector rectangles with RGBA fills and `Multiply` blend mode directly in page streams.
   - Eliminates platform-dependent annotation rendering quirks across Apple Preview, Adobe Acrobat, and Chrome PDF viewers.

5. **Coordinate Frame & CropBox Normalization**:
   - Explicitly models PDF user-space bottom-left origin ($y_1 > y_0$).
   - Normalizes non-zero `CropBox` origins and uprights rotated pages ($90^\circ, 180^\circ, 270^\circ$).

6. **Hardened Security & Robustness**:
   - Disables PDFium Google V8 JavaScript runtime, XFA forms, and external URI launch hooks.
   - Atomic output writes via temporary files and `--force` overwrite guards.
   - Standard 3-tier exit code contract (`0` = identical, `1` = differences found, `2` = execution error).

## Alternatives Considered & Rejected

1. **Pixel-Based Raster Diffing (e.g. `diff-pdf`)**:
   - *Rejected*: Loses vector crispness, degrades typography on zooming, produces unselectable raster PDFs, and blurs subtle formatting adjustments.

2. **Terminal / Text-Only Diffing**:
   - *Rejected*: Loses spatial layout, headers, multi-column margins, and document visual context.

3. **PDF Standard Annotations (`/Annots`, `/Highlight`)**:
   - *Rejected*: Inconsistent styling across PDF viewers (Apple Preview overrides highlight colors with fixed yellow, Acrobat ignores alpha parameters).

4. **Single-Pass Document-Wide Token Diffing**:
   - *Rejected*: Degenerates to $O(N^2)$ on restructured documents; paragraph insertions cross page boundaries and create cascading false-positive diffs across all pages.
