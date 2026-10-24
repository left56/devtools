//! M3 inspector UIs: JWT, Regex, Cron, URL Parse, Diff.
//! Structured results live here; algorithms stay in `devtools-tools`.

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::Theme,
    ui::{
        banner, button, card, chip, kv_row, pane, pane_header, paste_button,
        status_chip, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::{
    cron_tool::{self, Zone as CronZone},
    diff_tool::{self, DiffLine, LineKind},
    jwt::{self, JwtDecode},
    regex_tool::{self, RegexResult},
    url_parse::{self, UrlParts},
};
use gpui::{prelude::FluentBuilder, Timer, *};
use std::time::Duration;

enum InspectorState {
    Idle,
    Jwt(JwtDecode),
    Regex(RegexResult),
    Cron {
        lines: Vec<String>,
        zone: &'static str,
        expr: String,
    },
    Url(UrlParts),
    Diff {
        lines: Vec<DiffLine>,
        inserts: usize,
        deletes: usize,
    },
}

pub struct InspectorWorkbench {
    tool_id: &'static str,
    // Shared primary input (JWT token, cron expr, URL, regex haystack left, etc.)
    primary: Entity<TextField>,
    // Secondary: regex pattern / diff right / regex replace
    secondary: Entity<TextField>,
    tertiary: Entity<TextField>,
    cron_zone: CronZone,
    state: InspectorState,
    error: Option<String>,
    notice: &'static str,
    result_scroll: ScrollHandle,
    /// Debounce generation for regex live-run.
    live_gen: u64,
}

impl InspectorWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let primary = cx.new(|cx| TextField::new(true, false, "输入", cx));
        let secondary = cx.new(|cx| TextField::new(true, false, "", cx));
        let tertiary = cx.new(|cx| TextField::new(false, false, "替换（可选）", cx));
        for field in [&primary, &secondary, &tertiary] {
            cx.subscribe(field, |this, _, _: &Edited, cx| {
                this.notice = "";
                this.schedule_live(cx);
            })
            .detach();
        }
        Self {
            tool_id: ids::JWT,
            primary,
            secondary,
            tertiary,
            cron_zone: CronZone::Local,
            state: InspectorState::Idle,
            error: None,
            notice: "",
            result_scroll: ScrollHandle::new(),
            live_gen: 0,
        }
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        // Regex: pattern lives in secondary and is the first thing to fill.
        if self.tool_id == ids::REGEX {
            self.secondary.read(cx).focus(window);
        } else {
            self.primary.read(cx).focus(window);
        }
    }
    pub fn set_input_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.primary.update(cx, |field, cx| {
            field.set_input(text, cx);
            field.focus(window);
        });
        self.notice = "";
        cx.notify();
    }

    pub fn set_tool(&mut self, tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        if self.tool_id != tool_id {
            self.tool_id = tool_id;
            self.state = InspectorState::Idle;
            self.error = None;
            self.notice = "";
            self.primary.update(cx, |f, cx| {
                f.set_placeholder(primary_placeholder(tool_id), cx);
            });
            self.secondary.update(cx, |f, cx| {
                f.set_placeholder(secondary_placeholder(tool_id), cx);
            });
            self.schedule_live(cx);
        }
        self.focus_input(window, cx);
        cx.notify();
    }

    fn paste_into(&mut self, target: PasteTarget, window: &mut Window, cx: &mut Context<Self>) {
        let text = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        if text.is_empty() {
            self.notice = "剪贴板为空";
            cx.notify();
            return;
        }
        match target {
            PasteTarget::Primary => self.primary.update(cx, |f, cx| {
                f.set_input(&text, cx);
                f.focus(window);
            }),
            PasteTarget::Secondary => self.secondary.update(cx, |f, cx| {
                f.set_input(&text, cx);
                f.focus(window);
            }),
            PasteTarget::Tertiary => self.tertiary.update(cx, |f, cx| {
                f.set_input(&text, cx);
                f.focus(window);
            }),
        }
        self.notice = "已粘贴";
        cx.notify();
    }

    fn schedule_live(&mut self, cx: &mut Context<Self>) {
        self.live_gen = self.live_gen.wrapping_add(1);
        let generation = self.live_gen;
        let delay = if self.tool_id == ids::REGEX {
            Duration::from_millis(180)
        } else {
            Duration::from_millis(0)
        };
        cx.spawn(async move |this, cx| {
            if !delay.is_zero() {
                Timer::after(delay).await;
            }
            let _ = this.update(cx, |this, cx| {
                if this.live_gen == generation {
                    this.run_live(cx);
                }
            });
        })
        .detach();
    }

    fn run_live(&mut self, cx: &mut Context<Self>) {
        self.run_inner(cx);
    }

    fn run(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.run_inner(cx);
    }

    fn run_inner(&mut self, cx: &mut Context<Self>) {
        let primary = self.primary.read(cx).buffer.text().to_owned();
        let secondary = self.secondary.read(cx).buffer.text().to_owned();
        let tertiary = self.tertiary.read(cx).buffer.text().to_owned();
        self.error = None;
        self.state = InspectorState::Idle;

        let empty = match self.tool_id {
            ids::REGEX => secondary.trim().is_empty() && primary.trim().is_empty(),
            ids::DIFF => primary.is_empty() && secondary.is_empty(),
            _ => primary.trim().is_empty(),
        };
        if empty {
            self.notice = "";
            cx.notify();
            return;
        }

        let result = match self.tool_id {
            ids::JWT => jwt::decode(&primary)
                .map(InspectorState::Jwt)
                .map_err(|e| e.to_string()),
            ids::REGEX => {
                let replacement = {
                    let t = tertiary.trim();
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                };
                // secondary = pattern, primary = haystack
                regex_tool::run(&secondary, &primary, replacement)
                    .map(InspectorState::Regex)
                    .map_err(|e| e.to_string())
            }
            ids::CRON => cron_tool::next_runs(&primary, self.cron_zone, 5)
                .map(|r| InspectorState::Cron {
                    lines: r.next,
                    zone: r.zone_label,
                    expr: r.normalized_expr,
                })
                .map_err(|e| e.to_string()),
            ids::URL_PARSE => url_parse::parse(&primary)
                .map(InspectorState::Url)
                .map_err(|e| e.to_string()),
            ids::DIFF => {
                let diff = diff_tool::line_diff(&primary, &secondary);
                Ok(InspectorState::Diff {
                    lines: diff.lines,
                    inserts: diff.inserts,
                    deletes: diff.deletes,
                })
            }
            _ => Err("未知检查器".into()),
        };

        match result {
            Ok(state) => {
                self.state = state;
                self.notice = "";
            }
            Err(message) => {
                self.error = Some(message);
                self.notice = "";
            }
        }
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.live_gen = self.live_gen.wrapping_add(1);
        self.primary.update(cx, |f, cx| {
            f.clear(cx);
            f.focus(window);
        });
        self.secondary.update(cx, |f, cx| f.clear(cx));
        self.tertiary.update(cx, |f, cx| f.clear(cx));
        self.state = InspectorState::Idle;
        self.error = None;
        self.notice = "";
        cx.notify();
    }

    fn title(&self) -> (&'static str, &'static str) {
        match self.tool_id {
            ids::JWT => ("JWT", "离线 · 只解码 · 未验签"),
            ids::REGEX => ("Regex", "离线 · Rust regex（不是 PCRE）"),
            ids::CRON => ("Cron", "离线 · 5 段 · 之后 5 次运行"),
            ids::URL_PARSE => ("URL Parse", "离线 · 只拆分 · 不发请求"),
            ids::DIFF => ("Diff", "离线 · 行级对比 · 无外部进程"),
            _ => ("检查器", "离线"),
        }
    }

    fn copy_blob(&mut self, text: String, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.notice = "已复制";
        cx.notify();
    }

    fn options_bar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let mut row = div().flex().flex_wrap().items_center().gap_2();
        if self.tool_id == ids::CRON {
            row = row
                .child(div().text_xs().text_color(theme.muted).child("时区"))
                .child(
                    chip(theme, "cron-local", "本机", matches!(self.cron_zone, CronZone::Local)).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.cron_zone = CronZone::Local;
                            this.schedule_live(cx);
                        }),
                    ),
                )
                .child(
                    chip(theme, "cron-utc", "UTC", matches!(self.cron_zone, CronZone::Utc)).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.cron_zone = CronZone::Utc;
                            this.schedule_live(cx);
                        }),
                    ),
                );
        }
        if self.tool_id == ids::REGEX {
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("模式与替换在上方 · 被测文本在左 · 输入即匹配"),
            );
        } else {
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("输入即更新"),
            );
        }
        row
    }

    fn clear_row(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        div()
            .flex()
            .items_center()
            .gap_2()
            .flex_shrink_0()
            .child(
                button(theme, "inspect-clear", "清空").on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        this.clear(window, cx);
                    }),
                ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("输入即更新"),
            )
    }

    /// JWT / Cron / URL: compact top input strip (not a tall side column).
    fn compact_input_strip(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let height = match self.tool_id {
            ids::JWT | ids::URL_PARSE => px(100.),
            ids::CRON => px(88.),
            _ => px(120.),
        };
        pane(theme)
            .h(height)
            .flex_shrink_0()
            .child(
                pane_header(theme)
                    .child(div().text_sm().flex_1().child(primary_label(self.tool_id)))
                    .child(
                        paste_button(theme, "inspect-paste").on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                cx.stop_propagation();
                                this.paste_into(PasteTarget::Primary, window, cx);
                            }),
                        ),
                    )
                    .child(
                        button(theme, "inspect-clear", "清空").on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                cx.stop_propagation();
                                this.clear(window, cx);
                            }),
                        ),
                    ),
            )
            .child(div().flex_1().min_h_0().p_2().child(self.primary.clone()))
    }

    /// Pattern + replace on top (always visible); haystack | results below.
    fn regex_body(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_2()
            .child(self.clear_row(cx))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .h(px(72.))
                    .flex_shrink_0()
                    .child(
                        pane(theme)
                            .flex_1()
                            .child(
                                pane_header(theme)
                                    .child(div().text_sm().flex_1().child("模式（Rust regex）"))
                                    .child(
                                        paste_button(theme, "regex-paste-pat").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                                cx.stop_propagation();
                                                this.paste_into(PasteTarget::Secondary, window, cx);
                                            }),
                                        ),
                                    ),
                            )
                            .child(div().flex_1().min_h_0().p_2().child(self.secondary.clone())),
                    )
                    .child(
                        pane(theme)
                            .flex_1()
                            .child(
                                pane_header(theme)
                                    .child(div().text_sm().flex_1().child("替换（可选）"))
                                    .child(
                                        paste_button(theme, "regex-paste-repl").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                                cx.stop_propagation();
                                                this.paste_into(PasteTarget::Tertiary, window, cx);
                                            }),
                                        ),
                                    ),
                            )
                            .child(div().flex_1().min_h_0().p_2().child(self.tertiary.clone())),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .flex()
                    .gap_2()
                    .child(
                        pane(theme)
                            .w(relative(0.42))
                            .flex_shrink_0()
                            .child(
                                pane_header(theme)
                                    .child(div().text_sm().flex_1().child("被测文本"))
                                    .child(
                                        paste_button(theme, "regex-paste-hay").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                                cx.stop_propagation();
                                                this.paste_into(PasteTarget::Primary, window, cx);
                                            }),
                                        ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .p_2()
                                    .child(self.primary.clone()),
                            ),
                    )
                    .child(self.result_area(cx)),
            )
    }

    /// Diff: side-by-side inputs capped so the result pane stays usable.
    fn diff_inputs(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        div()
            .flex()
            .flex_col()
            .gap_2()
            .h(relative(0.32))
            .flex_shrink_0()
            .min_h(px(160.))
            .max_h(px(280.))
            .child(self.clear_row(cx))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .flex_1()
                    .min_h_0()
                    .child(
                        pane(theme)
                            .flex_1()
                            .child(
                                pane_header(theme)
                                    .child(div().text_sm().flex_1().child("原文（旧）"))
                                    .child(
                                        paste_button(theme, "diff-paste-old").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                                cx.stop_propagation();
                                                this.paste_into(PasteTarget::Primary, window, cx);
                                            }),
                                        ),
                                    ),
                            )
                            .child(div().flex_1().min_h_0().p_2().child(self.primary.clone())),
                    )
                    .child(
                        pane(theme)
                            .flex_1()
                            .child(
                                pane_header(theme)
                                    .child(div().text_sm().flex_1().child("新文本"))
                                    .child(
                                        paste_button(theme, "diff-paste-new").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                                cx.stop_propagation();
                                                this.paste_into(PasteTarget::Secondary, window, cx);
                                            }),
                                        ),
                                    ),
                            )
                            .child(div().flex_1().min_h_0().p_2().child(self.secondary.clone())),
                    ),
            )
    }

    fn result_area(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::get(cx);
        match &self.state {
            InspectorState::Idle => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .rounded_lg()
                .border_1()
                .border_color(theme.border)
                .bg(theme.panel)
                .text_color(theme.muted)
                .child("输入后在这里显示结构化结果")
                .into_any_element(),
            InspectorState::Jwt(decoded) => self.jwt_result(decoded, cx),
            InspectorState::Regex(result) => self.regex_result(result, cx),
            InspectorState::Cron { lines, zone, expr } => {
                let rows: Vec<_> = lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| {
                        div()
                            .id(SharedString::from(format!("cron-{i}")))
                            .mx_2()
                            .my_1()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(theme.elevated)
                            .font(crate::theme::monospace())
                            .text_sm()
                            .child(format!("{}. {line}", i + 1))
                    })
                    .collect();
                pane(theme)
                    .flex_1()
                    .child(
                        pane_header(theme)
                            .child(
                                div()
                                    .text_sm()
                                    .flex_1()
                                    .child(format!("之后 {} 次 · {zone}", lines.len())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font(crate::theme::monospace())
                                    .text_color(theme.muted)
                                    .child(expr.clone()),
                            ),
                    )
                    .child(
                        div()
                            .id("cron-results")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.result_scroll)
                            .py_1()
                            .children(rows),
                    )
                    .into_any_element()
            }
            InspectorState::Url(parts) => self.url_result(parts, cx),
            InspectorState::Diff {
                lines,
                inserts,
                deletes,
            } => self.diff_result(lines, *inserts, *deletes, cx),
        }
    }

    fn jwt_result(&self, decoded: &JwtDecode, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::get(cx);
        let alg_text = decoded.alg.clone().unwrap_or_else(|| "(无 alg)".into());
        let header = decoded.header_raw.clone();
        let payload = decoded.payload_raw.clone();
        let sig = decoded.signature_raw.clone();

        let mut banners = div().flex().flex_col().gap_1().flex_shrink_0();
        banners = banners.child(banner(theme, StatusKind::Warning, decoded.unsigned_notice));
        if decoded.alg_is_none {
            banners = banners.child(banner(theme, StatusKind::Error,
                format!("alg 原文为 `{alg_text}` · 不受信任 · 不是验签成功"),
            ));
        } else {
            banners = banners.child(
                div()
                    .px_1()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(format!("alg 原文：{alg_text}")),
            );
        }
        if let Some(exp) = &decoded.exp {
            let kind = if exp.expired {
                StatusKind::Error
            } else {
                StatusKind::Neutral
            };
            let status = if exp.expired { "已过期" } else { "尚未过期" };
            banners = banners.child(banner(theme, kind,
                format!(
                    "exp = {} → {} · {} · 未验签",
                    exp.unix_seconds, exp.utc_display, status
                ),
            ));
        }

        // Stacked cards sized to content — no equal-height empty columns.
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .child(banners)
            .child(
                div()
                    .id("jwt-cards")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(self.json_card("header", "Header", header, cx))
                    .child(self.json_card("payload", "Payload", payload, cx))
                    .child(self.json_card("signature", "Signature（原文）", sig, cx)),
            )
            .into_any_element()
    }

    fn json_card(
        &self,
        id: &'static str,
        title: &'static str,
        body: String,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = Theme::get(cx);
        let copy = body.clone();
        let display = if body.is_empty() {
            SharedString::from("（空）")
        } else {
            SharedString::from(body)
        };
        let empty = display.as_ref() == "（空）";
        card(theme)
            .flex_shrink_0()
            .child(
                pane_header(theme)
                    .child(div().text_sm().flex_1().child(title))
                    .child(button(theme, id, "复制").on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_blob(copy.clone(), cx);
                    }))),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{id}-body")))
                    .p_3()
                    .font(crate::theme::monospace())
                    .text_sm()
                    .text_color(if empty { theme.muted } else { theme.text })
                    .child(display),
            )
    }

    fn regex_result(&self, result: &RegexResult, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::get(cx);
        let rows: Vec<_> = result
            .matches
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let caps = m
                    .captures
                    .iter()
                    .enumerate()
                    .map(|(j, c)| {
                        let label = c.name.clone().unwrap_or_else(|| j.to_string());
                        format!("  [{label}] {}..{} {:?}", c.start, c.end, c.text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                div()
                    .id(SharedString::from(format!("re-match-{i}")))
                    .p_2()
                    .mb_1()
                    .rounded_md()
                    .bg(theme.elevated)
                    .border_1()
                    .border_color(theme.border_subtle)
                    .font(crate::theme::monospace())
                    .text_xs()
                    .child(format!("#{i} {}..{} {:?}", m.start, m.end, m.full))
                    .child(div().text_color(theme.muted).child(caps))
            })
            .collect();

        let mut view = pane(theme).flex_1().child(pane_header(theme).child(
            div()
                .text_sm()
                .flex_1()
                .child(format!("{} 处匹配 · {}", result.matches.len(), result.syntax_note)),
        ));
        if let Some(replaced) = &result.replaced {
            let copy = replaced.clone();
            view = view.child(
                div()
                    .p_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .flex_1()
                                    .child("替换结果"),
                            )
                            .child(button(theme, "copy-repl", "复制").on_click(cx.listener(
                                move |this, _, _, cx| this.copy_blob(copy.clone(), cx),
                            ))),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(theme.elevated)
                            .font(crate::theme::monospace())
                            .text_sm()
                            .child(replaced.clone()),
                    ),
            );
        }
        view.child(
            div()
                .id("regex-matches")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(&self.result_scroll)
                .p_2()
                .children(rows),
        )
        .into_any_element()
    }

    fn url_result(&self, parts: &UrlParts, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::get(cx);

        // Keep important structural fields; omit blank auth / fragment noise.
        let mut field_defs: Vec<(&'static str, String, bool)> = Vec::new();
        field_defs.push(("href", parts.href.clone(), parts.href.is_empty()));
        field_defs.push(("scheme", parts.scheme.clone(), parts.scheme.is_empty()));
        if !parts.username.is_empty() {
            field_defs.push(("username", parts.username.clone(), false));
        }
        if !parts.password.is_empty() {
            field_defs.push(("password", parts.password.clone(), false));
        }
        field_defs.push(("host", parts.host.clone(), parts.host.is_empty()));
        let port_str = parts.port.map(|p| p.to_string()).unwrap_or_default();
        if !port_str.is_empty() {
            field_defs.push(("port", port_str, false));
        }
        field_defs.push(("path", parts.path.clone(), parts.path.is_empty()));
        if !parts.query.is_empty() {
            field_defs.push(("query", parts.query.clone(), false));
        }
        if !parts.fragment.is_empty() {
            field_defs.push(("fragment", parts.fragment.clone(), false));
        }

        let mut rows: Vec<AnyElement> = field_defs
            .into_iter()
            .map(|(k, v, empty)| {
                let copy = v.clone();
                div()
                    .id(SharedString::from(format!("url-{k}")))
                    .flex()
                    .items_start()
                    .gap_2()
                    .child(kv_row(theme, format!("url-kv-{k}"), k, v, empty).flex_1())
                    .child(
                        div().pt_1().child(copy_chip(format!("copy-{k}"), copy, cx)),
                    )
                    .into_any_element()
            })
            .collect();

        if !parts.query_pairs.is_empty() {
            rows.push(
                div()
                    .mt_2()
                    .mb_1()
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.accent)
                            .child("Query Params"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h(px(1.))
                            .bg(theme.border_subtle),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(format!("{} 对", parts.query_pairs.len())),
                    )
                    .into_any_element(),
            );
            for (i, (k, v)) in parts.query_pairs.iter().enumerate() {
                let copy = format!("{k}={v}");
                let label = if k.is_empty() {
                    format!("[{i}]")
                } else {
                    k.clone()
                };
                rows.push(
                    div()
                        .id(SharedString::from(format!("qp-{i}")))
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(kv_row(theme, format!("qp-kv-{i}"), label, v.clone(), v.is_empty()).flex_1())
                        .child(
                            div()
                                .pt_1()
                                .child(copy_chip(format!("copy-qp-{i}"), copy, cx)),
                        )
                        .into_any_element(),
                );
            }
        }

        pane(theme)
            .flex_1()
            .child(
                pane_header(theme).child(div().text_sm().flex_1().child("URL 各段（未发请求）")),
            )
            .child(
                div()
                    .id("url-parts")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .py_2()
                    .children(rows),
            )
            .into_any_element()
    }

    fn diff_result(
        &self,
        lines: &[DiffLine],
        inserts: usize,
        deletes: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = Theme::get(cx);
        let rows: Vec<_> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let (prefix, color, bg) = match line.kind {
                    LineKind::Equal => (" ", theme.text, theme.panel),
                    LineKind::Insert => ("+", theme.success, theme.success_bg),
                    LineKind::Delete => ("-", theme.error, theme.error_bg),
                };
                let old = line
                    .old_line
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| " ".into());
                let new = line
                    .new_line
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| " ".into());
                div()
                    .id(SharedString::from(format!("diff-{i}")))
                    .flex()
                    .gap_2()
                    .px_2()
                    .py_0p5()
                    .bg(bg)
                    .font(crate::theme::monospace())
                    .text_sm()
                    .text_color(color)
                    .child(div().w(px(28.)).text_color(theme.muted).child(old))
                    .child(div().w(px(28.)).text_color(theme.muted).child(new))
                    .child(div().w(px(14.)).font_weight(FontWeight::BOLD).child(prefix))
                    .child(div().flex_1().child(line.text.clone()))
            })
            .collect();

        pane(theme)
            .flex_1()
            .child(
                pane_header(theme)
                    .child(div().text_sm().flex_1().child("对比结果"))
                    .child(status_chip(theme, format!("+{inserts}"), StatusKind::Success))
                    .child(status_chip(theme, format!("-{deletes}"), StatusKind::Error)),
            )
            .child(
                div()
                    .id("diff-lines")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .children(rows),
            )
            .into_any_element()
    }

    fn body(&self, cx: &mut Context<Self>) -> Div {
        match self.tool_id {
            // Vertical: compact input on top, results fill the rest.
            ids::JWT | ids::URL_PARSE | ids::CRON => div()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_2()
                .child(self.compact_input_strip(cx))
                .child(self.result_area(cx)),
            // Vertical: side-by-side inputs (capped), full-width diff below.
            ids::DIFF => div()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_2()
                .child(self.diff_inputs(cx))
                .child(self.result_area(cx)),
            ids::REGEX => self.regex_body(cx),
            _ => div()
                .flex_1()
                .child(div().text_color(Theme::get(cx).muted).child("未知检查器")),
        }
    }
}

fn copy_chip(id: String, value: String, cx: &mut Context<InspectorWorkbench>) -> Stateful<Div> {
    let theme = Theme::get(cx);
    div()
        .id(SharedString::from(id))
        .cursor_pointer()
        .px_2()
        .py_0p5()
        .rounded_md()
        .text_xs()
        .flex_shrink_0()
        .bg(theme.elevated)
        .text_color(theme.muted)
        .border_1()
        .border_color(theme.border)
        .hover(move |s| s.bg(theme.selected).text_color(theme.accent))
        .child("复制")
        .on_click(cx.listener(move |this, _, _, cx| {
            this.copy_blob(value.clone(), cx);
        }))
}

#[derive(Clone, Copy)]
enum PasteTarget {
    Primary,
    Secondary,
    Tertiary,
}

fn primary_placeholder(tool_id: &str) -> &'static str {
    match tool_id {
        ids::JWT => "粘贴 JWT（header.payload.signature）",
        ids::REGEX => "被测文本",
        ids::CRON => "5 段 cron，例如 */5 * * * *",
        ids::URL_PARSE => "https://example.com/path?x=1#frag",
        ids::DIFF => "原文（旧）",
        _ => "输入",
    }
}

fn secondary_placeholder(tool_id: &str) -> &'static str {
    match tool_id {
        ids::REGEX => "模式，例如 (\\w+)@(\\w+)",
        ids::DIFF => "新文本",
        _ => "",
    }
}

fn primary_label(tool_id: &str) -> &'static str {
    match tool_id {
        ids::JWT => "Token",
        ids::CRON => "表达式",
        ids::URL_PARSE => "URL",
        _ => "输入",
    }
}

impl Render for InspectorWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = self.title();
        div()
            .id("inspector-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, window, cx| this.run(window, cx)))
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .child(
                div()
                    .id("inspector-titlebar")
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
                    .gap_2()
                    .child(div().text_lg().child(title))
                    .child(status_chip(theme, "离线", StatusKind::Neutral))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(subtitle.trim_start_matches("离线 · ").to_owned()),
                    )
                    .child(div().flex_1())
                    .when(!self.notice.is_empty(), |view| {
                        view.child(status_chip(theme, self.notice, StatusKind::Accent))
                    }),
            )
            .child(self.options_bar(cx))
            .when_some(self.error.as_ref(), |view, error| {
                view.child(banner(theme, StatusKind::Error, error.clone()))
            })
            .child(self.body(cx))
    }
}
