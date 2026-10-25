//! Shared chrome widgets. Design follows Refactoring UI (casual + reserved):
//! clear hierarchy, restrained accent, dense but ordered spacing, quiet chips.

use crate::theme::{Theme, radius, space};
use gpui::{prelude::FluentBuilder, *};

/// Shared toolbar control height so primary / secondary / paste align in pane headers.
const TOOLBAR_BTN_H: f32 = 28.;

/// Primary action (运行 / 生成). Solid accent — the only loud control in a view.
pub fn primary_button(theme: Theme, id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .cursor_pointer()
        .h(px(TOOLBAR_BTN_H))
        .px(px(space::MD))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .bg(theme.accent)
        .text_color(theme.accent_fg)
        .hover(move |s| s.bg(theme.accent_hover))
        .child(label)
}

/// Secondary / outline action (清空、复制). Quiet so it never competes with CTAs.
pub fn button(theme: Theme, id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .cursor_pointer()
        .h(px(TOOLBAR_BTN_H))
        .px(px(space::MD))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .text_sm()
        .whitespace_nowrap()
        .bg(theme.elevated)
        .text_color(theme.muted)
        .border_1()
        .border_color(theme.border)
        .hover(move |s| s.bg(theme.selected).text_color(theme.text).border_color(theme.accent_muted))
        .child(label)
}

/// Compact option chip (字母表 / 时区 / 算法). Lower weight than buttons.
pub fn chip(
    theme: Theme,
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    active: bool,
) -> Stateful<Div> {
    let id = id.into();
    div()
        .id(id)
        .cursor_pointer()
        .px(px(space::SM))
        .py(px(2.))
        .rounded(px(radius::SM))
        .text_xs()
        .whitespace_nowrap()
        .bg(if active { theme.selected } else { theme.panel })
        .text_color(if active { theme.accent } else { theme.muted })
        .border_1()
        .border_color(if active {
            theme.accent_muted
        } else {
            theme.border_subtle
        })
        .hover(move |s| s.bg(theme.selected).text_color(theme.text))
        .child(single_line(label.into().as_ref()))
}

/// Non-interactive status pill (离线 · 已更新结果).
pub fn status_chip(theme: Theme, label: impl Into<SharedString>, kind: StatusKind) -> Div {
    let (fg, bg, border) = status_colors(theme, kind);
    div()
        .px(px(space::SM))
        .py(px(2.))
        .rounded(px(radius::SM))
        .text_xs()
        .whitespace_nowrap()
        .bg(bg)
        .text_color(fg)
        .border_1()
        .border_color(border)
        .child(single_line(label.into().as_ref()))
}

#[derive(Clone, Copy)]
pub enum StatusKind {
    Neutral,
    Accent,
    Warning,
    Error,
    Success,
}

fn status_colors(theme: Theme, kind: StatusKind) -> (Rgba, Rgba, Rgba) {
    match kind {
        StatusKind::Neutral => (theme.muted, theme.elevated, theme.border_subtle),
        StatusKind::Accent => (theme.accent, theme.selected, theme.accent_muted),
        StatusKind::Warning => (theme.warning, theme.warning_bg, theme.border_subtle),
        StatusKind::Error => (theme.error, theme.error_bg, theme.border_subtle),
        StatusKind::Success => (theme.success, theme.success_bg, theme.border_subtle),
    }
}


/// GPUI 0.2.2 panics when a plain text run contains `\\n` / `\\r`.
/// Flatten for chips, banners, titles, and other single-run labels.
pub fn single_line(text: impl AsRef<str>) -> SharedString {
    let flat: String = text
        .as_ref()
        .chars()
        .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
        .collect();
    SharedString::from(flat)
}

/// Multi-line monospace block: one GPUI text run per line (safe), capped for layout.
pub fn multiline_mono(theme: Theme, text: impl AsRef<str>) -> Div {
    let raw = text.as_ref();
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(1.))
        .min_w_0()
        .font(crate::theme::monospace())
        .text_sm()
        .text_color(theme.text);
    if raw.is_empty() {
        return col.child(
            div()
                .text_color(theme.faint)
                .child(SharedString::from("—")),
        );
    }
    const MAX_LINES: usize = 120;
    let total = raw.lines().count();
    for line in raw.lines().take(MAX_LINES) {
        let display = if line.is_empty() {
            SharedString::from(" ")
        } else {
            // lines() already strips separators; still flatten defensively.
            single_line(line)
        };
        col = col.child(div().min_w_0().child(display));
    }
    if total > MAX_LINES {
        col = col.child(
            div()
                .text_xs()
                .text_color(theme.muted)
                .child(SharedString::from(format!(
                    "…共 {total} 行，点复制可取全文"
                ))),
        );
    }
    col
}

/// Monospace art (QR ASCII, banners): no inter-line gap so the grid stays square.
pub fn multiline_mono_art(theme: Theme, text: impl AsRef<str>) -> Div {
    let raw = text.as_ref();
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(0.))
        .min_w_0()
        .font(crate::theme::monospace())
        .text_sm()
        .text_color(theme.text);
    if raw.is_empty() {
        return col.child(
            div()
                .text_color(theme.faint)
                .child(SharedString::from("—")),
        );
    }
    const MAX_LINES: usize = 200;
    let total = raw.lines().count();
    for line in raw.lines().take(MAX_LINES) {
        let display = if line.is_empty() {
            SharedString::from(" ")
        } else {
            single_line(line)
        };
        col = col.child(div().min_w_0().whitespace_nowrap().child(display));
    }
    if total > MAX_LINES {
        col = col.child(
            div()
                .text_xs()
                .text_color(theme.muted)
                .child(SharedString::from(format!(
                    "…共 {total} 行，点复制可取全文"
                ))),
        );
    }
    col
}

/// Paint an inline SVG string via GPUI's image decoder (usvg). Used for QR / placeholder.
pub fn svg_preview(svg: &str, side: f32) -> Img {
    let image = Image::from_bytes(ImageFormat::Svg, svg.as_bytes().to_vec());
    img(std::sync::Arc::new(image))
        .w(px(side))
        .h(px(side))
        .flex_shrink_0()
        .object_fit(ObjectFit::Contain)
}

/// True when a batch row should be painted as an SVG image instead of source text.
pub fn looks_like_svg_markup(label: &str, value: &str) -> bool {
    if label.eq_ignore_ascii_case("SVG") || label.eq_ignore_ascii_case("Data URL") {
        // Data URL is not paintables via ImageFormat::Svg bytes; only raw markup.
        if label.eq_ignore_ascii_case("Data URL") {
            return false;
        }
        return value.trim_start().starts_with("<svg")
            || value.trim_start().starts_with("<?xml");
    }
    let t = value.trim_start();
    (t.starts_with("<svg") || t.starts_with("<?xml")) && t.contains("<svg")
}

/// True for ASCII-art style rows that need tight monospace packing.
pub fn looks_like_ascii_art(label: &str, value: &str) -> bool {
    if label.eq_ignore_ascii_case("ASCII") || label.contains("横幅") || label.contains("ASCII") {
        return true;
    }
    // Heuristic: many short lines of #/block drawing
    let lines: Vec<&str> = value.lines().collect();
    lines.len() >= 8
        && lines.iter().all(|l| l.len() <= 120)
        && lines.iter().filter(|l| l.contains('#') || l.contains('█')).count() >= 4
}

/// Compact warning / error / info banner.
pub fn banner(theme: Theme, kind: StatusKind, text: impl Into<SharedString>) -> Div {
    let (fg, bg, border) = status_colors(theme, kind);
    div()
        .w_full()
        .px(px(space::MD))
        .py(px(space::SM))
        .rounded(px(radius::MD))
        .bg(bg)
        .border_1()
        .border_color(border)
        .text_sm()
        .text_color(fg)
        .child(single_line(text.into().as_ref()))
}

/// Full-height content pane (input / result columns).
pub fn pane(theme: Theme) -> Div {
    div()
        .h_full()
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(theme.panel)
        .rounded(px(radius::LG))
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
}

/// Content-sized card (JWT stacks, etc.) — no forced `h_full`.
pub fn card(theme: Theme) -> Div {
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .bg(theme.panel)
        .rounded(px(radius::LG))
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
}

/// Pane chrome header: muted label left, actions right. Dense (32px).
pub fn pane_header(theme: Theme) -> Div {
    div()
        .h(px(32.))
        .flex_shrink_0()
        .px(px(space::MD))
        .flex()
        .items_center()
        .gap(px(space::SM))
        .border_b_1()
        .border_color(theme.border_subtle)
        .bg(theme.elevated)
}

/// Muted pane title used inside `pane_header`.
pub fn pane_title(theme: Theme, label: impl Into<SharedString>) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.muted)
        .child(single_line(label.into().as_ref()))
}

/// Workbench page title row: title (semibold) > meta (muted) > trailing.
pub fn workbench_title(
    theme: Theme,
    title: impl Into<SharedString>,
    subtitle: impl Into<SharedString>,
) -> Div {
    let subtitle = subtitle.into();
    let show_sub = !subtitle.is_empty();
    div()
        .h(px(32.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(space::MD))
        .child(
            div()
                .text_lg()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text)
                .child(title.into()),
        )
        .when(show_sub, |view| {
            view.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(single_line(subtitle.as_ref())),
            )
        })
}

/// Sidebar / form section label — de-emphasized relative to values.
pub fn section_label(theme: Theme, label: impl Into<SharedString>) -> Div {
    div()
        .px(px(space::SM))
        .pt(px(space::MD))
        .pb(px(space::XS))
        .text_xs()
        .font_weight(FontWeight::MEDIUM)
        .text_color(theme.faint)
        .child(single_line(label.into().as_ref()))
}

/// Quiet empty / hint state for result panes.
pub fn empty_hint(theme: Theme, text: impl Into<SharedString>) -> Div {
    div()
        .px(px(space::LG))
        .py(px(space::XXL))
        .text_sm()
        .text_color(theme.faint)
        .child(single_line(text.into().as_ref()))
}

/// Fixed-width label + mono value box (URL Parse KV rows).
pub fn kv_row(
    theme: Theme,
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    empty: bool,
) -> Stateful<Div> {
    let value = value.into();
    div()
        .id(id.into())
        .flex()
        .items_center()
        .gap(px(space::SM))
        .px(px(space::MD))
        .py(px(space::XS))
        .child(
            div()
                .w(px(78.))
                .flex_shrink_0()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if empty { theme.faint } else { theme.muted })
                .child(single_line(label.into().as_ref())),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .px(px(space::SM))
                .py(px(space::XS))
                .rounded(px(radius::SM))
                .bg(theme.elevated)
                .border_1()
                .border_color(theme.border_subtle)
                .font(crate::theme::monospace())
                .text_sm()
                .text_color(if empty { theme.faint } else { theme.text })
                .child(if empty {
                    SharedString::from("—")
                } else {
                    single_line(value.as_ref())
                }),
        )
}

/// 「粘贴」 for input pane headers — same box size as `button`, accent outline.
pub fn paste_button(theme: Theme, id: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(id.into())
        .cursor_pointer()
        .h(px(TOOLBAR_BTN_H))
        .px(px(space::MD))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(radius::MD))
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .bg(theme.panel)
        .text_color(theme.accent)
        .border_1()
        .border_color(theme.accent_muted)
        .hover(move |s| s.bg(theme.selected))
        .child("粘贴")
}

#[cfg(test)]
mod tests {
    use super::{looks_like_ascii_art, looks_like_svg_markup, single_line};

    #[test]
    fn single_line_flattens_newlines() {
        let s = single_line("a\nb\rc");
        assert_eq!(s.as_ref(), "a b c");
    }

    #[test]
    fn detects_svg_rows() {
        assert!(looks_like_svg_markup(
            "SVG",
            r#"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg"></svg>"#
        ));
        assert!(!looks_like_svg_markup("Data URL", "data:image/svg+xml,%3Csvg"));
        assert!(!looks_like_svg_markup("明文", "hello"));
    }

    #[test]
    fn detects_ascii_art_rows() {
        let grid = "# #\n# #\n# #\n# #\n# #\n# #\n# #\n# #\n";
        assert!(looks_like_ascii_art("ASCII", grid));
        assert!(looks_like_ascii_art("ASCII 横幅", "x"));
        assert!(!looks_like_ascii_art("Hex", "deadbeef"));
    }
}
