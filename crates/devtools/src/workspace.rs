use crate::{
    actions::*,
    inspector_workbench::InspectorWorkbench,
    network_workbench::NetworkWorkbench,
    json_workbench::JsonWorkbench,
    prefs::{self, Prefs},
    text_field::{Edited, TextField},
    theme::Theme,
    transform_workbench::TransformWorkbench,
    ui::{button, chip, primary_button, status_chip, StatusKind},
};
use devtools_core::{Category, Detection, Registry, ToolMeta};
use gpui::{prelude::FluentBuilder, *};
use std::collections::BTreeSet;

pub struct Workspace {
    focus: FocusHandle,
    registry: Registry,
    collapsed_categories: BTreeSet<Category>,
    sidebar_collapsed: bool,
    selected: &'static str,
    workbench: Entity<JsonWorkbench>,
    transform: Entity<TransformWorkbench>,
    inspector: Entity<InspectorWorkbench>,
    network: Entity<NetworkWorkbench>,
    search: Entity<TextField>,
    palette_search: Entity<TextField>,
    palette_open: bool,
    palette_index: usize,
    previous_focus: Option<FocusHandle>,
    prefs: Prefs,
    /// Last one-shot clipboard read. Never watched in the background.
    last_clipboard: Option<String>,
    suggestions: Vec<Detection>,
    chrome_notice: &'static str,
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
            transform: cx.new(TransformWorkbench::new),
            inspector: cx.new(InspectorWorkbench::new),
            network: cx.new(NetworkWorkbench::new),
            search,
            palette_search,
            palette_open: false,
            palette_index: 0,
            previous_focus: None,
            prefs: prefs::load(),
            last_clipboard: None,
            suggestions: Vec::new(),
            chrome_notice: "",
        }
    }

    pub fn focus_editor(&self, window: &mut Window, cx: &App) {
        if self.selected == devtools_core::ids::JSON {
            self.workbench.read(cx).focus_input(window, cx);
        } else if devtools_tools::is_inspector_tool(self.selected) {
            self.inspector.read(cx).focus_input(window, cx);
        } else if devtools_tools::is_network_tool(self.selected) {
            self.network.read(cx).focus_input(window, cx);
        } else {
            self.transform.read(cx).focus_input(window, cx);
        }
    }

    fn persist_prefs(&self) {
        if let Err(err) = prefs::save(&self.prefs) {
            eprintln!("devtools: failed to save prefs: {err}");
        }
    }

    fn palette_tools(&self, cx: &App) -> Vec<ToolMeta> {
        let query = self.palette_search.read(cx).buffer.text();
        let searched: Vec<ToolMeta> = self
            .registry
            .search(query, None)
            .into_iter()
            .copied()
            .collect();
        if !query.trim().is_empty() {
            return searched;
        }
        // Empty query: recent tools first (still registered), then the rest.
        let mut ordered = Vec::new();
        let mut seen = BTreeSet::new();
        for id in self.prefs.recent_ids() {
            if let Some(tool) = self.registry.get(id)
                && seen.insert(tool.id)
            {
                ordered.push(*tool);
            }
        }
        for tool in searched {
            if seen.insert(tool.id) {
                ordered.push(tool);
            }
        }
        ordered
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
        self.prefs.touch_recent(id);
        self.persist_prefs();
        self.chrome_notice = "";
        if id == devtools_core::ids::JSON {
            self.focus_editor(window, cx);
        } else if devtools_tools::is_inspector_tool(id) {
            self.inspector
                .update(cx, |bench, cx| bench.set_tool(id, window, cx));
        } else if devtools_tools::is_network_tool(id) {
            self.network
                .update(cx, |bench, cx| bench.set_tool(id, window, cx));
        } else {
            self.transform
                .update(cx, |bench, cx| bench.set_tool(id, window, cx));
        }
        cx.notify();
    }

    fn toggle_favorite_action(
        &mut self,
        _: &ToggleFavorite,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.prefs.toggle_favorite(self.selected);
        self.persist_prefs();
        self.chrome_notice = if self.prefs.is_favorite(self.selected) {
            "已加入收藏"
        } else {
            "已取消收藏"
        };
        cx.notify();
    }

    fn toggle_favorite_id(&mut self, id: &'static str, cx: &mut Context<Self>) {
        self.prefs.toggle_favorite(id);
        self.persist_prefs();
        cx.notify();
    }

    fn read_clipboard(&mut self, _: &ReadClipboard, _window: &mut Window, cx: &mut Context<Self>) {
        let text = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        if text.is_empty() {
            self.last_clipboard = None;
            self.suggestions.clear();
            self.chrome_notice = "剪贴板为空";
        } else {
            self.suggestions = devtools_tools::applicable_suggestions(&self.registry, &text);
            self.last_clipboard = Some(text);
            self.chrome_notice = if self.suggestions.is_empty() {
                "已读取剪贴板 · 无匹配建议（可手动插入）"
            } else {
                "已读取剪贴板 · 点击建议仅切换工具，不改动输入"
            };
        }
        cx.notify();
    }

    fn insert_clipboard(
        &mut self,
        _: &InsertClipboard,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(text) = self.last_clipboard.clone() else {
            self.chrome_notice = "请先读取剪贴板";
            cx.notify();
            return;
        };
        self.insert_text_into_current(&text, window, cx);
        self.chrome_notice = "已插入剪贴板到输入";
        cx.notify();
    }

    fn insert_text_into_current(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected == devtools_core::ids::JSON {
            self.workbench
                .update(cx, |bench, cx| bench.set_input_text(text, window, cx));
        } else if devtools_tools::is_inspector_tool(self.selected) {
            self.inspector
                .update(cx, |bench, cx| bench.set_input_text(text, window, cx));
        } else if devtools_tools::is_network_tool(self.selected) {
            self.network
                .update(cx, |bench, cx| bench.set_input_text(text, window, cx));
        } else {
            self.transform
                .update(cx, |bench, cx| bench.set_input_text(text, window, cx));
        }
    }

    fn apply_suggestion(&mut self, tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        // Click suggestion → switch tool only. Keep editor contents (decision 5 / M4).
        self.select_tool(tool_id, window, cx);
        self.chrome_notice = "已切换工具 · 输入未改动；需要时可插入剪贴板";
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, window: &mut Window, cx: &mut Context<Self>) {
        if self.palette_open {
            self.close_palette(window, cx);
        }
        self.sidebar_collapsed = !self.sidebar_collapsed;
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

    fn tool_row(
        &self,
        tool: &ToolMeta,
        indent: bool,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let theme = Theme::dark();
        let id = tool.id;
        let favored = self.prefs.is_favorite(id);
        div()
            .id(SharedString::from(format!("tool-{id}")))
            .cursor_pointer()
            .when(indent, |s| s.ml_3())
            .px_2()
            .py_1p5()
            .rounded_md()
            .bg(if id == self.selected {
                theme.selected
            } else {
                theme.sidebar
            })
            .border_1()
            .border_color(if id == self.selected {
                theme.accent_muted
            } else {
                theme.sidebar
            })
            .hover(move |s| s.bg(theme.selected))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_xs().text_color(theme.accent).child("{ }"))
                    .child(div().text_sm().flex_1().child(tool.name))
                    .child(
                        div()
                            .id(SharedString::from(format!("fav-{id}")))
                            .cursor_pointer()
                            .px_1()
                            .text_xs()
                            .text_color(if favored {
                                theme.warning
                            } else {
                                theme.muted
                            })
                            .child(if favored { "★" } else { "☆" })
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_favorite_id(id, cx);
                            })),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| this.select_tool(id, window, cx)))
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
            .child(div().h(px(titlebar_pad())).flex_shrink_0())
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
                    .child(div().text_xs().text_color(theme.muted).child(palette_hint_short())),
            )
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.sidebar_collapsed {
            return self.compact_sidebar(cx).into_any_element();
        }
        let theme = Theme::dark();
        let query = self.search.read(cx).buffer.text().to_owned();
        let tools = self.registry.search(&query, None);
        let mut folders = Vec::new();

        // Favorites section (only when not searching, or when matches).
        let fav_tools: Vec<_> = self
            .prefs
            .favorites
            .iter()
            .filter_map(|id| self.registry.get(id))
            .filter(|tool| {
                query.trim().is_empty()
                    || tools.iter().any(|t| t.id == tool.id)
            })
            .collect();
        if !fav_tools.is_empty() {
            let rows: Vec<_> = fav_tools
                .into_iter()
                .map(|tool| self.tool_row(tool, true, cx))
                .collect();
            folders.push(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.warning)
                            .child("★  收藏"),
                    )
                    .children(rows),
            );
        }

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
            let rows: Vec<_> = matches
                .into_iter()
                .map(|tool| self.tool_row(tool, true, cx))
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
                            .py_1()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
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
            .w(px(220.))
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
                    .h(px(titlebar_pad()))
                    .flex_shrink_0()
                    .pl(px(sidebar_title_pl()))
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
                    .h(px(32.))
                    .px_2()
                    .py_1()
                    .bg(theme.elevated)
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
                    .child(palette_hint()),
            )
            .into_any_element()
    }

    fn efficiency_bar(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = Theme::dark();
        let has_clip = self.last_clipboard.as_ref().is_some_and(|t| !t.is_empty());
        let mut suggestions_row = div().flex().flex_wrap().items_center().gap_1();
        for detection in &self.suggestions {
            let id = detection.tool_id;
            let label = self
                .registry
                .get(id)
                .map(|t| t.name)
                .unwrap_or(id);
            let conf = match detection.confidence {
                devtools_core::Confidence::High => "高",
                devtools_core::Confidence::Medium => "中",
            };
            let chip_label = format!("打开 {label} · {conf}");
            suggestions_row = suggestions_row.child(
                chip(
                    SharedString::from(format!("suggest-{id}")),
                    SharedString::from(chip_label),
                    id == self.selected,
                )
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.apply_suggestion(id, window, cx);
                })),
            );
        }

        div()
            .id("efficiency-bar")
            .flex_shrink_0()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.elevated)
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        primary_button("read-clipboard", "读取剪贴板")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.read_clipboard(&ReadClipboard, window, cx)
                            })),
                    )
                    .child(
                        button("insert-clipboard", "插入到输入")
                            .when(!has_clip, |s| s.opacity(0.45))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_clipboard(&InsertClipboard, window, cx)
                            })),
                    )
                    .child(
                        button(
                            "toggle-fav",
                            if self.prefs.is_favorite(self.selected) {
                                "取消收藏"
                            } else {
                                "收藏此工具"
                            },
                        )
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.toggle_favorite_action(&ToggleFavorite, window, cx)
                        })),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(clipboard_hint()),
                    ),
            )
            .when(!self.suggestions.is_empty(), |view| {
                view.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.muted)
                                .child("建议"),
                        )
                        .child(suggestions_row)
                        .child(status_chip(
                            "点击只切换工具",
                            StatusKind::Neutral,
                        )),
                )
            })
            .when(!self.chrome_notice.is_empty(), |view| {
                view.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(self.chrome_notice),
                )
            })
    }

    fn palette(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let theme = Theme::dark();
        let tools = self.palette_tools(cx);
        let query_empty = self.palette_search.read(cx).buffer.text().trim().is_empty();
        let recent_count = if query_empty {
            self.prefs
                .recent_ids()
                .into_iter()
                .filter(|id| self.registry.contains(id))
                .count()
                .min(tools.len())
        } else {
            0
        };
        let rows: Vec<_> = tools
            .iter()
            .enumerate()
            .map(|(index, tool)| {
                let id = tool.id;
                let is_recent = query_empty && index < recent_count;
                div()
                    .id(SharedString::from(format!("palette-{id}-{index}")))
                    .cursor_pointer()
                    .p_3()
                    .rounded_md()
                    .bg(if self.palette_index == index {
                        theme.selected
                    } else {
                        theme.panel
                    })
                    .child(format!(
                        "{}{}  ·  {}",
                        if is_recent { "最近 · " } else { "" },
                        tool.name,
                        tool.category.label()
                    ))
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
                    .w(px(520.))
                    .max_h(px(480.))
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
                    .when(query_empty && recent_count > 0, |view| {
                        view.child(
                            div()
                                .text_xs()
                                .text_color(theme.accent)
                                .child(format!("最近使用（{recent_count}）")),
                        )
                    })
                    .child(
                        div()
                            .h(px(40.))
                            .p_2()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_md()
                            .child(self.palette_search.clone()),
                    )
                    .child(
                        div()
                            .id("palette-rows")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .children(rows),
                    )
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
            .on_action(cx.listener(Self::read_clipboard))
            .on_action(cx.listener(Self::insert_clipboard))
            .on_action(cx.listener(Self::toggle_favorite_action))
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
                    .flex()
                    .flex_col()
                    .child(self.efficiency_bar(cx))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .when(self.selected == devtools_core::ids::JSON, |view| {
                                view.child(self.workbench.clone())
                            })
                            .when(devtools_tools::is_inspector_tool(self.selected), |view| {
                                view.child(self.inspector.clone())
                            })
                            .when(devtools_tools::is_network_tool(self.selected), |view| {
                                view.child(self.network.clone())
                            })
                            .when(
                                self.selected != devtools_core::ids::JSON
                                    && !devtools_tools::is_inspector_tool(self.selected)
                                    && !devtools_tools::is_network_tool(self.selected),
                                |view| view.child(self.transform.clone()),
                            ),
                    ),
            )
            .when(self.palette_open, |view| view.child(self.palette(cx)))
    }
}

fn titlebar_pad() -> f32 {
    #[cfg(target_os = "macos")]
    {
        48.
    }
    #[cfg(not(target_os = "macos"))]
    {
        12.
    }
}

fn sidebar_title_pl() -> f32 {
    #[cfg(target_os = "macos")]
    {
        86.
    }
    #[cfg(not(target_os = "macos"))]
    {
        12.
    }
}

fn palette_hint() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "⌘K 搜索 · ⌘⇧V 读取剪贴板"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl+K 搜索 · Ctrl+Shift+V 读取剪贴板"
    }
}

fn palette_hint_short() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "⌘K"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl+K"
    }
}

fn clipboard_hint() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "⌘⇧V 读一次 · ⌘⇧I 插入 · 无后台监听"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl+Shift+V 读一次 · Ctrl+Shift+I 插入 · 无后台监听"
    }
}
