//! Visual diff themes and styling palettes for `pdf-vdiff`.

use clap::ValueEnum;

/// Default horizontal highlight padding in PDF points.
pub const HIGHLIGHT_PAD_X: f32 = 0.5;

/// Default vertical highlight padding in PDF points.
pub const HIGHLIGHT_PAD_Y: f32 = 1.0;

/// Default highlight rectangle corner radius in PDF points.
pub const HIGHLIGHT_CORNER_RADIUS: f32 = 1.5;

/// Default top header banner height in PDF points.
pub const DEFAULT_HEADER_HEIGHT: f32 = 36.0;

/// Default gutter spacing between left and right pages in PDF points.
pub const DEFAULT_GUTTER_WIDTH: f32 = 24.0;

/// Default horizontal gap threshold for column detection in PDF points.
pub const DEFAULT_COLUMN_GUTTER_THRESHOLD: f32 = 12.0;

/// Default vertical baseline grouping tolerance in PDF points.
pub const DEFAULT_BASELINE_TOLERANCE: f32 = 2.0;

/// Full RGBA color palette definition for rendering visual diffs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiffTheme {
    pub name: &'static str,
    pub deletion_fill: [f32; 4],
    pub deletion_stroke: [f32; 4],
    pub addition_fill: [f32; 4],
    pub addition_stroke: [f32; 4],
    pub replaced_line_tint: [f32; 4],
    pub gutter_color: [f32; 4],
    pub header_bg: [f32; 4],
    pub header_border: [f32; 4],
    pub header_title_color: [f32; 4],
    pub header_meta_color: [f32; 4],
    pub placeholder_bg: [f32; 4],
    pub placeholder_border: [f32; 4],
    pub placeholder_text: [f32; 4],
}

/// Supported theme variants selectable via `--theme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum ThemeKind {
    /// Soft IntelliJ-style red/green diff colors (Default).
    #[default]
    // Primary identifier is "intellij"; "intelli-j" is provided as a kebab-case alias
    #[value(name = "intellij", alias = "intelli-j")]
    IntelliJ,
    /// Clean GitHub-style split diff colors.
    #[value(name = "github", alias = "git-hub")]
    GitHub,
    /// Traditional vibrant red/green diff colors.
    #[value(name = "classic")]
    Classic,
    /// High-contrast palette for maximum visual accessibility.
    #[value(name = "high-contrast")]
    HighContrast,
}

impl ThemeKind {
    /// Returns the complete theme specification for this variant.
    pub fn theme(&self) -> &'static DiffTheme {
        match self {
            ThemeKind::IntelliJ => &INTELLIJ_THEME,
            ThemeKind::GitHub => &GITHUB_THEME,
            ThemeKind::Classic => &CLASSIC_THEME,
            ThemeKind::HighContrast => &HIGH_CONTRAST_THEME,
        }
    }
}

/// Helper function to convert 8-bit float RGB values in `[0.0..255.0]` and normalized alpha `[0.0..1.0]`
/// into normalized `[0.0..1.0]` RGBA float channels for PDFium rendering.
const fn rgba(r: f32, g: f32, b: f32, a: f32) -> [f32; 4] {
    [r / 255.0, g / 255.0, b / 255.0, a]
}

/// Helper function to convert 8-bit float RGB values in `[0.0..255.0]` to opaque normalized `[0.0..1.0]` RGBA floats.
const fn rgb(r: f32, g: f32, b: f32) -> [f32; 4] {
    rgba(r, g, b, 1.0)
}

/// IntelliJ Theme Palette (Default).
pub static INTELLIJ_THEME: DiffTheme = DiffTheme {
    name: "intellij",
    deletion_fill: rgba(255.0, 180.0, 180.0, 0.40),
    deletion_stroke: rgb(229.0, 115.0, 115.0),
    addition_fill: rgba(180.0, 235.0, 195.0, 0.40),
    addition_stroke: rgb(129.0, 199.0, 132.0),
    replaced_line_tint: rgba(255.0, 235.0, 150.0, 0.20),
    gutter_color: rgb(220.0, 224.0, 230.0),
    header_bg: rgb(245.0, 247.0, 250.0),
    header_border: rgb(215.0, 220.0, 228.0),
    header_title_color: rgb(33.0, 37.0, 41.0),
    header_meta_color: rgb(108.0, 117.0, 125.0),
    placeholder_bg: rgb(250.0, 250.0, 252.0),
    placeholder_border: rgb(200.0, 204.0, 210.0),
    placeholder_text: rgb(140.0, 145.0, 155.0),
};

/// GitHub Split Diff Palette.
pub static GITHUB_THEME: DiffTheme = DiffTheme {
    name: "github",
    deletion_fill: rgba(255.0, 205.0, 210.0, 0.45),
    deletion_stroke: rgb(215.0, 58.0, 73.0),
    addition_fill: rgba(200.0, 240.0, 205.0, 0.45),
    addition_stroke: rgb(40.0, 167.0, 69.0),
    replaced_line_tint: rgba(255.0, 245.0, 180.0, 0.25),
    gutter_color: rgb(209.0, 213.0, 218.0),
    header_bg: rgb(246.0, 248.0, 250.0),
    header_border: rgb(209.0, 213.0, 218.0),
    header_title_color: rgb(36.0, 41.0, 47.0),
    header_meta_color: rgb(87.0, 96.0, 106.0),
    placeholder_bg: rgb(246.0, 248.0, 250.0),
    placeholder_border: rgb(209.0, 213.0, 218.0),
    placeholder_text: rgb(140.0, 149.0, 159.0),
};

/// Classic Vibrant Palette.
pub static CLASSIC_THEME: DiffTheme = DiffTheme {
    name: "classic",
    deletion_fill: rgba(255.0, 128.0, 128.0, 0.50),
    deletion_stroke: rgb(200.0, 0.0, 0.0),
    addition_fill: rgba(128.0, 230.0, 128.0, 0.50),
    addition_stroke: rgb(0.0, 160.0, 0.0),
    replaced_line_tint: rgba(255.0, 255.0, 140.0, 0.30),
    gutter_color: rgb(180.0, 180.0, 180.0),
    header_bg: rgb(240.0, 240.0, 240.0),
    header_border: rgb(180.0, 180.0, 180.0),
    header_title_color: rgb(0.0, 0.0, 0.0),
    header_meta_color: rgb(100.0, 100.0, 100.0),
    placeholder_bg: rgb(245.0, 245.0, 245.0),
    placeholder_border: rgb(180.0, 180.0, 180.0),
    placeholder_text: rgb(120.0, 120.0, 120.0),
};

/// High-Contrast Palette (WCAG AAA Compliant).
pub static HIGH_CONTRAST_THEME: DiffTheme = DiffTheme {
    name: "high-contrast",
    deletion_fill: rgba(255.0, 90.0, 90.0, 0.70),
    deletion_stroke: rgb(180.0, 0.0, 0.0),
    addition_fill: rgba(80.0, 220.0, 80.0, 0.70),
    addition_stroke: rgb(0.0, 120.0, 0.0),
    replaced_line_tint: rgba(255.0, 240.0, 50.0, 0.45),
    gutter_color: rgb(100.0, 100.0, 100.0),
    header_bg: rgb(230.0, 230.0, 230.0),
    header_border: rgb(100.0, 100.0, 100.0),
    header_title_color: rgb(0.0, 0.0, 0.0),
    header_meta_color: rgb(50.0, 50.0, 50.0),
    placeholder_bg: rgb(240.0, 240.0, 240.0),
    placeholder_border: rgb(100.0, 100.0, 100.0),
    placeholder_text: rgb(60.0, 60.0, 60.0),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_retrieval() {
        assert_eq!(ThemeKind::IntelliJ.theme().name, "intellij");
        assert_eq!(ThemeKind::GitHub.theme().name, "github");
        assert_eq!(ThemeKind::Classic.theme().name, "classic");
        assert_eq!(ThemeKind::HighContrast.theme().name, "high-contrast");
    }

    #[test]
    fn test_alpha_ranges() {
        for kind in [
            ThemeKind::IntelliJ,
            ThemeKind::GitHub,
            ThemeKind::Classic,
            ThemeKind::HighContrast,
        ] {
            let t = kind.theme();
            assert!(t.deletion_fill[3] > 0.0 && t.deletion_fill[3] <= 1.0);
            assert!(t.addition_fill[3] > 0.0 && t.addition_fill[3] <= 1.0);
            assert!(t.replaced_line_tint[3] > 0.0 && t.replaced_line_tint[3] <= 1.0);
        }
    }
}
