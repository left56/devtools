//! Generic left/right editor for M2 pure-function tools.
//!
//! Tool-specific options live here; algorithms stay in `devtools-tools`.
//! No platform APIs — the same widget should work once GPUI builds on
//! Linux / Windows.

use crate::{
    text_field::{Edited, TextField},
    theme::Theme,
    ui::{button, chip, pane, pane_header},
};
use devtools_core::ids;
use devtools_tools::{
    base64::{self, Alphabet},
    case::{self, CaseStyle},
    hash::{self, Algorithm},
    html_escape,
    number_base::{self, Base},
    timestamp::{self, Unit, Zone},
    url_encode,
    uuid_tool::{self, Version as UuidVersion},
};
use gpui::{prelude::FluentBuilder, *};

#[derive(Clone, Copy)]
enum TransformAction {
    Encode,
    Decode,
    Hash,
    Generate,
    FromUnix,
    ToUnix,
    Convert,
}

pub struct TransformWorkbench {
    tool_id: &'static str,
    input: Entity<TextField>,
    output: Entity<TextField>,
    base64_url_safe: bool,
    hash_algo: Algorithm,
    uuid_version: UuidVersion,
    ts_unit: Unit,
    ts_zone: Zone,
    num_from: Base,
    num_to: Base,
    case_style: CaseStyle,
    output_text: Option<String>,
    error: Option<String>,
    error_offset: Option<usize>,
    last_input: Option<String>,
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
            tool_id: ids::BASE64,
            input,
            output: cx.new(|cx| TextField::new(true, true, "运行后在这里显示结果", cx)),
            base64_url_safe: false,
            hash_algo: Algorithm::Sha256,
            uuid_version: UuidVersion::V4,
            ts_unit: Unit::Seconds,
            ts_zone: Zone::Local,
            num_from: Base::DEC,
            num_to: Base::HEX,
            case_style: CaseStyle::Snake,
            output_text: None,
            error: None,
            error_offset: None,
            last_input: None,
            notice: "",
            split: 0.48,
            pane_width: 1000.,
            resizing: None,
        }
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        self.input.read(cx).focus(window);
    }

    pub fn set_tool(&mut self, tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        if self.tool_id != tool_id {
            self.tool_id = tool_id;
            // Keep the typed text when switching among transform tools; clear
            // only the last run so the stale banner does not lie about another tool.
            self.output_text = None;
            self.error = None;
            self.error_offset = None;
            self.last_input = None;
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

    fn execute(&mut self, action: TransformAction, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input.read(cx).buffer.text().to_owned();
        let result = run_transform(self.tool_id, &input, action, self);
        self.last_input = Some(input);
        match result {
            Ok(output) => {
                self.output_text = Some(output.clone());
                self.error = None;
                self.error_offset = None;
                self.output
                    .update(cx, |field, cx| field.set_output(&output, cx));
            }
            Err((message, offset)) => {
                self.output_text = None;
                self.error = Some(message);
                self.error_offset = offset;
                self.output
                    .update(cx, |field, cx| field.set_output("", cx));
                if let Some(byte) = offset {
                    self.input.update(cx, |field, cx| {
                        field.reveal(byte, cx);
                        field.focus(window);
                    });
                }
            }
        }
        self.notice = "";
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
        self.error_offset = None;
        self.last_input = None;
        self.notice = "";
        cx.notify();
    }

    fn title(&self) -> (&'static str, &'static str) {
        match self.tool_id {
            ids::BASE64 => ("Base64", "离线 · 默认标准字母表 · URL-safe 可选"),
            ids::URL_ENCODE => ("URL Encode", "离线 · 组件编解码 · 不是整段 URL 解析"),
            ids::HTML_ESCAPE => ("HTML Escape", "离线 · 实体转义与反转义"),
            ids::HASH => ("Hash", "离线 · MD5/SHA-1 不用于密码或完整性安全"),
            ids::UUID => ("UUID", "离线 · v4 随机 · 可选 v7"),
            ids::TIMESTAMP => ("Timestamp", "离线 · 秒/毫秒 · 默认本机时区"),
            ids::NUMBER_BASE => ("Number Base", "离线 · 2/8/10/16 · 非法数字指出位置"),
            ids::CASE => ("Case", "离线 · snake / camel / kebab / CONSTANT"),
            _ => ("工具", "离线"),
        }
    }

    fn options_bar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::dark();
        let mut row = div().flex().flex_wrap().items_center().gap_2();
        match self.tool_id {
            ids::BASE64 => {
                row = row
                    .child(div().text_xs().text_color(theme.muted).child("字母表"))
                    .child(chip("b64-std", "标准", !self.base64_url_safe).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.base64_url_safe = false;
                            cx.notify();
                        }),
                    ))
                    .child(chip("b64-url", "URL-safe", self.base64_url_safe).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.base64_url_safe = true;
                            cx.notify();
                        }),
                    ));
            }
            ids::HASH => {
                row = row.child(div().text_xs().text_color(theme.muted).child("算法"));
                for algo in Algorithm::ALL {
                    let active = self.hash_algo == algo;
                    let id = match algo {
                        Algorithm::Md5 => "hash-md5",
                        Algorithm::Sha1 => "hash-sha1",
                        Algorithm::Sha256 => "hash-sha256",
                        Algorithm::Sha512 => "hash-sha512",
                    };
                    let label = if let Some(note) = algo.safety_note() {
                        format!("{} · {}", algo.label(), note)
                    } else {
                        algo.label().to_string()
                    };
                    row = row.child(chip(id, label, active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.hash_algo = algo;
                            cx.notify();
                        },
                    )));
                }
            }
            ids::UUID => {
                row = row.child(div().text_xs().text_color(theme.muted).child("版本"));
                for version in UuidVersion::ALL {
                    let active = self.uuid_version == version;
                    let id = match version {
                        UuidVersion::V4 => "uuid-v4",
                        UuidVersion::V7 => "uuid-v7",
                    };
                    row = row.child(chip(id, version.label(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.uuid_version = version;
                            cx.notify();
                        },
                    )));
                }
            }
            ids::TIMESTAMP => {
                row = row
                    .child(div().text_xs().text_color(theme.muted).child("单位"))
                    .child(chip("ts-sec", "秒", matches!(self.ts_unit, Unit::Seconds)).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.ts_unit = Unit::Seconds;
                            cx.notify();
                        }),
                    ))
                    .child(chip("ts-ms", "毫秒", matches!(self.ts_unit, Unit::Milliseconds)).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.ts_unit = Unit::Milliseconds;
                            cx.notify();
                        }),
                    ))
                    .child(div().text_xs().text_color(theme.muted).child("时区"))
                    .child(chip("ts-local", "本机", matches!(self.ts_zone, Zone::Local)).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.ts_zone = Zone::Local;
                            cx.notify();
                        }),
                    ))
                    .child(chip("ts-utc", "UTC", matches!(self.ts_zone, Zone::Utc)).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.ts_zone = Zone::Utc;
                            cx.notify();
                        }),
                    ));
            }
            ids::NUMBER_BASE => {
                row = row.child(div().text_xs().text_color(theme.muted).child("从"));
                for base in Base::ALL {
                    let active = self.num_from == base;
                    let id = format!("from-{}", base.label());
                    row = row.child(chip(id, base.label(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.num_from = base;
                            cx.notify();
                        },
                    )));
                }
                row = row.child(div().text_xs().text_color(theme.muted).child("到"));
                for base in Base::ALL {
                    let active = self.num_to == base;
                    let id = format!("to-{}", base.label());
                    row = row.child(chip(id, base.label(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.num_to = base;
                            cx.notify();
                        },
                    )));
                }
            }
            ids::CASE => {
                row = row.child(div().text_xs().text_color(theme.muted).child("风格"));
                for style in CaseStyle::ALL {
                    let active = self.case_style == style;
                    let id = format!("case-{}", style.label());
                    row = row.child(chip(id, style.label(), active).on_click(cx.listener(
                        move |this, _, _, cx| {
                            this.case_style = style;
                            cx.notify();
                        },
                    )));
                }
            }
            _ => {}
        }
        row
    }

    fn action_buttons(&self, cx: &mut Context<Self>) -> Div {
        let mut row = div().flex().items_center().gap_2();
        match self.tool_id {
            ids::BASE64 | ids::URL_ENCODE | ids::HTML_ESCAPE => {
                row = row
                    .child(button("encode", "编码").on_click(cx.listener(|this, _, window, cx| {
                        this.execute(TransformAction::Encode, window, cx)
                    })))
                    .child(button("decode", "解码").on_click(cx.listener(|this, _, window, cx| {
                        this.execute(TransformAction::Decode, window, cx)
                    })));
            }
            ids::HASH => {
                row = row.child(button("hash", "计算").on_click(cx.listener(
                    |this, _, window, cx| this.execute(TransformAction::Hash, window, cx),
                )));
            }
            ids::UUID => {
                row = row.child(button("generate", "生成").on_click(cx.listener(
                    |this, _, window, cx| this.execute(TransformAction::Generate, window, cx),
                )));
            }
            ids::TIMESTAMP => {
                row = row
                    .child(button("from-unix", "时间戳 → 时间").on_click(cx.listener(
                        |this, _, window, cx| this.execute(TransformAction::FromUnix, window, cx),
                    )))
                    .child(button("to-unix", "时间 → 时间戳").on_click(cx.listener(
                        |this, _, window, cx| this.execute(TransformAction::ToUnix, window, cx),
                    )));
            }
            ids::NUMBER_BASE | ids::CASE => {
                row = row.child(button("convert", "转换").on_click(cx.listener(
                    |this, _, window, cx| this.execute(TransformAction::Convert, window, cx),
                )));
            }
            _ => {}
        }
        row.child(button("clear", "清空").on_click(
            cx.listener(|this, _, window, cx| this.clear(window, cx)),
        ))
    }
}

fn placeholder_for(tool_id: &str) -> &'static str {
    match tool_id {
        ids::BASE64 => "粘贴文本以编码，或粘贴 Base64 以解码",
        ids::URL_ENCODE => "粘贴要编码的组件，或粘贴百分号编码以解码",
        ids::HTML_ESCAPE => "粘贴 HTML 片段以转义或反转义",
        ids::HASH => "粘贴要计算摘要的文本",
        ids::UUID => "点击生成；也可粘贴 UUID 查看版本（生成不依赖输入）",
        ids::TIMESTAMP => "粘贴 Unix 时间戳或 YYYY-MM-DD HH:MM:SS",
        ids::NUMBER_BASE => "粘贴整数，例如 255 或 ff",
        ids::CASE => "粘贴标识符，例如 hello_world",
        _ => "在这里粘贴输入",
    }
}

fn run_transform(
    tool_id: &str,
    input: &str,
    action: TransformAction,
    state: &TransformWorkbench,
) -> Result<String, (String, Option<usize>)> {
    match (tool_id, action) {
        (ids::BASE64, TransformAction::Encode) => {
            let alphabet = if state.base64_url_safe {
                Alphabet::UrlSafe
            } else {
                Alphabet::Standard
            };
            Ok(base64::encode(input, alphabet))
        }
        (ids::BASE64, TransformAction::Decode) => {
            let alphabet = if state.base64_url_safe {
                Alphabet::UrlSafe
            } else {
                Alphabet::Standard
            };
            base64::decode(input, alphabet).map_err(|e| (e.to_string(), None))
        }
        (ids::URL_ENCODE, TransformAction::Encode) => Ok(url_encode::encode(input)),
        (ids::URL_ENCODE, TransformAction::Decode) => {
            url_encode::decode(input).map_err(|e| (e.to_string(), None))
        }
        (ids::HTML_ESCAPE, TransformAction::Encode) => Ok(html_escape::escape(input)),
        (ids::HTML_ESCAPE, TransformAction::Decode) => html_escape::unescape(input)
            .map_err(|e| (e.to_string(), e.offset)),
        (ids::HASH, TransformAction::Hash) => Ok(hash::digest(input, state.hash_algo)),
        (ids::UUID, TransformAction::Generate) => Ok(uuid_tool::generate(state.uuid_version)),
        (ids::TIMESTAMP, TransformAction::FromUnix) => {
            timestamp::from_unix(input, state.ts_unit, state.ts_zone)
                .map_err(|e| (e.to_string(), None))
        }
        (ids::TIMESTAMP, TransformAction::ToUnix) => {
            timestamp::to_unix(input, state.ts_unit, state.ts_zone)
                .map_err(|e| (e.to_string(), None))
        }
        (ids::NUMBER_BASE, TransformAction::Convert) => {
            number_base::convert(input, state.num_from, state.num_to).map_err(|e| {
                let offset = e.char_index.and_then(|index| {
                    // Map digit index back onto the trimmed body roughly.
                    let trimmed = input.trim();
                    let body = trimmed.trim_start_matches(['+', '-']).trim();
                    body.char_indices().nth(index).map(|(byte, _)| {
                        trimmed.find(body).unwrap_or(0) + byte
                    })
                });
                (e.to_string(), offset)
            })
        }
        (ids::CASE, TransformAction::Convert) => Ok(case::convert(input, state.case_style)),
        _ => Err(("此工具没有该操作".into(), None)),
    }
}

impl Render for TransformWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::dark();
        let stale = self
            .last_input
            .as_ref()
            .is_some_and(|last| last != self.input.read(cx).buffer.text());
        let (title, subtitle) = self.title();
        let entity = cx.entity().downgrade();
        div()
            .id("transform-workbench")
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
                div()
                    .id("transform-titlebar")
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
                    .child(div().text_lg().child(title))
                    .child(div().text_xs().text_color(theme.muted).child(subtitle))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(theme.accent).child(self.notice)),
            )
            .child(self.options_bar(cx))
            .when(stale, |view| {
                view.child(
                    div()
                        .text_xs()
                        .text_color(rgb(0xe5c78b))
                        .child("输入已更改 · 结果来自上次运行，请重新执行"),
                )
            })
            .when_some(self.error.as_ref(), |view, error| {
                view.child(div().text_sm().text_color(theme.error).child(error.clone()))
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
                        pane()
                            .w(relative(self.split))
                            .flex_shrink_0()
                            .child(
                                pane_header()
                                    .child(div().text_sm().flex_1().child("原文"))
                                    .child(self.action_buttons(cx)),
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
                        pane()
                            .flex_1()
                            .child(
                                pane_header()
                                    .child(div().text_sm().flex_1().child("结果"))
                                    .child(
                                        button("copy-result", "复制结果")
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
