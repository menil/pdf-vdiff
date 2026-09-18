# AI Decision Record: PDFium Lifecycle, Text Extraction, and 4-Layer Vector Compositor

## Context & Problem Statement
`pdf-vdiff` requires a robust engine to:
1. Initialize PDFium in a hardened, script-disabled environment with dynamic library discovery.
2. Extract text tokens and character bounds from PDF pages and coalesce glyph runs into words.
3. Composite a landscape side-by-side visual diff PDF using a 4-layer Z-order rendering pipeline (Layer 1: Backgrounds/Placeholders, Layer 2: Vector XObjects, Layer 3: Vector Highlights with Multiply blend mode, Layer 4: Chrome/Gutter/Header).
4. Save the generated document atomically to prevent partial writes.

## Architectural Decisions
1. **Singleton PDFium Lifecycle (`OnceLock`)**:
   PDFium uses global process-level state. Repeatedly initializing and destroying dynamic library handles across multiple concurrent test runs causes thread-safety traps. We encapsulated PDFium initialization inside a `OnceLock` singleton and enforced single-threaded test execution (`--test-threads=1`) in `Justfile`.

2. **4-Layer Vector Compositor Pipeline**:
   - Layer 1: Canvas page background and dashed placeholder boxes for mismatched page counts (`[No corresponding page in ...]`).
   - Layer 2: Native vector page content embedded as `PdfPageXObjectFormObject` using `copy_into_x_object_form_object` and translated to viewport pane origins.
   - Layer 3: Visual diff highlight rectangles rendered as native `PdfPagePathObject` paths with RGBA fills, thin strokes, and `PdfPageObjectBlendMode::Multiply`.
   - Layer 4: Vertical gutter divider line and metadata header banner with Helvetica typography.

3. **Atomic File Persistence**:
   Output PDFs are serialized into a temporary file (`tempfile::NamedTempFile`) in the destination directory and atomically renamed/persisted to avoid corrupted or partial outputs on abort.

4. **Token Coalescing**:
   Raw PDFium glyph segments placed with kerning/tracking are coalesced into natural word tokens when sharing a visual baseline and sub-pixel horizontal proximity without trailing spaces.

## Verification & Validation
- Validated against private real resume PDFs (363 KB side-by-side diff PDF generated successfully).
- Synthetic unit tests for single/multi-page documents, page count mismatches, and header suppression.
- Achieved **89.19% line coverage** via `cargo llvm-cov`.
