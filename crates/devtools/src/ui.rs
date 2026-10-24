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
        .child(label.into())
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

fn status_colors(theme: Theme, kind: StatusKind) -> (Rgba, Rgba, Rgba) {
    match kind {
        StatusKind::Neutral => (theme.muted, theme.elevated, theme.border_subtle),
        StatusKind::Accent => (theme.accent, theme.selected, theme.accent_muted),
        StatusKind::Warning => (theme.warning, theme.warning_bg, theme.border_subtle),
        StatusKind::Error => (theme.error, theme.error_bg, theme.border_subtle),
        StatusKind::Success => (theme.success, theme.success_bg, theme.border_subtle),
    }
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
        .child(text.into())
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
        .child(label.into())
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
                    .child(subtitle),
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
        .child(label.into())
}

/// Quiet empty / hint state for result panes.
pub fn empty_hint(theme: Theme, text: impl Into<SharedString>) -> Div {
    div()
        .px(px(space::LG))
        .py(px(space::XXL))
        .text_sm()
        .text_color(theme.faint)
        .child(text.into())
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
                .child(label.into()),
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
                    value
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
