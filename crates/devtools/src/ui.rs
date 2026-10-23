//! Small shared chrome widgets for workbenches. No tool logic here.

use crate::theme::Theme;
use gpui::*;

pub fn button(id: &'static str, label: &'static str) -> Stateful<Div> {
    let theme = Theme::dark();
    div()
        .id(id)
        .cursor_pointer()
        .px_3()
        .py_2()
        .rounded_md()
        .text_sm()
        .whitespace_nowrap()
        .bg(theme.selected)
        .hover(move |s| s.bg(theme.border))
        .child(label)
}

pub fn chip(id: impl Into<SharedString>, label: impl Into<SharedString>, active: bool) -> Stateful<Div> {
    let theme = Theme::dark();
    let id = id.into();
    div()
        .id(id)
        .cursor_pointer()
        .px_2()
        .py_1()
        .rounded_md()
        .text_xs()
        .whitespace_nowrap()
        .bg(if active { theme.selected } else { theme.panel })
        .text_color(if active { theme.accent } else { theme.muted })
        .border_1()
        .border_color(theme.border)
        .hover(move |s| s.bg(theme.border))
        .child(label.into())
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

pub fn pane_header() -> Div {
    let theme = Theme::dark();
    div()
        .h(px(48.))
        .flex_shrink_0()
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .border_b_1()
        .border_color(theme.border)
}
