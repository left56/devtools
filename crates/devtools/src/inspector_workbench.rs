//! M3 inspector UIs: JWT, Regex, Cron, URL Parse, Diff.
//! Structured results live here; algorithms stay in `devtools-tools`.

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::Theme,
    ui::{button, chip, pane, pane_header},
};
use devtools_core::ids;
use devtools_tools::{
    cron_tool::{self, Zone as CronZone},
    diff_tool::{self, DiffLine, LineKind},
    jwt::{self, JwtDecode},
    regex_tool::{self, RegexResult},
    url_parse::{self, UrlParts},
};
use gpui::{prelude::FluentBuilder, *};

enum InspectorState {
    Idle,
    Jwt(JwtDecode),
    Regex(RegexResult),
    Cron { lines: Vec<String>, zone: &'static str, expr: String },
    Url(UrlParts),
    Diff { lines: Vec<DiffLine>, inserts: usize, deletes: usize },
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
}

impl InspectorWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let primary = cx.new(|cx| TextField::new(true, false, "输入", cx));
        let secondary = cx.new(|cx| TextField::new(true, false, "", cx));
        let tertiary = cx.new(|cx| TextField::new(false, false, "替换（可选）", cx));
        for field in [&primary, &secondary, &tertiary] {
            cx.subscribe(field, |this, _, _: &Edited, cx| {
                this.notice = "";
                cx.notify();
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
        }
        self.focus_input(window, cx);
        cx.notify();
    }

    fn run(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let primary = self.primary.read(cx).buffer.text().to_owned();
        let secondary = self.secondary.read(cx).buffer.text().to_owned();
        let tertiary = self.tertiary.read(cx).buffer.text().to_owned();
        self.error = None;
        self.state = InspectorState::Idle;

        let result = match self.tool_id {
            ids::JWT => jwt::decode(&primary).map(InspectorState::Jwt).map_err(|e| e.to_string()),
            ids::REGEX => {
                let replacement = {
                    let t = tertiary.trim();
                    if t.is_empty() { None } else { Some(t) }
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
                self.notice = "已更新结果";
            }
            Err(message) => {
                self.error = Some(message);
                self.notice = "";
            }
        }
        cx.notify();
    }

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        let theme = Theme::dark();
        let mut row = div().flex().flex_wrap().items_center().gap_2();
        if self.tool_id == ids::CRON {
            row = row
                .child(div().text_xs().text_color(theme.muted).child("时区"))
                .child(chip("cron-local", "本机", matches!(self.cron_zone, CronZone::Local)).on_click(
                    cx.listener(|this, _, window, cx| {
                        this.cron_zone = CronZone::Local;
                        this.run(window, cx);
                    }),
                ))
                .child(chip("cron-utc", "UTC", matches!(self.cron_zone, CronZone::Utc)).on_click(
                    cx.listener(|this, _, window, cx| {
                        this.cron_zone = CronZone::Utc;
                        this.run(window, cx);
                    }),
                ));
        }
        if self.tool_id == ids::REGEX {
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("模式在下方第二框 · 可选替换在第三框 · Ctrl+Enter 运行"),
            );
        }
        row
    }

    fn run_clear_row(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::dark();
        div()
            .flex()
            .gap_2()
            .flex_shrink_0()
            .child(
                button("inspect-run", "运行").on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, window, cx| {
                        cx.stop_propagation();
                        this.run(window, cx);
                    }),
                ),
            )
            .child(
                button("inspect-clear", "清空").on_mouse_down(
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
                    .child("Ctrl+Enter 运行"),
            )
    }

    fn input_area(&self, cx: &mut Context<Self>) -> Div {
        match self.tool_id {
            ids::REGEX => div()
                .flex()
                .flex_col()
                .gap_2()
                .flex_1()
                .min_h_0()
                .child(self.run_clear_row(cx))
                .child(
                    pane()
                        .h(px(72.))
                        .flex_shrink_0()
                        .child(pane_header().child(div().text_sm().child("模式（Rust regex）")))
                        .child(div().flex_1().p_2().child(self.secondary.clone())),
                )
                .child(
                    pane()
                        .flex_1()
                        .child(pane_header().child(div().text_sm().flex_1().child("被测文本")))
                        .child(div().flex_1().min_h_0().p_2().child(self.primary.clone())),
                )
                .child(
                    pane()
                        .h(px(64.))
                        .flex_shrink_0()
                        .child(pane_header().child(div().text_sm().child("替换（可选）")))
                        .child(div().flex_1().p_2().child(self.tertiary.clone())),
                ),
            ids::DIFF => div()
                .flex()
                .flex_col()
                .gap_2()
                .flex_1()
                .min_h_0()
                .child(self.run_clear_row(cx))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .flex_1()
                        .min_h_0()
                        .child(
                            pane()
                                .flex_1()
                                .child(pane_header().child(div().text_sm().child("原文（旧）")))
                                .child(div().flex_1().min_h_0().p_2().child(self.primary.clone())),
                        )
                        .child(
                            pane()
                                .flex_1()
                                .child(pane_header().child(div().text_sm().child("新文本")))
                                .child(
                                    div().flex_1().min_h_0().p_2().child(self.secondary.clone()),
                                ),
                        ),
                ),
            _ => pane()
                .flex_1()
                .child(
                    pane_header()
                        .child(div().text_sm().flex_1().child(primary_label(self.tool_id)))
                        .child(
                            button("inspect-run", "运行").on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                    cx.stop_propagation();
                                    this.run(window, cx);
                                }),
                            ),
                        )
                        .child(
                            button("inspect-clear", "清空").on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                    cx.stop_propagation();
                                    this.clear(window, cx);
                                }),
                            ),
                        ),
                )
                .child(div().flex_1().min_h_0().p_3().child(self.primary.clone())),
        }
    }

    fn result_area(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::dark();
        match &self.state {
            InspectorState::Idle => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_color(theme.muted)
                .child("运行后在这里显示结构化结果")
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
                            .px_3()
                            .py_2()
                            .font(crate::theme::monospace())
                            .child(format!("{}. {line}", i + 1))
                    })
                    .collect();
                pane()
                    .flex_1()
                    .child(
                        pane_header()
                            .child(div().text_sm().flex_1().child(format!("之后 {} 次 · {zone}", lines.len())))
                            .child(div().text_xs().text_color(theme.muted).child(expr.clone())),
                    )
                    .child(
                        div()
                            .id("cron-results")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.result_scroll)
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
        let theme = Theme::dark();
        let alg_text = decoded.alg.clone().unwrap_or_else(|| "(无 alg)".into());
        let header = decoded.header_raw.clone();
        let payload = decoded.payload_raw.clone();
        let sig = decoded.signature_raw.clone();
        let mut warnings = div().flex().flex_col().gap_1();
        warnings = warnings.child(
            div()
                .text_sm()
                .text_color(rgb(0xe5c78b))
                .child(decoded.unsigned_notice),
        );
        if decoded.alg_is_none {
            warnings = warnings.child(
                div()
                    .text_sm()
                    .text_color(theme.error)
                    .child(format!("alg 原文为 `{alg_text}` · 不受信任 · 不是验签成功")),
            );
        } else {
            warnings = warnings.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(format!("alg 原文：{alg_text}")),
            );
        }
        if let Some(exp) = &decoded.exp {
            let color = if exp.expired { theme.error } else { theme.muted };
            let status = if exp.expired { "已过期" } else { "尚未过期" };
            warnings = warnings.child(
                div()
                    .text_sm()
                    .text_color(color)
                    .child(format!(
                        "exp = {} → {} · {} · 未验签",
                        exp.unix_seconds, exp.utc_display, status
                    )),
            );
        }

        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .child(warnings)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .gap_2()
                    .child(self.json_pane("header", "Header", header, cx))
                    .child(self.json_pane("payload", "Payload", payload, cx))
                    .child(self.json_pane("signature", "Signature（原文）", sig, cx)),
            )
            .into_any_element()
    }

    fn json_pane(
        &self,
        id: &'static str,
        title: &'static str,
        body: String,
        cx: &mut Context<Self>,
    ) -> Div {
        let copy = body.clone();
        pane()
            .flex_1()
            .child(
                pane_header()
                    .child(div().text_sm().flex_1().child(title))
                    .child(button(id, "复制").on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_blob(copy.clone(), cx);
                    }))),
            )
            .child(
                div()
                    .id(SharedString::from(format!("{id}-body")))
                    .flex_1()
                    .min_h_0()
                    .p_3()
                    .font(crate::theme::monospace())
                    .text_sm()
                    .overflow_y_scroll()
                    .child(body),
            )
    }

    fn regex_result(&self, result: &RegexResult, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::dark();
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
                        let label = c
                            .name
                            .clone()
                            .unwrap_or_else(|| j.to_string());
                        format!("  [{label}] {}..{} {:?}", c.start, c.end, c.text)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                div()
                    .id(SharedString::from(format!("re-match-{i}")))
                    .p_2()
                    .mb_1()
                    .rounded_md()
                    .bg(theme.bg)
                    .font(crate::theme::monospace())
                    .text_xs()
                    .child(format!("#{i} {}..{} {:?}", m.start, m.end, m.full))
                    .child(div().text_color(theme.muted).child(caps))
            })
            .collect();

        let mut view = pane().flex_1().child(
            pane_header()
                .child(
                    div()
                        .text_sm()
                        .flex_1()
                        .child(format!("{} 处匹配 · {}", result.matches.len(), result.syntax_note)),
                ),
        );
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
                            .child(div().text_xs().text_color(theme.muted).flex_1().child("替换结果"))
                            .child(button("copy-repl", "复制").on_click(cx.listener(
                                move |this, _, _, cx| this.copy_blob(copy.clone(), cx),
                            ))),
                    )
                    .child(
                        div()
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
        let theme = Theme::dark();
        let fields = [
            ("href", parts.href.as_str()),
            ("scheme", parts.scheme.as_str()),
            ("username", parts.username.as_str()),
            ("password", parts.password.as_str()),
            ("host", parts.host.as_str()),
            (
                "port",
                &parts
                    .port
                    .map(|p| p.to_string())
                    .unwrap_or_default(),
            ),
            ("path", parts.path.as_str()),
            ("query", parts.query.as_str()),
            ("fragment", parts.fragment.as_str()),
        ];
        let mut rows: Vec<AnyElement> = fields
            .into_iter()
            .map(|(k, v)| {
                let copy = v.to_owned();
                div()
                    .id(SharedString::from(format!("url-{k}")))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .w(px(90.))
                            .flex_shrink_0()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(k),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .font(crate::theme::monospace())
                            .text_sm()
                            .child(v.to_owned()),
                    )
                    .child(copy_chip(format!("copy-{k}"), copy.clone(), cx))
                    .into_any_element()
            })
            .collect();

        for (i, (k, v)) in parts.query_pairs.iter().enumerate() {
            let copy = format!("{k}={v}");
            rows.push(
                div()
                    .id(SharedString::from(format!("qp-{i}")))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .child(
                        div()
                            .w(px(90.))
                            .text_xs()
                            .text_color(theme.accent)
                            .child(format!("query[{i}]")),
                    )
                    .child(
                        div()
                            .flex_1()
                            .font(crate::theme::monospace())
                            .text_sm()
                            .child(copy.clone()),
                    )
                    .child(copy_chip(format!("copy-qp-{i}"), copy.clone(), cx))
                    .into_any_element(),
            );
        }

        pane()
            .flex_1()
            .child(pane_header().child(div().text_sm().child("URL 各段（未发请求）")))
            .child(
                div()
                    .id("url-parts")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .children(rows),
            )
            .into_any_element()
    }

    fn diff_result(
        &self,
        lines: &[DiffLine],
        inserts: usize,
        deletes: usize,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = Theme::dark();
        let rows: Vec<_> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let (prefix, color, bg) = match line.kind {
                    LineKind::Equal => (" ", theme.text, theme.panel),
                    LineKind::Insert => ("+", rgb(0xa8d6aa), rgb(0x1e2a22)),
                    LineKind::Delete => ("-", rgb(0xff9292), rgb(0x2a1e1e)),
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
                    .py_1()
                    .bg(bg)
                    .font(crate::theme::monospace())
                    .text_sm()
                    .text_color(color)
                    .child(div().w(px(28.)).text_color(theme.muted).child(old))
                    .child(div().w(px(28.)).text_color(theme.muted).child(new))
                    .child(div().w(px(14.)).child(prefix))
                    .child(div().flex_1().child(line.text.clone()))
            })
            .collect();

        pane()
            .flex_1()
            .child(
                pane_header().child(
                    div()
                        .text_sm()
                        .child(format!("+{inserts}  /  -{deletes}")),
                ),
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
}


fn copy_chip(id: String, value: String, cx: &mut Context<InspectorWorkbench>) -> Stateful<Div> {
    let theme = Theme::dark();
    div()
        .id(SharedString::from(id))
        .cursor_pointer()
        .px_2()
        .py_1()
        .rounded_md()
        .text_xs()
        .bg(theme.selected)
        .hover(move |s| s.bg(theme.border))
        .child("复制")
        .on_click(cx.listener(move |this, _, _, cx| {
            this.copy_blob(value.clone(), cx);
        }))
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
        let theme = Theme::dark();
        let (title, subtitle) = self.title();
        div()
            .id("inspector-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, window, cx| this.run(window, cx)))
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
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
                    .gap_3()
                    .child(div().text_lg().child(title))
                    .child(div().text_xs().text_color(theme.muted).child(subtitle))
                    .child(div().flex_1())
                    .child(div().text_xs().text_color(theme.accent).child(self.notice)),
            )
            .child(self.options_bar(cx))
            .when_some(self.error.as_ref(), |view, error| {
                view.child(div().text_sm().text_color(theme.error).child(error.clone()))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .w(relative(0.42))
                            .flex_shrink_0()
                            .h_full()
                            .flex()
                            .flex_col()
                            .child(self.input_area(cx)),
                    )
                    .child(self.result_area(cx)),
            )
    }
}
