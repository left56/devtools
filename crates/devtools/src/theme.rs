use gpui::{Rgba, rgb};

/// Dark palette used by the workbench. Light theme is intentionally absent.
/// Surfaces rise: `bg` < `sidebar` ≈ page chrome < `panel` < `elevated`.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Rgba,
    pub sidebar: Rgba,
    pub panel: Rgba,
    pub elevated: Rgba,
    pub border: Rgba,
    pub border_subtle: Rgba,
    pub text: Rgba,
    pub muted: Rgba,
    pub accent: Rgba,
    pub accent_muted: Rgba,
    pub success: Rgba,
    pub success_bg: Rgba,
    pub warning: Rgba,
    pub warning_bg: Rgba,
    pub error: Rgba,
    pub error_bg: Rgba,
    pub selected: Rgba,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            // Near-black page; slightly lifted chrome and cards.
            bg: rgb(0x121214),
            sidebar: rgb(0x161618),
            panel: rgb(0x1e1e22),
            elevated: rgb(0x26262b),
            border: rgb(0x2e2e34),
            border_subtle: rgb(0x25252a),
            text: rgb(0xececf1),
            muted: rgb(0x8e8e99),
            // Soft blue accent — readable, not neon.
            accent: rgb(0x6b9eff),
            accent_muted: rgb(0x3a4a6a),
            success: rgb(0x7dcea0),
            success_bg: rgb(0x1a2a22),
            warning: rgb(0xe5c078),
            warning_bg: rgb(0x2a2418),
            error: rgb(0xf0a0a0),
            error_bg: rgb(0x2a1c1c),
            selected: rgb(0x243044),
        }
    }
}

/// Explicit fallback also avoids incorrect CJK glyphs in GPUI's default fallback.
pub fn monospace() -> gpui::Font {
    let mut font = gpui::font("Menlo");
    font.fallbacks = Some(gpui::FontFallbacks::from_fonts(vec![
        "PingFang SC".into(),
        ".SystemUIFont".into(),
    ]));
    font
}
