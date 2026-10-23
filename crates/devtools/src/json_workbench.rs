use crate::{
    actions::*,
    json_run::{JsonAction, JsonRun, run_json},
    text_field::{Edited, TextField},
    theme::Theme,
    tree_state::TreeState,
    ui::{button, pane, pane_header},
};
use devtools_tools::json::{JsonDocument, JsonKind};
use gpui::{prelude::FluentBuilder, *};

pub struct JsonWorkbench {
    pub input: Entity<TextField>,
    output: Entity<TextField>,
    path: Entity<TextField>,
    jq: Entity<TextField>,
    run: JsonRun,
    last_input: Option<String>,
    tree: TreeState,
    tree_focus: FocusHandle,
    tree_scroll: ScrollHandle,
    text_view: bool,
    notice: &'static str,
    split: f32,
    pane_width: f32,
    resizing: Option<(Pixels, f32)>,
}

impl JsonWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextField::new(true, false, "在这里粘贴 JSON", cx));
        cx.subscribe(&input, |this, _, _: &Edited, cx| {
            this.notice = "";
            cx.notify();
        })
        .detach();
        Self {
            input,
            output: cx.new(|cx| TextField::new(true, true, "运行后在这里显示结果", cx)),
            path: cx.new(|cx| TextField::new(false, true, "", cx)),
            jq: cx.new(|cx| TextField::new(false, true, "", cx)),
            run: JsonRun::default(),
            last_input: None,
            tree: TreeState::default(),
            tree_focus: cx.focus_handle(),
            tree_scroll: ScrollHandle::new(),
            text_view: false,
            notice: "",
            split: 0.48,
            pane_width: 1000.,
            resizing: None,
        }
    }
    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        self.input.read(cx).focus(window);
    }
    fn execute(&mut self, action: JsonAction, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input.read(cx).buffer.text().to_owned();
        self.run = run_json(&input, action);
        self.last_input = Some(input);
        let result = self.run.output.as_deref().unwrap_or("");
        self.output
            .update(cx, |field, cx| field.set_output(result, cx));
        if let Some(doc) = &self.run.document {
            self.tree.reset(doc);
        }
        self.tree_scroll.set_offset(point(px(0.), px(0.)));
        self.sync_inspector(cx);
        if let Some(byte) = self.run.error_offset {
            self.input.update(cx, |field, cx| {
                field.reveal(byte, cx);
                field.focus(window);
            });
        }
        self.text_view = matches!(action, JsonAction::Minify);
        self.notice = "";
        cx.notify();
    }
    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |field, cx| {
            field.clear(cx);
            field.focus(window);
        });
        self.output.update(cx, |field, cx| field.set_output("", cx));
        self.run = JsonRun::default();
        self.last_input = None;
        self.tree = TreeState::default();
        self.sync_inspector(cx);
        self.notice = "";
        cx.notify();
    }
    fn sync_inspector(&self, cx: &mut Context<Self>) {
        let node = self
            .run
            .document
            .as_ref()
            .and_then(|doc| doc.node(self.tree.selected));
        self.path.update(cx, |field, cx| {
            field.set_output(node.map_or("", |n| n.json_path.as_str()), cx)
        });
        self.jq.update(cx, |field, cx| {
            field.set_output(node.map_or("", |n| n.jq.as_str()), cx)
        });
    }
    fn update_tree(
        &mut self,
        update: impl FnOnce(&mut TreeState, &JsonDocument),
        cx: &mut Context<Self>,
    ) {
        if let Some(doc) = &self.run.document {
            update(&mut self.tree, doc);
            if let Some(index) = self
                .tree
                .visible(doc)
                .iter()
                .position(|(id, _)| *id == self.tree.selected)
            {
                self.tree_scroll.scroll_to_item(index);
            }
        }
        self.sync_inspector(cx);
        self.notice = "";
        cx.notify();
    }
    fn copy_value(&mut self, cx: &mut Context<Self>) {
        if let Some(value) = self
            .run
            .document
            .as_ref()
            .and_then(|doc| doc.node_value(self.tree.selected))
        {
            cx.write_to_clipboard(ClipboardItem::new_string(value));
            self.notice = "已复制节点值";
            cx.notify();
        }
    }
    fn tree_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::dark();
        let Some(doc) = &self.run.document else {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .text_color(theme.muted)
                .child(div().text_3xl().text_color(theme.accent).child("{ }"))
                .child("格式化后查看 JSON 树")
                .child(div().text_xs().child("展开节点 · 选择路径 · 聚焦子树"))
                .into_any_element();
        };
        let rows: Vec<_> = self
            .tree
            .visible(doc)
            .into_iter()
            .map(|(id, depth)| {
                let node = &doc.nodes()[id];
                let has_children = !node.children.is_empty();
                let is_selected = id == self.tree.selected;
                let color = match node.kind {
                    JsonKind::String => rgb(0xa8d6aa),
                    JsonKind::Number => rgb(0xe5c78b),
                    JsonKind::Boolean | JsonKind::Null => rgb(0xc5a5e8),
                    _ => theme.muted,
                };
                div()
                    .id(SharedString::from(format!("node-{id}")))
                    .h(px(30.))
                    .flex_shrink_0()
                    .w_full()
                    .pl(px(8. + depth as f32 * 18.))
                    .pr_2()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded_sm()
                    .cursor_pointer()
                    .bg(if id == self.tree.selected {
                        theme.selected
                    } else {
                        theme.panel
                    })
                    .hover(move |s| {
                        s.bg(if is_selected {
                            theme.selected
                        } else {
                            theme.border
                        })
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.tree_focus.focus(window);
                        this.update_tree(|tree, _| tree.selected = id, cx);
                    }))
                    .child(
                        div()
                            .id(SharedString::from(format!("toggle-{id}")))
                            .w(px(16.))
                            .flex_shrink_0()
                            .text_color(theme.muted)
                            .child(if has_children {
                                if self.tree.expanded(id) { "▾" } else { "▸" }
                            } else {
                                "·"
                            })
                            .on_click(cx.listener(move |this, _, window, cx| {
                                cx.stop_propagation();
                                this.tree_focus.focus(window);
                                this.update_tree(
                                    |tree, _| {
                                        tree.selected = id;
                                        if has_children {
                                            tree.toggle(id);
                                        }
                                    },
                                    cx,
                                );
                            })),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .max_w(relative(0.5))
                            .truncate()
                            .child(node.label.clone()),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_color(color)
                            .child(node.preview.clone()),
                    )
            })
            .collect();
        let selected = &doc.nodes()[self.tree.selected];
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        button("expand-all", "展开").on_click(cx.listener(|this, _, _, cx| {
                            this.update_tree(|tree, _| tree.expand_all(), cx)
                        })),
                    )
                    .child(button("collapse-all", "收起").on_click(cx.listener(
                        |this, _, _, cx| this.update_tree(|tree, doc| tree.collapse_all(doc), cx),
                    )))
                    .when(self.tree.scope != 0, |view| {
                        view.child(button("tree-root", "返回根").on_click(cx.listener(
                            |this, _, _, cx| this.update_tree(|tree, _| tree.root(), cx),
                        )))
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(format!("{} 个节点", doc.nodes().len())),
                    ),
            )
            .child(
                div()
                    .id("json-tree")
                    .key_context("JsonTree")
                    .track_focus(&self.tree_focus)
                    .flex_1()
                    .min_h_0()
                    .p_2()
                    .text_sm()
                    .font(crate::theme::monospace())
                    .overflow_y_scroll()
                    .track_scroll(&self.tree_scroll)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, _| this.tree_focus.focus(window)),
                    )
                    .on_action(cx.listener(|this, _: &TreeUp, _, cx| {
                        this.update_tree(|tree, doc| tree.step(false, doc), cx)
                    }))
                    .on_action(cx.listener(|this, _: &TreeDown, _, cx| {
                        this.update_tree(|tree, doc| tree.step(true, doc), cx)
                    }))
                    .on_action(cx.listener(|this, _: &TreeLeft, _, cx| {
                        this.update_tree(|tree, doc| tree.left(doc), cx)
                    }))
                    .on_action(cx.listener(|this, _: &TreeRight, _, cx| {
                        this.update_tree(|tree, doc| tree.right(doc), cx)
                    }))
                    .on_action(cx.listener(|this, _: &TreeFocus, _, cx| {
                        this.update_tree(|tree, doc| tree.focus_selected(doc), cx)
                    }))
                    .on_action(cx.listener(|this, _: &TreeRoot, _, cx| {
                        this.update_tree(|tree, _| tree.root(), cx)
                    }))
                    .on_action(cx.listener(|this, _: &Copy, _, cx| this.copy_value(cx)))
                    .children(rows),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .p_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .child(format!("选中节点 · {}", selected.kind.label())),
                            )
                            .child(div().flex_1())
                            .when(selected.kind.is_container(), |view| {
                                view.child(button("focus-node", "聚焦节点").on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.update_tree(|tree, doc| tree.focus_selected(doc), cx)
                                    },
                                )))
                            })
                            .child(
                                button("copy-value", "复制值")
                                    .on_click(cx.listener(|this, _, _, cx| this.copy_value(cx))),
                            ),
                    )
                    .child(self.path_row("JSONPath", self.path.clone(), "copy-path", false, cx))
                    .child(self.path_row("jq", self.jq.clone(), "copy-jq", true, cx))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child("↑↓ 选择 · ←→ 折叠/展开 · Enter 聚焦 · Esc 返回根"),
                    ),
            )
            .into_any_element()
    }
    fn path_row(
        &self,
        label: &'static str,
        field: Entity<TextField>,
        id: &'static str,
        is_jq: bool,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = Theme::dark();
        div()
            .flex()
            .items_center()
            .gap_2()
            .h(px(32.))
            .child(
                div()
                    .w(px(66.))
                    .flex_shrink_0()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .px_2()
                    .py_1()
                    .rounded_sm()
                    .bg(theme.bg)
                    .child(field),
            )
            .child(
                button(id, "复制").on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(node) = this
                        .run
                        .document
                        .as_ref()
                        .and_then(|doc| doc.node(this.tree.selected))
                    {
                        cx.write_to_clipboard(ClipboardItem::new_string(if is_jq {
                            node.jq.clone()
                        } else {
                            node.json_path.clone()
                        }));
                        this.notice = if is_jq {
                            "已复制 jq 表达式"
                        } else {
                            "已复制 JSONPath"
                        };
                        cx.notify();
                    }
                })),
            )
    }
}


impl Render for JsonWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        let stale = self
            .last_input
            .as_ref()
            .is_some_and(|last| last != self.input.read(cx).buffer.text());
        let entity = cx.entity().downgrade();
        div()
            .id("json-workbench")
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if let Some((start, split)) = this.resizing {
                    if event.pressed_button == Some(MouseButton::Left) {
                        let min = (240. / this.pane_width.max(1.)).min(0.45);
                        this.split = (split
                            + f32::from(event.position.x - start) / this.pane_width.max(1.))
                        .clamp(min, (1. - 310. / this.pane_width.max(1.)).max(min));
                        cx.notify();
                    } else {
                        this.resizing = None;
                    }
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.resizing = None),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.resizing = None),
            )
            .child(
                div()
                    .id("workbench-titlebar")
                    .window_control_area(WindowControlArea::Drag)
                    .on_mouse_down(MouseButton::Left, |event, window, _| {
                        if event.click_count == 2 {
                            window.titlebar_double_click();
                        }
                    })
                    .h(px(28.))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().text_lg().child("JSON"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child("离线 · 严格解析 · 保留键顺序"),
                    )
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(theme.accent).child(self.notice)),
            )
            .when(stale, |view| {
                view.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xe5c78b))
                        .child("输入已更改 · 结果和路径来自上次运行，请重新格式化或压缩"),
                )
            })
            .when_some(self.run.error.as_ref(), |view, error| {
                view.child(
                    div()
                        .text_sm()
                        .text_color(theme.error)
                        .child(error.to_string()),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .flex()
                    .on_children_prepainted(move |bounds, _, cx| {
                        if let (Some(first), Some(last)) = (bounds.first(), bounds.last()) {
                            let width = f32::from(last.right() - first.left());
                            if let Some(entity) = entity.upgrade() {
                                entity.update(cx, |this, cx| {
                                    this.pane_width = width;
                                    let min = (240. / width.max(1.)).min(0.45);
                                    let split =
                                        this.split.clamp(min, (1. - 310. / width.max(1.)).max(min));
                                    if this.split != split {
                                        this.split = split;
                                        cx.notify();
                                    }
                                });
                            }
                        }
                    })
                    .child(
                        pane()
                            .w(relative(self.split))
                            .flex_shrink_0()
                            .child(
                                pane_header()
                                    .child(div().text_sm().flex_1().child("原文"))
                                    .child(button("prettify", "格式化").on_click(cx.listener(
                                        |this, _, window, cx| {
                                            this.execute(JsonAction::Prettify, window, cx)
                                        },
                                    )))
                                    .child(button("minify", "压缩").on_click(cx.listener(
                                        |this, _, window, cx| {
                                            this.execute(JsonAction::Minify, window, cx)
                                        },
                                    )))
                                    .child(button("clear", "清空").on_click(
                                        cx.listener(|this, _, window, cx| this.clear(window, cx)),
                                    )),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .min_w_0()
                                    .p_3()
                                    .child(self.input.clone()),
                            ),
                    )
                    .child(
                        div()
                            .id("json-divider")
                            .w(px(10.))
                            .h_full()
                            .flex_shrink_0()
                            .cursor(CursorStyle::ResizeLeftRight)
                            .flex()
                            .items_center()
                            .justify_center()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                    this.resizing = Some((event.position.x, this.split));
                                    cx.stop_propagation();
                                }),
                            )
                            .child(div().w(px(2.)).h(px(36.)).rounded_sm().bg(theme.border)),
                    )
                    .child(
                        pane()
                            .flex_1()
                            .child(
                                pane_header()
                                    .child(div().text_sm().flex_1().child("结果"))
                                    .child(
                                        button("tree-tab", "JSON 树")
                                            .bg(if !self.text_view {
                                                theme.selected
                                            } else {
                                                theme.panel
                                            })
                                            .text_color(if !self.text_view {
                                                theme.accent
                                            } else {
                                                theme.muted
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.text_view = false;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        button("text-tab", "文本")
                                            .bg(if self.text_view {
                                                theme.selected
                                            } else {
                                                theme.panel
                                            })
                                            .text_color(if self.text_view {
                                                theme.accent
                                            } else {
                                                theme.muted
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.text_view = true;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        button("copy-result", "复制结果")
                                            .opacity(if self.run.output.is_some() {
                                                1.
                                            } else {
                                                0.4
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                if let Some(output) = &this.run.output {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(output.clone()),
                                                    );
                                                    this.notice = "已复制结果";
                                                    cx.notify();
                                                }
                                            })),
                                    ),
                            )
                            .child(if self.text_view {
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .min_w_0()
                                    .p_3()
                                    .child(self.output.clone())
                                    .into_any_element()
                            } else {
                                self.tree_view(cx)
                            }),
                    ),
            )
    }
}
