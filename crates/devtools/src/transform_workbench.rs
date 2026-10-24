//! Generate-only workbench (UUID / ULID / Token / Lorem).
//!
//! No source/「原文」 pane — options chips + 「生成」 + full-width result.
//! Converters use `BatchWorkbench`.

use crate::{
    actions::PrimaryRun,
    text_field::TextField,
    theme::Theme,
    ui::{
        button, chip, pane, pane_header, pane_title, primary_button, status_chip, workbench_title,
        StatusKind,
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
    output: Entity<TextField>,
    uuid_version: UuidVersion,
    lorem_paragraphs: usize,
    token_length: usize,
    token_alphabet: TokenAlphabet,
    output_text: Option<String>,
    error: Option<String>,
    notice: &'static str,
}

impl TransformWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            tool_id: ids::UUID,
            output: cx.new(|cx| TextField::new(true, true, "点击「生成」显示结果", cx)),
            uuid_version: UuidVersion::V4,
            lorem_paragraphs: 1,
            token_length: 32,
            token_alphabet: TokenAlphabet::Hex,
            output_text: None,
            error: None,
            notice: "",
        }
    }

    /// Generators have no source field; kept for workspace focus routing.
    pub fn focus_input(&self, _window: &mut Window, _cx: &App) {}

    /// Generators ignore pasted source text (options chips drive length/etc.).
    pub fn set_input_text(&mut self, _text: &str, _window: &mut Window, cx: &mut Context<Self>) {
        self.notice = "";
        cx.notify();
    }

    pub fn set_tool(&mut self, tool_id: &'static str, _window: &mut Window, cx: &mut Context<Self>) {
        if self.tool_id != tool_id {
            self.tool_id = tool_id;
            self.output_text = None;
            self.error = None;
            self.notice = "";
            self.output
                .update(cx, |field, cx| field.set_output("", cx));
        }
        cx.notify();
    }

    fn execute(&mut self, cx: &mut Context<Self>) {
        match run_generate(self.tool_id, self) {
            Ok(output) => {
                self.output_text = Some(output.clone());
                self.error = None;
                self.output
                    .update(cx, |field, cx| field.set_output(&output, cx));
                self.notice = "已生成";
            }
            Err(message) => {
                self.output_text = None;
                self.error = Some(message);
                self.output
                    .update(cx, |field, cx| field.set_output("", cx));
            }
        }
        cx.notify();
    }

    fn clear_result(&mut self, cx: &mut Context<Self>) {
        self.output.update(cx, |field, cx| field.set_output("", cx));
        self.output_text = None;
        self.error = None;
        self.notice = "";
        cx.notify();
    }

    fn title(&self) -> (&'static str, &'static str) {
        match self.tool_id {
            ids::UUID => ("UUID", "v4 随机 · 可选 v7"),
            ids::ULID => ("ULID", "时间有序 · 点击生成"),
            ids::TOKEN => ("Token", "随机令牌 · 点击生成"),
            ids::LOREM => ("Lorem ipsum", "占位段落 · 点击生成"),
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
        row = row
            .child(
                primary_button(theme, "generate", "生成")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                            this.execute(cx);
                        }),
                    )
                    .on_click(cx.listener(|this, _, _, cx| this.execute(cx))),
            )
            .child(
                button(theme, "clear-result", "清空结果").on_click(
                    cx.listener(|this, _, _, cx| this.clear_result(cx)),
                ),
            );
        row
    }
}

fn run_generate(tool_id: &str, state: &TransformWorkbench) -> Result<String, String> {
    match tool_id {
        ids::UUID => Ok(uuid_tool::generate(state.uuid_version)),
        ids::ULID => Ok(ulid_tool::generate()),
        ids::TOKEN => Ok(token_gen::generate(
            state.token_length,
            state.token_alphabet,
        )),
        ids::LOREM => Ok(lorem::generate(state.lorem_paragraphs)),
        _ => Err("此工具没有该操作".into()),
    }
}

impl Render for TransformWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = self.title();
        div()
            .id("transform-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, _, cx| this.execute(cx)))
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(crate::theme::space::MD))
            .p(px(crate::theme::space::XL))
            .child(
                workbench_title(theme, title, subtitle)
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
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                output.clone(),
                                            ));
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
            )
    }
}
