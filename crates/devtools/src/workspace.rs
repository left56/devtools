use crate::{
    actions::*,
    json_workbench::JsonWorkbench,
    text_field::{Edited, TextField},
    theme::Theme,
};
use devtools_core::{Category, Registry, ToolMeta};
use gpui::{prelude::FluentBuilder, *};
use std::collections::BTreeSet;

pub struct Workspace {
    focus: FocusHandle,
    registry: Registry,
    collapsed_categories: BTreeSet<Category>,
    sidebar_collapsed: bool,
    selected: &'static str,
    workbench: Entity<JsonWorkbench>,
    search: Entity<TextField>,
    palette_search: Entity<TextField>,
    palette_open: bool,
    palette_index: usize,
    previous_focus: Option<FocusHandle>,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextField::new(false, false, "搜索工具", cx));
        let palette_search = cx.new(|cx| TextField::new(false, false, "搜索全部工具…", cx));
        cx.subscribe(&search, |_, _, _: &Edited, cx| cx.notify())
            .detach();
        cx.subscribe(&palette_search, |this, _, _: &Edited, cx| {
            this.palette_index = 0;
            cx.notify();
        })
        .detach();
        Self {
            focus: cx.focus_handle(),
            registry: Registry::new(devtools_tools::catalog()),
            collapsed_categories: BTreeSet::new(),
            sidebar_collapsed: false,
            selected: devtools_core::ids::JSON,
            workbench: cx.new(JsonWorkbench::new),
            search,
            palette_search,
            palette_open: false,
            palette_index: 0,
            previous_focus: None,
        }
    }
    pub fn focus_editor(&self, window: &mut Window, cx: &App) {
        self.workbench.read(cx).focus_input(window, cx);
    }
    fn palette_tools(&self, cx: &App) -> Vec<ToolMeta> {
        self.registry
            .search(self.palette_search.read(cx).buffer.text(), None)
            .into_iter()
            .copied()
            .collect()
    }
    fn open_palette(&mut self, _: &TogglePalette, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette_open {
            self.close_palette(window, cx);
            return;
        }
        self.previous_focus = window.focused(cx);
        self.palette_search.update(cx, |field, cx| {
            field.set_output("", cx);
            field.focus(window);
        });
        self.palette_index = 0;
        self.palette_open = true;
        cx.notify();
    }
    fn close_palette(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.palette_open = false;
        if let Some(focus) = self.previous_focus.take() {
            focus.focus(window);
        } else {
            self.focus_editor(window, cx);
        }
        cx.notify();
    }
    fn select_tool(&mut self, id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = id;
        self.palette_open = false;
        self.previous_focus = None;
        self.focus_editor(window, cx);
        cx.notify();
    }
    fn toggle_sidebar(&mut self, _: &ToggleSidebar, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette_open {
            self.close_palette(window, cx);
        }
        self.sidebar_collapsed = !self.sidebar_collapsed;
        // Never leave keyboard focus in a search field that is no longer visible.
        if self.sidebar_collapsed && self.search.read(cx).focus.is_focused(window) {
            self.focus_editor(window, cx);
        }
        cx.notify();
    }
    fn sidebar_toggle(&self, compact: bool, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = Theme::dark();
        div()
            .id("toggle-sidebar")
            .cursor_pointer()
            .flex_shrink_0()
            .px_2()
            .py_1()
            .rounded_md()
            .text_sm()
            .text_color(theme.muted)
            .hover(move |s| s.bg(theme.border).text_color(theme.text))
            .child(if compact { "展开 ›" } else { "‹" })
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_click(
                cx.listener(|this, _, window, cx| this.toggle_sidebar(&ToggleSidebar, window, cx)),
            )
    }
    fn compact_sidebar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::dark();
        let tools: Vec<_> = self
            .registry
            .tools()
            .iter()
            .map(|tool| {
                let id = tool.id;
                div()
                    .id(SharedString::from(format!("compact-tool-{id}")))
                    .w_full()
                    .px_2()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .bg(if id == self.selected {
                        theme.selected
                    } else {
                        theme.sidebar
                    })
                    .hover(move |s| s.bg(theme.selected))
                    .child(div().text_color(theme.accent).child("{ }"))
                    .child(div().text_xs().child(tool.name))
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.select_tool(id, window, cx)),
                    )
            })
            .collect();
        div()
            .w(px(80.))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .bg(theme.sidebar)
            .border_r_1()
            .border_color(theme.border)
            // Native traffic lights occupy the first 48 px even when collapsed.
            .child(div().h(px(48.)).flex_shrink_0())
            .child(
                div()
                    .p_2()
                    .flex()
                    .justify_center()
                    .child(self.sidebar_toggle(true, cx)),
            )
            .child(
                div()
                    .id("compact-tools")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_2()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(tools),
            )
            .child(
                div()
                    .p_2()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .id("compact-search")
                            .w_full()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .text_sm()
                            .flex()
                            .justify_center()
                            .hover(move |s| s.bg(theme.border))
                            .child("搜索")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_palette(&TogglePalette, window, cx)
                            })),
                    )
                    .child(div().text_xs().text_color(theme.muted).child("⌘K")),
            )
    }
    fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.sidebar_collapsed {
            return self.compact_sidebar(cx).into_any_element();
        }
        let theme = Theme::dark();
        let query = self.search.read(cx).buffer.text();
        let tools = self.registry.search(query, None);
        let mut folders = Vec::new();
        for category in self.registry.categories_in_use() {
            let matches: Vec<_> = tools
                .iter()
                .filter(|tool| tool.category == category)
                .collect();
            if matches.is_empty() {
                continue;
            }
            let expanded =
                !query.trim().is_empty() || !self.collapsed_categories.contains(&category);
            let rows: Vec<_> =
                matches
                    .into_iter()
                    .map(|tool| {
                        let id = tool.id;
                        div()
                            .id(SharedString::from(format!("tool-{id}")))
                            .cursor_pointer()
                            .ml_4()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(if id == self.selected {
                                theme.selected
                            } else {
                                theme.sidebar
                            })
                            .hover(move |s| s.bg(theme.selected))
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(div().text_color(theme.accent).child("{ }"))
                                    .child(tool.name),
                            )
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.select_tool(id, window, cx)
                            }))
                    })
                    .collect();
            folders.push(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .id(SharedString::from(format!("folder-{}", category.id())))
                            .cursor_pointer()
                            .px_2()
                            .py_2()
                            .text_sm()
                            .text_color(theme.muted)
                            .child(format!(
                                "{}  {}",
                                if expanded { "▾" } else { "▸" },
                                category.label()
                            ))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if !this.collapsed_categories.remove(&category) {
                                    this.collapsed_categories.insert(category);
                                }
                                cx.notify();
                            })),
                    )
                    .when(expanded, |view| view.children(rows)),
            );
        }
        div()
            .w(px(210.))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .bg(theme.sidebar)
            .border_r_1()
            .border_color(theme.border)
            .child(
                div()
                    .id("sidebar-titlebar")
                    .h(px(48.))
                    .flex_shrink_0()
                    .pl(px(86.))
                    .pr_2()
                    .gap_1()
                    .flex()
                    .items_center()
                    .text_base()
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down(MouseButton::Left, |event, window, _| {
                        if event.click_count == 2 {
                            window.titlebar_double_click();
                        }
                    })
                    .child(div().flex_1().child("Devtools"))
                    .child(self.sidebar_toggle(false, cx)),
            )
            .child(
                div()
                    .mx_3()
                    .mb_2()
                    .h(px(36.))
                    .p_2()
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.border)
                    .rounded_md()
                    .child(self.search.clone()),
            )
            .child(
                div()
                    .id("tools")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_2()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .when(folders.is_empty(), |view| {
                        view.child(
                            div()
                                .p_2()
                                .text_sm()
                                .text_color(theme.muted)
                                .child("没有匹配的工具"),
                        )
                    })
                    .children(folders),
            )
            .child(
                div()
                    .p_3()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("⌘K 搜索全部工具"),
            )
            .into_any_element()
    }
    fn palette(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = Theme::dark();
        let tools = self.palette_tools(cx);
        let rows: Vec<_> = tools
            .iter()
            .enumerate()
            .map(|(index, tool)| {
                let id = tool.id;
                div()
                    .id(SharedString::from(format!("palette-{id}")))
                    .cursor_pointer()
                    .p_3()
                    .rounded_md()
                    .bg(if self.palette_index == index {
                        theme.selected
                    } else {
                        theme.panel
                    })
                    .child(format!("{}  ·  {}", tool.name, tool.category.label()))
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.select_tool(id, window, cx)),
                    )
            })
            .collect();
        div()
            .id("palette-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x00000099))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| this.close_palette(window, cx)),
            )
            .child(
                div()
                    .id("command-palette")
                    .key_context("CommandPalette")
                    .absolute()
                    .top(px(72.))
                    .left(px(280.))
                    .w(px(500.))
                    .p_3()
                    .rounded_lg()
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted)
                            .child("搜索工具 · ↑ ↓ 选择 · 回车打开 · Esc 关闭"),
                    )
                    .child(
                        div()
                            .h(px(40.))
                            .p_2()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_md()
                            .child(self.palette_search.clone()),
                    )
                    .children(rows)
                    .when(tools.is_empty(), |view| {
                        view.child(
                            div()
                                .p_3()
                                .text_sm()
                                .text_color(theme.muted)
                                .child("没有匹配的工具"),
                        )
                    }),
            )
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        div()
            .id("workspace-root")
            .relative()
            .track_focus(&self.focus)
            .size_full()
            .flex()
            .bg(theme.bg)
            .text_color(theme.text)
            .on_action(|_: &CloseWindow, window, _| window.remove_window())
            .on_action(|_: &ToggleFullscreen, window, _| window.toggle_fullscreen())
            .on_action(|_: &MinimizeWindow, window, _| window.minimize_window())
            .on_action(cx.listener(Self::open_palette))
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(
                cx.listener(|this, _: &PaletteClose, window, cx| this.close_palette(window, cx)),
            )
            .on_action(cx.listener(|this, _: &PaletteUp, _, cx| {
                this.palette_index = this.palette_index.saturating_sub(1);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PaletteDown, _, cx| {
                this.palette_index =
                    (this.palette_index + 1).min(this.palette_tools(cx).len().saturating_sub(1));
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &PaletteConfirm, window, cx| {
                if let Some(tool) = this.palette_tools(cx).get(this.palette_index) {
                    this.select_tool(tool.id, window, cx);
                }
            }))
            .child(self.sidebar(cx))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .when(self.selected == devtools_core::ids::JSON, |view| {
                        view.child(self.workbench.clone())
                    }),
            )
            .when(self.palette_open, |view| view.child(self.palette(cx)))
    }
}
