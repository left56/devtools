//! Generate-only workbench (UUID / ULID / Token / Lorem).
//!
//! Converters use `BatchWorkbench` (one input → all related outputs).

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::Theme,
    ui::{
        button, chip, pane, pane_header, pane_title, paste_button, primary_button, status_chip,
        workbench_title, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::{
    lorem, token_gen,
    token_gen::Alphabet as TokenAlphabet,
    ulid_tool,
    uuid_tool::{self, Version as UuidVersion},
};
use gpui::{prelude::FluentBuilder, *};

pub struct TransformWorkbench {
    tool_id: &'static str,
    input: Entity<TextField>,
    output: Entity<TextField>,
    uuid_version: UuidVersion,
    lorem_paragraphs: usize,
    token_length: usize,
    token_alphabet: TokenAlphabet,
    output_text: Option<String>,
    error: Option<String>,
    notice: &'static str,
    split: f32,
    pane_width: f32,
    resizing: Option<(Pixels, f32)>,
}

impl TransformWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextField::new(true, false, "在这里粘贴输入", cx));
        cx.subscribe(&input, |this, _, _: &Edited, cx| {
            this.notice = "";
            cx.notify();
        })
        .detach();
        Self {
            tool_id: ids::UUID,
            input,
            output: cx.new(|cx| TextField::new(true, true, "点击「生成」显示结果", cx)),
            uuid_version: UuidVersion::V4,
            lorem_paragraphs: 1,
            token_length: 32,
            token_alphabet: TokenAlphabet::Hex,
            output_text: None,
            error: None,
            notice: "",
            split: 0.48,
            pane_width: 1000.,
            resizing: None,
        }
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        self.input.read(cx).focus(window);
    }

    pub fn set_input_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |field, cx| {
            field.set_input(text, cx);
            field.focus(window);
        });
        self.notice = "";
        cx.notify();
    }

    pub fn set_tool(&mut self, tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        if self.tool_id != tool_id {
            self.tool_id = tool_id;
            self.output_text = None;
            self.error = None;
            self.notice = "";
            self.output
                .update(cx, |field, cx| field.set_output("", cx));
            self.input.update(cx, |field, cx| {
                field.set_placeholder(placeholder_for(tool_id), cx);
            });
        }
        self.focus_input(window, cx);
        cx.notify();
    }

    fn paste_into_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        if text.is_empty() {
            self.notice = "剪贴板为空";
            cx.notify();
            return;
        }
        self.input.update(cx, |field, cx| {
            field.set_input(&text, cx);
            field.focus(window);
        });
        self.notice = "已粘贴";
        cx.notify();
    }

    fn execute(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input.read(cx).buffer.text().to_owned();
        let result = run_generate(self.tool_id, &input, self);
        match result {
            Ok(output) => {
                self.output_text = Some(output.clone());
                self.error = None;
                self.output
                    .update(cx, |field, cx| field.set_output(&output, cx));
                self.notice = "已更新结果";
            }
            Err(message) => {
                self.output_text = None;
                self.error = Some(message);
                self.output
                    .update(cx, |field, cx| field.set_output("", cx));
                self.input.update(cx, |field, _cx| {
                    field.focus(window);
                });
            }
        }
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |field, cx| {
            field.clear(cx);
            field.focus(window);
        });
        self.output.update(cx, |field, cx| field.set_output("", cx));
        self.output_text = None;
        self.error = None;
        self.notice = "";
        cx.notify();
    }

    fn title(&self) -> (&'static str, &'static str) {
        match self.tool_id {
            ids::UUID => ("UUID", "离线 · v4 随机 · 可选 v7"),
            ids::ULID => ("ULID", "离线 · 时间有序 · 点击生成"),
            ids::TOKEN => ("Token", "离线 · 随机令牌 · 点击生成"),
            ids::LOREM => ("Lorem ipsum", "离线 · 占位段落 · 点击生成"),
            _ => ("工具", "离线"),
        }
    }

    fn options_bar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let mut row = div().flex().flex_wrap().items_center().gap_2();
        match self.tool_id {
            ids::UUID => {
                row = row.child(div().text_xs().text_color(theme.muted).child("版本"));
                for version in UuidVersion::ALL {
                    let active = self.uuid_version == version;
                    let id = match version {
                        UuidVersion::V4 => "uuid-v4",
                        UuidVersion::V7 => "uuid-v7",
                    };
                    row = row.child(chip(theme, id, version.label(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.uuid_version = version;
                            cx.notify();
                        },
                    )));
                }
            }
            ids::LOREM => {
                row = row.child(div().text_xs().text_color(theme.muted).child("段落数"));
                for n in [1usize, 2, 3, 5] {
                    let active = self.lorem_paragraphs == n;
                    let id = format!("lorem-{n}");
                    row = row.child(chip(theme, id, n.to_string(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.lorem_paragraphs = n;
                            cx.notify();
                        },
                    )));
                }
            }
            ids::TOKEN => {
                row = row.child(div().text_xs().text_color(theme.muted).child("长度"));
                for n in [16usize, 24, 32, 64] {
                    let active = self.token_length == n;
                    let id = format!("token-len-{n}");
                    row = row.child(chip(theme, id, n.to_string(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.token_length = n;
                            cx.notify();
                        },
                    )));
                }
                row = row.child(div().text_xs().text_color(theme.muted).child("字母表"));
                for alphabet in TokenAlphabet::ALL {
                    let active = self.token_alphabet == alphabet;
                    let id = format!("token-alpha-{}", alphabet.label());
                    row = row.child(chip(theme, id, alphabet.label(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.token_alphabet = alphabet;
                            cx.notify();
                        },
                    )));
                }
            }
            ids::ULID => {}
            _ => {}
        }
        row.child(
            div()
                .text_xs()
                .text_color(theme.muted)
                .child("点击生成"),
        )
    }

    fn input_header(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        pane_header(theme)
            .child(pane_title(theme, "原文"))
            .child(
                paste_button(theme, "transform-paste").on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        this.paste_into_input(window, cx);
                    }),
                ),
            )
            .child(
                primary_button(theme, "generate", "生成")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, window, cx| {
                            cx.stop_propagation();
                            this.execute(window, cx);
                        }),
                    )
                    .on_click(cx.listener(|this, _, window, cx| this.execute(window, cx))),
            )
            .child(button(theme, "clear", "清空").on_click(
                cx.listener(|this, _, window, cx| this.clear(window, cx)),
            ))
    }
}

fn placeholder_for(tool_id: &str) -> &'static str {
    match tool_id {
        ids::UUID | ids::ULID => "点击「生成」；输入不参与生成",
        ids::TOKEN => "可选在输入里写长度数字；主要用上方选项",
        ids::LOREM => "可选：在上方选段落数，再点生成",
        _ => "在这里粘贴输入",
    }
}

fn run_generate(
    tool_id: &str,
    input: &str,
    state: &TransformWorkbench,
) -> Result<String, String> {
    match tool_id {
        ids::UUID => Ok(uuid_tool::generate(state.uuid_version)),
        ids::ULID => Ok(ulid_tool::generate()),
        ids::TOKEN => {
            let len = token_gen::parse_length(input, state.token_length);
            Ok(token_gen::generate(len, state.token_alphabet))
        }
        ids::LOREM => Ok(lorem::generate(state.lorem_paragraphs)),
        _ => Err("此工具没有该操作".into()),
    }
}

impl Render for TransformWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = self.title();
        let entity = cx.entity().downgrade();
        div()
            .id("transform-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, window, cx| {
                this.execute(window, cx)
            }))
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(crate::theme::space::MD))
            .p(px(crate::theme::space::XL))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if let Some((start, split)) = this.resizing {
                    if event.pressed_button == Some(MouseButton::Left) {
                        let min = (240. / this.pane_width.max(1.)).min(0.45);
                        this.split = (split
                            + f32::from(event.position.x - start) / this.pane_width.max(1.))
                        .clamp(min, (1. - 240. / this.pane_width.max(1.)).max(min));
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
                workbench_title(
                    theme,
                    title,
                    subtitle.trim_start_matches("离线 · ").to_owned(),
                )
                .id("transform-titlebar")
                .window_control_area(WindowControlArea::Drag)
                .on_mouse_down(MouseButton::Left, |event, window, _| {
                    if event.click_count == 2 {
                        window.titlebar_double_click();
                    }
                })
                .child(status_chip(theme, "离线", StatusKind::Neutral))
                .child(div().flex_1())
                .when(!self.notice.is_empty(), |view| {
                    view.child(status_chip(theme, self.notice, StatusKind::Accent))
                }),
            )
            .child(self.options_bar(cx))
            .when_some(self.error.as_ref(), |view, error| {
                view.child(crate::ui::banner(theme, StatusKind::Error, error.clone()))
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
                                        this.split.clamp(min, (1. - 240. / width.max(1.)).max(min));
                                    if this.split != split {
                                        this.split = split;
                                        cx.notify();
                                    }
                                });
                            }
                        }
                    })
                    .child(
                        pane(theme)
                            .w(relative(self.split))
                            .flex_shrink_0()
                            .child(self.input_header(cx))
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
                            .id("transform-divider")
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
                        pane(theme)
                            .flex_1()
                            .child(
                                pane_header(theme)
                                    .child(pane_title(theme, "结果"))
                                    .child(
                                        button(theme, "copy-result", "复制结果")
                                            .opacity(if self.output_text.is_some() {
                                                1.
                                            } else {
                                                0.4
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                if let Some(output) = &this.output_text {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(output.clone()),
                                                    );
                                                    this.notice = "已复制结果";
                                                    cx.notify();
                                                }
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .min_w_0()
                                    .p_3()
                                    .child(self.output.clone()),
                            ),
                    ),
            )
    }
}
