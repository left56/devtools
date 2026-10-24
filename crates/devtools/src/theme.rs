use gpui::{App, Global, Rgba, Window, WindowAppearance, rgb};

/// Surfaces rise: `bg` < `sidebar` ≈ page chrome < `panel` < `elevated`.
///
/// Personality (Refactoring UI): **developer tools = casual + reserved**.
/// Cool neutrals, one muted blue accent, dense but ordered chrome. No purple
/// default, no pure black/white, no decorative gradients.
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
    /// Lower contrast than `muted` — category labels, hints, placeholders.
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

/// Spacing scale (px). Prefer these over ad-hoc values.
pub mod space {
    pub const XS: f32 = 4.;
    pub const SM: f32 = 8.;
    pub const MD: f32 = 12.;
    pub const LG: f32 = 16.;
    pub const XL: f32 = 24.;
    pub const XXL: f32 = 32.;
}

/// Corner radius matched to casual+reserved (moderate, not pill-everything).
pub mod radius {
    pub const SM: f32 = 4.;
    pub const MD: f32 = 6.;
    pub const LG: f32 = 8.;
}

impl Theme {
    /// Dark: cool slate surfaces, reduced-sat accent (colors punch harder on dark).
    pub fn dark() -> Self {
        Self {
            // slate-ish near-black — never #000
            bg: rgb(0x0f1218),
            sidebar: rgb(0x13171f),
            panel: rgb(0x1a1f29),
            elevated: rgb(0x222834),
            border: rgb(0x2c3442),
            border_subtle: rgb(0x232936),
            // slate-100, not pure white
            text: rgb(0xe8edf5),
            muted: rgb(0x8b95a8),
            faint: rgb(0x5c6678),
            // Soft blue, sat dialed back for dark
            accent: rgb(0x6b93e8),
            accent_muted: rgb(0x2a3a5a),
            accent_fg: rgb(0x0c1018),
            accent_hover: rgb(0x85a8f0),
            success: rgb(0x6fbf8f),
            success_bg: rgb(0x16241c),
            warning: rgb(0xd4b06a),
            warning_bg: rgb(0x241e14),
            error: rgb(0xe09090),
            error_bg: rgb(0x261818),
            selected: rgb(0x1e2a40),
            is_dark: true,
        }
    }

    /// Light: designed cool neutrals — not an inverted dark. White panels on
    /// cool gray-blue canvas; body text near-black slate for WCAG AA.
    pub fn light() -> Self {
        Self {
            bg: rgb(0xeef1f6),
            sidebar: rgb(0xe4e8f0),
            panel: rgb(0xffffff),
            elevated: rgb(0xf7f8fb),
            border: rgb(0xc8d0dc),
            border_subtle: rgb(0xdce1ea),
            text: rgb(0x151a22),
            muted: rgb(0x5c6678),
            faint: rgb(0x8b95a8),
            // Slightly stronger blue for light-mode CTAs
            accent: rgb(0x3568d4),
            accent_muted: rgb(0xc5d4f0),
            accent_fg: rgb(0xffffff),
            accent_hover: rgb(0x2a56b8),
            success: rgb(0x157a3c),
            success_bg: rgb(0xe3f6ea),
            warning: rgb(0x9a6b10),
            warning_bg: rgb(0xfbf0d4),
            error: rgb(0xb42318),
            error_bg: rgb(0xfce8e6),
            selected: rgb(0xdce8fa),
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
