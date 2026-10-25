//! Explicit-network tools: HTTP Client, DNS, IP Location, Network Info, Speed Test, Whois.
//! Opening a tool never sends traffic; the user must click 运行.

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::Theme,
    ui::{
        banner, button, chip, empty_hint, multiline_mono, kv_row, pane, pane_header, pane_title,
        paste_button, primary_button, status_chip, workbench_title, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::{
    dns_lookup::{self, DnsResult, RecordKind},
    http_request::{self, HttpResponse, Method},
    ip_location, network_info, speed_test, whois_tool,
};
use gpui::{prelude::FluentBuilder, *};

enum NetworkState {
    Idle,
    Http(HttpResponse),
    Dns(DnsResult),
    Rows(Vec<(String, String)>),
}

pub struct NetworkWorkbench {
    tool_id: &'static str,
    primary: Entity<TextField>,
    headers: Entity<TextField>,
    body: Entity<TextField>,
    method: Method,
    state: NetworkState,
    error: Option<String>,
    notice: &'static str,
    result_scroll: ScrollHandle,
}

impl NetworkWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let primary = cx.new(|cx| TextField::new(false, false, "https://…", cx));
        let headers = cx.new(|cx| TextField::new(true, false, "Accept: application/json", cx));
        let body = cx.new(|cx| TextField::new(true, false, "请求体（可选）", cx));
        for field in [&primary, &headers, &body] {
            cx.subscribe(field, |this, _, _: &Edited, cx| {
                this.notice = "";
                cx.notify();
            })
            .detach();
        }
        Self {
            tool_id: ids::HTTP_REQUEST,
            primary,
            headers,
            body,
            method: Method::Get,
            state: NetworkState::Idle,
            error: None,
            notice: "",
            result_scroll: ScrollHandle::new(),
        }
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        if needs_primary(self.tool_id) {
            self.primary.read(cx).focus(window);
        }
    }

    pub fn set_tool(&mut self, tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        if self.tool_id != tool_id {
            self.tool_id = tool_id;
            self.state = NetworkState::Idle;
            self.error = None;
            self.notice = "";
            self.primary.update(cx, |f, cx| {
                f.clear(cx);
                f.set_placeholder(primary_placeholder(tool_id), cx);
            });
            self.headers.update(cx, |f, cx| f.clear(cx));
            self.body.update(cx, |f, cx| f.clear(cx));
            self.method = Method::Get;
        }
        self.focus_input(window, cx);
        cx.notify();
    }

    pub fn set_input_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.primary.update(cx, |field, cx| {
            field.set_input(text, cx);
            field.focus(window);
        });
        self.notice = "";
        cx.notify();
    }

    fn paste_into_primary(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = cx
            .read_from_clipboard()
            .and_then(|item| item.text())
            .unwrap_or_default();
        if text.is_empty() {
            self.notice = "剪贴板为空";
            cx.notify();
            return;
        }
        self.primary.update(cx, |field, cx| {
            field.set_input(&text, cx);
            field.focus(window);
        });
        self.notice = "已粘贴";
        cx.notify();
    }

    fn run(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.error = None;
        self.state = NetworkState::Idle;
        let primary = self.primary.read(cx).buffer.text().to_owned();
        let headers = self.headers.read(cx).buffer.text().to_owned();
        let body = self.body.read(cx).buffer.text().to_owned();

        let result = match self.tool_id {
            ids::HTTP_REQUEST => match http_request::prepare(self.method, &primary, &headers, &body)
            {
                Ok(prepared) => http_request::execute(&prepared)
                    .map(NetworkState::Http)
                    .map_err(|e| e.to_string()),
                Err(e) => Err(e.to_string()),
            },
            ids::DNS_LOOKUP => dns_lookup::lookup(&primary, RecordKind::Address)
                .map(NetworkState::Dns)
                .map_err(|e| e.to_string()),
            ids::IP_LOCATION => ip_location::lookup(&primary).map(NetworkState::Rows),
            ids::NETWORK_INFO => network_info::collect(true).map(NetworkState::Rows),
            ids::SPEED_TEST => speed_test::run().map(NetworkState::Rows),
            ids::WHOIS => whois_tool::lookup(&primary).map(NetworkState::Rows),
            _ => Err("未知联网工具".into()),
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
            if needs_primary(self.tool_id) {
                f.focus(window);
            }
        });
        self.headers.update(cx, |f, cx| f.clear(cx));
        self.body.update(cx, |f, cx| f.clear(cx));
        self.state = NetworkState::Idle;
        self.error = None;
        self.notice = "";
        cx.notify();
    }

    fn title(&self) -> (&'static str, &'static str) {
        match self.tool_id {
            ids::HTTP_REQUEST => ("HTTP Client", "显式发送 · 打开不联网"),
            ids::DNS_LOOKUP => ("DNS Lookup", "显式查询"),
            ids::IP_LOCATION => ("IP Location", "显式联网 · 未内置 MMDB"),
            ids::NETWORK_INFO => ("Network Information", "本机网卡 + 公网 IP"),
            ids::SPEED_TEST => ("Speed Test", "粗测下载 / 延迟"),
            ids::WHOIS => ("Whois", "TCP 43 · 显式连接"),
            _ => ("网络工具", ""),
        }
    }

    fn method_bar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let methods = [
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Delete,
            Method::Patch,
            Method::Head,
        ];
        let mut row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_1()
            .child(div().text_xs().text_color(theme.muted).child("方法"));
        for method in methods {
            let active = self.method == method;
            row = row.child(
                chip(theme, method.as_str(), method.as_str(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.method = method;
                        cx.notify();
                    },
                )),
            );
        }
        row
    }

    fn result_view(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = Theme::get(cx);
        match &self.state {
            NetworkState::Idle => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(empty_hint(theme, "点击「运行」查看结果"))
                .into_any_element(),
            NetworkState::Http(response) => {
                let header_rows: Vec<_> = response
                    .headers
                    .iter()
                    .enumerate()
                    .map(|(i, (k, v))| {
                        kv_row(theme, format!("hdr-{i}"), k.clone(), v.clone(), false)
                    })
                    .collect();
                div()
                    .id("http-result")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(status_chip(
                                theme,
                                format!("{} {}", response.status, response.status_text),
                                if (200..300).contains(&response.status) {
                                    StatusKind::Success
                                } else if response.status >= 400 {
                                    StatusKind::Error
                                } else {
                                    StatusKind::Warning
                                },
                            ))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .font(crate::theme::monospace())
                                    .child(response.url.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.muted)
                            .child("响应头"),
                    )
                    .children(header_rows)
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.muted)
                            .child("响应体"),
                    )
                    .child(
                        div()
                            .w_full()
                            .p_2()
                            .rounded_md()
                            .bg(theme.elevated)
                            .border_1()
                            .border_color(theme.border)
                            .font(crate::theme::monospace())
                            .text_sm()
                            .child(multiline_mono(theme, response.body.as_str())),
                    )
                    .into_any_element()
            }
            NetworkState::Dns(result) => {
                let rows: Vec<_> = result
                    .addresses
                    .iter()
                    .enumerate()
                    .map(|(i, ip)| {
                        let kind = if ip.is_ipv4() { "A" } else { "AAAA" };
                        kv_row(theme, format!("ip-{i}"), kind, ip.to_string(), false)
                    })
                    .collect();
                div()
                    .id("dns-result")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(status_chip(theme, result.kind.label(), StatusKind::Accent))
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_sm()
                                    .child(result.name.clone()),
                            )
                            .child(status_chip(
                                theme,
                                format!("{} 条", result.addresses.len()),
                                StatusKind::Neutral,
                            )),
                    )
                    .children(rows)
                    .into_any_element()
            }
            NetworkState::Rows(rows) => {
                let view_rows: Vec<_> = rows
                    .iter()
                    .enumerate()
                    .map(|(i, (k, v))| {
                        let multiline = v.contains('\n') || v.len() > 120;
                        if multiline {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(theme.muted)
                                        .child(k.clone()),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .p_2()
                                        .rounded_md()
                                        .bg(theme.elevated)
                                        .border_1()
                                        .border_color(theme.border)
                                        .font(crate::theme::monospace())
                                        .text_sm()
                                        .child(multiline_mono(theme, v.as_str())),
                                )
                                .into_any_element()
                        } else {
                            kv_row(theme, format!("row-{i}"), k.clone(), v.clone(), false)
                                .into_any_element()
                        }
                    })
                    .collect();
                div()
                    .id("net-rows")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .p_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(view_rows)
                    .into_any_element()
            }
        }
    }
}

fn needs_primary(tool_id: &str) -> bool {
    !matches!(tool_id, ids::NETWORK_INFO | ids::SPEED_TEST)
}

fn primary_placeholder(tool_id: &str) -> &'static str {
    match tool_id {
        ids::HTTP_REQUEST => "https://example.com/",
        ids::DNS_LOOKUP => "example.com 或 localhost",
        ids::IP_LOCATION => "8.8.8.8 或 2001:4860:4860::8888",
        ids::WHOIS => "example.com",
        ids::NETWORK_INFO => "（无需输入；点击运行）",
        ids::SPEED_TEST => "（无需输入；点击运行）",
        _ => "输入",
    }
}

fn primary_label(tool_id: &str) -> &'static str {
    match tool_id {
        ids::HTTP_REQUEST => "URL",
        ids::DNS_LOOKUP => "主机名",
        ids::IP_LOCATION => "IP",
        ids::WHOIS => "域名 / IP",
        _ => "输入",
    }
}

impl Render for NetworkWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = self.title();
        let is_http = self.tool_id == ids::HTTP_REQUEST;
        let show_primary = needs_primary(self.tool_id);

        div()
            .id("network-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, window, cx| this.run(window, cx)))
            .size_full()
            .min_w_0()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(crate::theme::space::MD))
            .p(px(crate::theme::space::XL))
            .child(
                workbench_title(theme, title, subtitle)
                    .child(status_chip(theme, "会联网", StatusKind::Warning))
                    .child(div().flex_1())
                    .when(!self.notice.is_empty(), |view| {
                        view.child(status_chip(theme, self.notice, StatusKind::Success))
                    }),
            )
            .when(is_http, |view| view.child(self.method_bar(cx)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        primary_button(theme, "network-run", "运行").on_click(cx.listener(
                            |this, _, window, cx| this.run(window, cx),
                        )),
                    )
                    .child(button(theme, "network-clear", "清空").on_click(
                        cx.listener(|this, _, window, cx| this.clear(window, cx)),
                    )),
            )
            .when_some(self.error.clone(), |view, message| {
                view.child(banner(theme, StatusKind::Error, message))
            })
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .when(show_primary, |view| {
                        view.child(
                            pane(theme)
                                .h(if is_http { px(120.) } else { px(80.) })
                                .flex_shrink_0()
                                .child(
                                    pane_header(theme)
                                        .child(pane_title(theme, primary_label(self.tool_id)))
                                        .child(
                                            paste_button(theme, "network-paste").on_mouse_down(
                                                MouseButton::Left,
                                                cx.listener(
                                                    |this, _: &MouseDownEvent, window, cx| {
                                                        cx.stop_propagation();
                                                        this.paste_into_primary(window, cx);
                                                    },
                                                ),
                                            ),
                                        ),
                                )
                                .child(div().flex_1().min_h_0().p_2().child(self.primary.clone())),
                        )
                    })
                    .when(is_http, |view| {
                        view.child(
                            div()
                                .h(px(160.))
                                .flex_shrink_0()
                                .flex()
                                .gap_2()
                                .child(
                                    pane(theme)
                                        .flex_1()
                                        .child(
                                            pane_header(theme).child(pane_title(
                                                theme,
                                                "请求头（Key: Value）",
                                            )),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_h_0()
                                                .p_2()
                                                .child(self.headers.clone()),
                                        ),
                                )
                                .child(
                                    pane(theme)
                                        .flex_1()
                                        .child(
                                            pane_header(theme).child(pane_title(theme, "请求体")),
                                        )
                                        .child(
                                            div().flex_1().min_h_0().p_2().child(self.body.clone()),
                                        ),
                                ),
                        )
                    })
                    .child(
                        pane(theme)
                            .flex_1()
                            .child(pane_header(theme).child(pane_title(theme, "结果")))
                            .child(self.result_view(cx)),
                    ),
            )
    }
}
