# AI Decision Record: Core Data Models, Themes, and Typed Error Handling

## Context & Goal
Provide the fundamental domain representations for PDF geometric bounding boxes, extracted tokens, diff operations, visual theme palettes, and structured domain errors for `pdf-vdiff`.

## Architecture & Key Decisions
1. **PDF User-Space Bounding Box (`Rect`)**:
   - Explicitly designed with bottom-left origin ($y_1 > y_0$) matching PDF specifications.
   - Provided `union`, `expand`, `contains_point`, and `overlaps` geometric operations.
2. **Text Token Representation (`TextToken`)**:
   - Contains raw extracted `text`, `normalized_text` (for diff key matching), `CropBox`-normalized `bounds`, `page_index`, `column_index`, and `line_index`.
3. **Theme Presets (`theme.rs`)**:
   - Implemented normalized RGBA float palettes for `IntelliJ`, `GitHub`, `Classic`, and `HighContrast` themes.
   - Centralized layout constants (`HIGHLIGHT_PAD_X`, `HIGHLIGHT_PAD_Y`, `HIGHLIGHT_CORNER_RADIUS`, `DEFAULT_HEADER_HEIGHT`, `DEFAULT_GUTTER_WIDTH`, `DEFAULT_COLUMN_GUTTER_THRESHOLD`, `DEFAULT_BASELINE_TOLERANCE`).
4. **Typed Domain Errors (`PdfVdiffError`)**:
   - Modeled domain failure variants using `thiserror` to cleanly distinguish missing files, encrypted documents, scanned zero-text PDFs, and rendering errors.

## Alternatives Considered & Rejected
- *Screen Coordinates (Top-Left Origin)*: Rejected because PDFium and PDF page composition inherently use bottom-left coordinate systems; keeping native coordinate frames avoids dual-conversion overhead and vertical inversion bugs.
