//! One-input → many labeled outputs (it-tools date-converter style).
//!
//! Live-updates for pure transforms. Generators are not hosted here.

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::{self, Theme},
    ui::{
        button, chip, empty_hint, pane, pane_header, pane_title, paste_button, primary_button,
        status_chip, workbench_title, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::{
    binary_tool, chmod_calc, color_tool, number_base,
    number_base::Base,
    percentage, roman, text_stats, timestamp,
};
use gpui::{prelude::FluentBuilder, *};

pub struct BatchWorkbench {
    tool_id: &'static str,
    input: Entity<TextField>,
    rows: Vec<(&'static str, String)>,
    error: Option<String>,
    notice: &'static str,
    num_from: Base,
    result_scroll: ScrollHandle,
}

impl BatchWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextField::new(true, false, "在这里粘贴输入", cx));
        cx.subscribe(&input, |this, _, _: &Edited, cx| {
            this.notice = "";
            this.live_refresh(cx);
        })
        .detach();
        Self {
            tool_id: ids::TIMESTAMP,
            input,
            rows: Vec::new(),
            error: None,
            notice: "",
            num_from: Base::DEC,
            result_scroll: ScrollHandle::new(),
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
        // Edited subscription triggers live_refresh.
        cx.notify();
    }

    pub fn set_tool(&mut self, tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        let switched = self.tool_id != tool_id;
        if switched {
            self.tool_id = tool_id;
            self.rows.clear();
            self.error = None;
            self.notice = "";
            self.input.update(cx, |field, cx| {
                field.set_placeholder(placeholder_for(tool_id), cx);
            });
        }
        if tool_id == ids::TIMESTAMP {
            let current = self.input.read(cx).buffer.text().to_owned();
            if current.trim().is_empty() {
                let seed = timestamp::now_seed();
                self.input.update(cx, |field, cx| field.set_input(&seed, cx));
            }
        }
        self.live_refresh(cx);
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

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.tool_id == ids::TIMESTAMP {
            let seed = timestamp::now_seed();
            self.input.update(cx, |field, cx| {
                field.set_input(&seed, cx);
                field.focus(window);
            });
        } else {
            self.input.update(cx, |field, cx| {
                field.clear(cx);
                field.focus(window);
            });
        }
        self.notice = "";
        self.live_refresh(cx);
        cx.notify();
    }

    fn seed_now(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let seed = timestamp::now_seed();
        self.input.update(cx, |field, cx| {
            field.set_input(&seed, cx);
            field.focus(window);
        });
        self.notice = "已填入当前时间";
        cx.notify();
    }

    fn live_refresh(&mut self, cx: &mut Context<Self>) {
        let input = self.input.read(cx).buffer.text().to_owned();
        if input.trim().is_empty() && self.tool_id != ids::TEXT_STATS {
            self.rows.clear();
            self.error = None;
            cx.notify();
            return;
        }
        match run_batch(self.tool_id, &input, self.num_from) {
            Ok(rows) => {
                self.rows = rows;
                self.error = None;
            }
            Err(message) => {
                self.rows.clear();
                self.error = Some(message);
            }
        }
        cx.notify();
    }

    fn title(&self) -> (&'static str, &'static str) {
        match self.tool_id {
            ids::TIMESTAMP => ("日期时间", "自动识别 · 批量格式 · 默认当前时间"),
            ids::NUMBER_BASE => ("进制转换", "同时显示 2/8/10/16 · 可识别 0x/0b/0o"),
            ids::TEXT_STATS => ("文本统计", "字符 · 词 · 行 · 字节"),
            ids::ROMAN => ("罗马数字", "1–3999 · 自动识别方向"),
            ids::PERCENTAGE => ("百分比", "输入两个数字，例如 25 200"),
            ids::CHMOD => ("Chmod", "755 ↔ rwxr-xr-x"),
            ids::BINARY => ("文本 / 二进制", "二进制 · 十六进制 · 码点"),
            ids::COLOR => ("颜色转换", "#hex · rgb · hsl"),
            _ => ("工具", "离线"),
        }
    }

    fn options_bar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let mut row = div().flex().flex_wrap().items_center().gap_2();
        if self.tool_id == ids::NUMBER_BASE {
            row = row.child(div().text_xs().text_color(theme.muted).child("从"));
            for base in Base::ALL {
                let active = self.num_from == base;
                let id = format!("batch-from-{}", base.label());
                row = row.child(chip(theme, id, base.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.num_from = base;
                        this.live_refresh(cx);
                    },
                )));
            }
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("也可在输入里写 0x / 0b / 0o"),
            );
        } else {
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("输入即更新 · 每行可单独复制"),
            );
        }
        row
    }

    fn input_header(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let mut header = pane_header(theme)
            .child(pane_title(theme, "输入"))
            .child(
                paste_button(theme, "batch-paste").on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        this.paste_into_input(window, cx);
                    }),
                ),
            );
        if self.tool_id == ids::TIMESTAMP {
            header = header.child(
                primary_button(theme, "batch-now", "现在").on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        this.seed_now(window, cx);
                    }),
                ),
            );
        }
        header.child(button(theme, "batch-clear", "清空").on_click(
            cx.listener(|this, _, window, cx| this.clear(window, cx)),
        ))
    }

    fn result_pane(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let header = pane_header(theme).child(pane_title(theme, "结果"));
        let mut pane_view = pane(theme).flex_1().child(header);
        if self.rows.is_empty() && self.error.is_none() {
            pane_view = pane_view.child(empty_hint(theme, "输入后显示全部格式"));
        } else {
            let rows: Vec<AnyElement> = self
                .rows
                .iter()
                .enumerate()
                .map(|(i, (label, value))| {
                    let copy = value.clone();
                    div()
                        .id(SharedString::from(format!("batch-row-{i}")))
                        .flex()
                        .items_center()
                        .gap(px(theme::space::SM))
                        .px(px(theme::space::MD))
                        .py(px(theme::space::XS))
                        .child(
                            div()
                                .w(px(128.))
                                .flex_shrink_0()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.muted)
                                .child(*label),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .px(px(theme::space::SM))
                                .py(px(theme::space::XS))
                                .rounded(px(theme::radius::SM))
                                .bg(theme.elevated)
                                .border_1()
                                .border_color(theme.border_subtle)
                                .font(theme::monospace())
                                .text_sm()
                                .text_color(theme.text)
                                .child(value.clone()),
                        )
                        .child(copy_chip(format!("batch-copy-{i}"), copy, cx))
                        .into_any_element()
                })
                .collect();
            pane_view = pane_view.child(
                div()
                    .id("batch-results")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .py(px(theme::space::SM))
                    .children(rows),
            );
        }
        pane_view
    }
}

fn copy_chip(id: String, value: String, cx: &mut Context<BatchWorkbench>) -> Stateful<Div> {
    let theme = Theme::get(cx);
    div()
        .id(SharedString::from(id))
        .cursor_pointer()
        .h(px(28.))
        .px(px(theme::space::SM))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(theme::radius::MD))
        .text_sm()
        .flex_shrink_0()
        .bg(theme.elevated)
        .text_color(theme.muted)
        .border_1()
        .border_color(theme.border)
        .hover(move |s| s.bg(theme.selected).text_color(theme.accent))
        .child("复制")
        .on_click(cx.listener(move |this, _, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(value.clone()));
            this.notice = "已复制";
            cx.notify();
        }))
}

fn placeholder_for(tool_id: &str) -> &'static str {
    match tool_id {
        ids::TIMESTAMP => "Unix 秒/毫秒、ISO 8601 或 YYYY-MM-DD HH:MM:SS",
        ids::NUMBER_BASE => "例如 255、0xff、0b1111",
        ids::TEXT_STATS => "粘贴要统计的文本",
        ids::ROMAN => "例如 1994 或 MCMXCIV",
        ids::PERCENTAGE => "两个数字，例如 25 200",
        ids::CHMOD => "例如 755 或 rwxr-xr-x",
        ids::BINARY => "文本，或二进制 / 十六进制",
        ids::COLOR => "#336699 或 rgb(51, 102, 153)",
        _ => "在这里粘贴输入",
    }
}

fn run_batch(
    tool_id: &str,
    input: &str,
    num_from: Base,
) -> Result<Vec<(&'static str, String)>, String> {
    match tool_id {
        ids::TIMESTAMP => timestamp::convert_all(input)
            .map(|rows| {
                rows.into_iter()
                    .map(|r| (r.label, r.value))
                    .collect()
            })
            .map_err(|e| e.to_string()),
        ids::NUMBER_BASE => {
            let (detected, normalized) = number_base::normalize_input(input);
            // Prefer explicit chip when input has no prefix.
            let from = if input.trim().to_ascii_lowercase().starts_with("0x")
                || input.trim().to_ascii_lowercase().starts_with("0b")
                || input.trim().to_ascii_lowercase().starts_with("0o")
                || input.trim().to_ascii_lowercase().starts_with("-0x")
                || input.trim().to_ascii_lowercase().starts_with("-0b")
                || input.trim().to_ascii_lowercase().starts_with("-0o")
            {
                detected
            } else {
                num_from
            };
            number_base::convert_all(&normalized, from).map_err(|e| e.to_string())
        }
        ids::TEXT_STATS => Ok(text_stats::rows(input)),
        ids::ROMAN => roman::convert_all(input),
        ids::PERCENTAGE => percentage::convert_all(input),
        ids::CHMOD => chmod_calc::convert_all(input),
        ids::BINARY => binary_tool::convert_all(input),
        ids::COLOR => color_tool::convert_all(input),
        _ => Err("未知批量工具".into()),
    }
}

impl Render for BatchWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = self.title();
        div()
            .id("batch-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, _, cx| this.live_refresh(cx)))
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(theme::space::MD))
            .p(px(theme::space::XL))
            .child(
                workbench_title(theme, title, subtitle)
                    .id("batch-titlebar")
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
                    .gap(px(theme::space::MD))
                    .child(
                        pane(theme)
                            .w(relative(0.38))
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
                    .child(self.result_pane(cx)),
            )
    }
}
