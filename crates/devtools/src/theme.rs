use gpui::{App, Global, Rgba, Window, WindowAppearance, rgb};

/// Surfaces rise: `bg` < `sidebar` ≈ page chrome < `panel` < `elevated`.
///
/// Developer-tools personality (Refactoring UI): casual + reserved — muted
/// neutrals, one restrained blue accent, no purple-as-default.
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
    /// Lower contrast than `muted` — category labels, hints.
    pub faint: Rgba,
    pub accent: Rgba,
    pub accent_muted: Rgba,
    /// Text/icon color drawn on top of `accent` fills.
    pub accent_fg: Rgba,
    pub accent_hover: Rgba,
    pub success: Rgba,
    pub success_bg: Rgba,
    pub warning: Rgba,
    pub warning_bg: Rgba,
    pub error: Rgba,
    pub error_bg: Rgba,
    pub selected: Rgba,
    pub is_dark: bool,
}

/// App-global palette, refreshed from `WindowAppearance` (and optional force).
#[derive(Clone, Copy)]
pub struct ActiveTheme(pub Theme);

impl Global for ActiveTheme {}

impl Theme {
    pub fn dark() -> Self {
        Self {
            bg: rgb(0x121214),
            sidebar: rgb(0x161618),
            panel: rgb(0x1e1e22),
            elevated: rgb(0x26262b),
            border: rgb(0x2e2e34),
            border_subtle: rgb(0x25252a),
            text: rgb(0xececf1),
            muted: rgb(0x8e8e99),
            faint: rgb(0x5c5c66),
            // Soft blue — readable, not neon, not violet.
            accent: rgb(0x6b9eff),
            accent_muted: rgb(0x3a4a6a),
            accent_fg: rgb(0x0e1016),
            accent_hover: rgb(0x8bb0ff),
            success: rgb(0x7dcea0),
            success_bg: rgb(0x1a2a22),
            warning: rgb(0xe5c078),
            warning_bg: rgb(0x2a2418),
            error: rgb(0xf0a0a0),
            error_bg: rgb(0x2a1c1c),
            selected: rgb(0x243044),
            is_dark: true,
        }
    }

    /// Intentional light palette (not an inverted dark). Enough contrast for
    /// body text; category labels use `faint`.
    pub fn light() -> Self {
        Self {
            bg: rgb(0xf4f4f5),
            sidebar: rgb(0xececee),
            panel: rgb(0xffffff),
            elevated: rgb(0xfafafa),
            border: rgb(0xd4d4d8),
            border_subtle: rgb(0xe4e4e7),
            text: rgb(0x18181b),
            muted: rgb(0x71717a),
            faint: rgb(0xa1a1aa),
            accent: rgb(0x3b6fd9),
            accent_muted: rgb(0xc7d7f5),
            accent_fg: rgb(0xffffff),
            accent_hover: rgb(0x2f5bb8),
            success: rgb(0x15803d),
            success_bg: rgb(0xdcfce7),
            warning: rgb(0xa16207),
            warning_bg: rgb(0xfef3c7),
            error: rgb(0xb91c1c),
            error_bg: rgb(0xfee2e2),
            selected: rgb(0xdbeafe),
            is_dark: false,
        }
    }

    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
        }
    }

    /// Resolve palette for this window.
    ///
    /// `DEVTOOLS_FORCE_THEME=light|dark` overrides system appearance (screenshot
    /// / verify only). Otherwise follows `window.appearance()`.
    ///
    /// Linux X11: GPUI defaults to Light until xdg-desktop-portal reports a
    /// color-scheme; without a portal (e.g. bare Xvfb) the app stays Light.
    /// macOS follows system light/dark (including Vibrant*).
    pub fn resolve(window: &Window) -> Self {
        match std::env::var("DEVTOOLS_FORCE_THEME").ok().as_deref() {
            Some("light") => return Self::light(),
            Some("dark") => return Self::dark(),
            _ => {}
        }
        Self::for_appearance(window.appearance())
    }

    pub fn sync(window: &Window, cx: &mut App) {
        cx.set_global(ActiveTheme(Self::resolve(window)));
    }

    pub fn get(cx: &App) -> Self {
        cx.try_global::<ActiveTheme>()
            .map(|t| t.0)
            .unwrap_or_else(Self::dark)
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
