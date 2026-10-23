//! Small shared chrome widgets for workbenches. No tool logic here.

use crate::theme::Theme;
use gpui::*;

/// Primary action (运行 / 编码 / 格式化). Accent fill.
pub fn primary_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    let theme = Theme::dark();
    div()
        .id(id)
        .cursor_pointer()
        .px_3()
        .py_1()
        .rounded_md()
        .text_sm()
        .whitespace_nowrap()
        .bg(theme.accent)
        .text_color(rgb(0x0e1016))
        .hover(move |s| s.bg(rgb(0x8bb0ff)))
        .child(label)
}

/// Secondary / ghost action (清空、复制、次要操作).
pub fn button(id: &'static str, label: &'static str) -> Stateful<Div> {
    let theme = Theme::dark();
    div()
        .id(id)
        .cursor_pointer()
        .px_3()
        .py_1()
        .rounded_md()
        .text_sm()
        .whitespace_nowrap()
        .bg(theme.elevated)
        .text_color(theme.text)
        .border_1()
        .border_color(theme.border)
        .hover(move |s| s.bg(theme.selected).border_color(theme.accent_muted))
        .child(label)
}

/// Compact option chip (字母表 / 时区 / 算法).
pub fn chip(id: impl Into<SharedString>, label: impl Into<SharedString>, active: bool) -> Stateful<Div> {
    let theme = Theme::dark();
    let id = id.into();
    div()
        .id(id)
        .cursor_pointer()
        .px_2()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .whitespace_nowrap()
        .bg(if active { theme.selected } else { theme.panel })
        .text_color(if active { theme.accent } else { theme.muted })
        .border_1()
        .border_color(if active { theme.accent_muted } else { theme.border })
        .hover(move |s| s.bg(theme.selected))
        .child(label.into())
}

/// Non-interactive status pill (离线 · 已更新结果).
pub fn status_chip(label: impl Into<SharedString>, kind: StatusKind) -> Div {
    let theme = Theme::dark();
    let (fg, bg, border) = match kind {
        StatusKind::Neutral => (theme.muted, theme.elevated, theme.border),
        StatusKind::Accent => (theme.accent, theme.selected, theme.accent_muted),
        StatusKind::Warning => (theme.warning, theme.warning_bg, rgb(0x4a3c20)),
        StatusKind::Error => (theme.error, theme.error_bg, rgb(0x4a2a2a)),
        StatusKind::Success => (theme.success, theme.success_bg, rgb(0x2a3c30)),
    };
    div()
        .px_2()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .whitespace_nowrap()
        .bg(bg)
        .text_color(fg)
        .border_1()
        .border_color(border)
        .child(label.into())
}

#[derive(Clone, Copy)]
pub enum StatusKind {
    Neutral,
    Accent,
    Warning,
    Error,
    Success,
}

/// Compact warning / error banner for inspector results.
pub fn banner(kind: StatusKind, text: impl Into<SharedString>) -> Div {
    let theme = Theme::dark();
    let (fg, bg, border) = match kind {
        StatusKind::Warning => (theme.warning, theme.warning_bg, rgb(0x4a3c20)),
        StatusKind::Error => (theme.error, theme.error_bg, rgb(0x4a2a2a)),
        StatusKind::Success => (theme.success, theme.success_bg, rgb(0x2a3c30)),
        StatusKind::Accent => (theme.accent, theme.selected, theme.accent_muted),
        StatusKind::Neutral => (theme.muted, theme.elevated, theme.border),
    };
    div()
        .w_full()
        .px_3()
        .py_2()
        .rounded_md()
        .bg(bg)
        .border_1()
        .border_color(border)
        .text_sm()
        .text_color(fg)
        .child(text.into())
}

pub fn pane() -> Div {
    let theme = Theme::dark();
    div()
        .h_full()
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(theme.panel)
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
}

/// Content-sized card (does not force `h_full`). Used for stacked JWT panes.
pub fn card() -> Div {
    let theme = Theme::dark();
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .bg(theme.panel)
        .rounded_lg()
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
}

pub fn pane_header() -> Div {
    let theme = Theme::dark();
    div()
        .h(px(36.))
        .flex_shrink_0()
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .border_b_1()
        .border_color(theme.border)
        .bg(theme.elevated)
}

/// Fixed-width label + mono value box used by URL Parse (and similar KV UIs).
pub fn kv_row(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    empty: bool,
) -> Stateful<Div> {
    let theme = Theme::dark();
    let value = value.into();
    div()
        .id(id.into())
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1()
        .child(
            div()
                .w(px(78.))
                .flex_shrink_0()
                .text_xs()
                .text_color(if empty { theme.border } else { theme.muted })
                .child(label.into()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme.elevated)
                .border_1()
                .border_color(theme.border_subtle)
                .font(crate::theme::monospace())
                .text_sm()
                .text_color(if empty { theme.muted } else { theme.text })
                .child(if empty {
                    SharedString::from("—")
                } else {
                    value
                }),
        )
}
