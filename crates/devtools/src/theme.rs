use gpui::{Rgba, rgb};

/// Dark palette used by the workbench. Light theme is intentionally absent.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Rgba,
    pub sidebar: Rgba,
    pub panel: Rgba,
    pub border: Rgba,
    pub text: Rgba,
    pub muted: Rgba,
    pub accent: Rgba,
    pub error: Rgba,
    pub selected: Rgba,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: rgb(0x1c1c1e),
            sidebar: rgb(0x141416),
            panel: rgb(0x242428),
            border: rgb(0x34343a),
            text: rgb(0xececf1),
            muted: rgb(0x9b9ba6),
            accent: rgb(0x7aa2ff),
            error: rgb(0xff9292),
            selected: rgb(0x2c364c),
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
