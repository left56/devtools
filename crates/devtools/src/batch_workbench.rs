//! One-input → many labeled outputs (it-tools date-converter style).
//!
//! Live-updates for pure transforms. Generators are not hosted here.

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::{self, Theme},
    ui::{
        button, chip, empty_hint, multiline_mono, pane, pane_header, pane_title, paste_button,
        primary_button, single_line, status_chip, workbench_title, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::{
    ascii_drawer, base64, base64_hex, basic_auth, bcrypt_tool, binary_tool, bip39_tool, case,
    chmod_calc,
    color_tool, cron_presets, csv_tool, data_format, docker_compose, email_norm, encrypt_tool,
    eta_tool, git_memo, hash, hmac_tool, html_escape, http_status, iban_tool, ipv4_tool, ipv6_ula, json_diff_tool,
    keycode_info, list_conv, mac_gen, markdown_html, math_eval, meta_tags, mime_tool, nato_alpha,
    number_base,
    number_base::Base,
    numeronym, obfuscator, otp_tool, password_strength, percentage, phone_tool, quantity, qr_tool,
    random_port, regex_memo, roman, rsa_tool, safelink, slugify, sql_format, svg_placeholder,
    temperature, text_stats, timestamp, ua_parse, unicode_escape, url_encode, wifi_qr, xml_tool,
};
use gpui::{prelude::FluentBuilder, *};

pub struct BatchWorkbench {
    tool_id: &'static str,
    input: Entity<TextField>,
    rows: Vec<(&'static str, String)>,
    error: Option<String>,
    notice: &'static str,
    num_from: Base,
    bip39_words: usize,
    quantity: usize,
    result_scroll: ScrollHandle,
}

impl BatchWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextField::new(false, false, "在这里粘贴输入", cx));
        cx.subscribe(&input, |this, _, _: &Edited, cx| {
            this.notice = "";
            if this.tool_id == ids::BIP39 {
                this.validate_bip39(cx);
            } else if !devtools_tools::is_batch_generate_tool(this.tool_id) {
                this.live_refresh(cx);
            } else {
                cx.notify();
            }
        })
        .detach();
        Self {
            tool_id: ids::TIMESTAMP,
            input,
            rows: Vec::new(),
            error: None,
            notice: "",
            num_from: Base::DEC,
            bip39_words: 12,
            quantity: 1,
            result_scroll: ScrollHandle::new(),
        }
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        if devtools_tools::is_batch_generate_tool(self.tool_id) && self.tool_id != ids::BIP39 {
            return;
        }
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
            self.quantity = 1;
            self.input.update(cx, |field, cx| {
                field.set_placeholder(placeholder_for(tool_id), cx);
                field.set_multiline(
                    !devtools_tools::is_batch_single_line_tool(tool_id),
                    cx,
                );
                if devtools_tools::is_batch_generate_tool(tool_id) {
                    field.clear(cx);
                }
            });
        }
        if tool_id == ids::TIMESTAMP {
            let current = self.input.read(cx).buffer.text().to_owned();
            if current.trim().is_empty() {
                let seed = timestamp::now_seed();
                self.input.update(cx, |field, cx| field.set_input(&seed, cx));
            }
        }
        if !devtools_tools::is_batch_generate_tool(tool_id) {
            self.live_refresh(cx);
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

    fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if devtools_tools::is_batch_generate_tool(self.tool_id) {
            self.rows.clear();
            self.error = None;
            self.notice = "";
            if self.tool_id == ids::BIP39 {
                self.input.update(cx, |field, cx| {
                    field.clear(cx);
                });
            }
            cx.notify();
            return;
        }
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

    fn generate_random(&mut self, cx: &mut Context<Self>) {
        // Clear optional BIP39 validate field so result is clearly a fresh generate.
        if self.tool_id == ids::BIP39 {
            self.input.update(cx, |field, cx| field.clear(cx));
        }
        match run_batch_generate(self.tool_id, self.bip39_words, self.quantity) {
            Ok(rows) => {
                self.rows = rows;
                self.error = None;
                self.notice = "已生成";
            }
            Err(message) => {
                self.rows.clear();
                self.error = Some(message);
            }
        }
        cx.notify();
    }

    fn validate_bip39(&mut self, cx: &mut Context<Self>) {
        let input = self.input.read(cx).buffer.text().to_owned();
        if input.trim().is_empty() {
            self.error = None;
            cx.notify();
            return;
        }
        match bip39_tool::convert_all(&input) {
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

    fn live_refresh(&mut self, cx: &mut Context<Self>) {
        if devtools_tools::is_batch_generate_tool(self.tool_id) {
            cx.notify();
            return;
        }
        let input = self.input.read(cx).buffer.text().to_owned();
        if input.trim().is_empty()
            && self.tool_id != ids::TEXT_STATS
            && self.tool_id != ids::CRON_PRESETS
            && self.tool_id != ids::GIT_MEMO
            && self.tool_id != ids::REGEX_MEMO
        {
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
            ids::BASE64 => ("Base64", "标准 / URL-safe 编码，可解码时一并展示"),
            ids::URL_ENCODE => ("URL Encode", "编码与解码批量 · 含 % 优先解码"),
            ids::HTML_ESCAPE => ("HTML Escape", "转义与反转义批量"),
            ids::HASH => ("Hash", "MD5/SHA 全家桶 · 弱算法已标注"),
            ids::CASE => ("Case", "snake / camel / Pascal / kebab / CONSTANT / Title"),
            ids::SLUGIFY => ("Slugify", "Slug 与规范化变体"),
            ids::NUMBER_BASE => ("进制转换", "同时显示 2/8/10/16 · 可识别 0x/0b/0o"),
            ids::TEXT_STATS => ("文本统计", "字符 · 词 · 行 · 字节"),
            ids::ROMAN => ("罗马数字", "1–3999 · 自动识别方向"),
            ids::PERCENTAGE => ("百分比", "输入两个数字，例如 25 200"),
            ids::CHMOD => ("Chmod", "755 ↔ rwxr-xr-x"),
            ids::BINARY => ("文本 / 二进制", "二进制 · 十六进制 · 码点"),
            ids::COLOR => ("颜色转换", "#hex · rgb · hsl"),
            ids::BASIC_AUTH => ("Basic Auth", "user:password → Authorization"),
            ids::HMAC => ("HMAC", "第一行密钥，其余为消息"),
            ids::OTP => ("OTP / TOTP", "Base32 密钥 · 可选第二行验证码"),
            ids::EMAIL_NORM => ("邮箱规范化", "小写 · Gmail 点号 · +tag"),
            ids::LIST_CONV => ("列表转换", "排序 · 反转 · 去重 · 连接"),
            ids::JSON_YAML => ("JSON ↔ YAML", "自动识别 · 给出 JSON/YAML/TOML"),
            ids::JSON_TOML => ("JSON ↔ TOML", "自动识别 · 给出 JSON/YAML/TOML"),
            ids::SQL_FORMAT => ("SQL 格式化", "轻量美化 / 压缩"),
            ids::MARKDOWN_HTML => ("Markdown → HTML", "CommonMark"),
            ids::IPV4_SUBNET => ("IPv4 子网", "CIDR 计算"),
            ids::IPV4_ADDR => ("IPv4 地址", "点分 · 整数 · 十六进制"),
            ids::IPV4_RANGE => ("IPv4 范围", "起止 → CIDR"),
            ids::IPV6_ULA => ("IPv6 ULA", "点击生成 · 可设数量"),
            ids::MAC_GEN => ("MAC 生成", "可设数量 · 无 OUI 查询"),
            ids::MIME => ("MIME 类型", "扩展名 ↔ Content-Type"),
            ids::HTTP_STATUS => ("HTTP 状态码", "数字或关键词"),
            ids::UA_PARSE => ("User-Agent", "轻量启发式解析"),
            ids::IBAN => ("IBAN", "mod-97 校验"),
            ids::TEMPERATURE => ("温度换算", "C / F / K"),
            ids::ETA => ("ETA", "大小 速度 → 耗时"),
            ids::NUMERONYM => ("Numeronym", "i18n / k8s"),
            ids::OBFUSCATE => ("字符串遮罩", "脱敏截图"),
            ids::NATO => ("NATO 音标", "拼读字母"),
            ids::SAFELINK => ("Safelink 解码", "提取真实 URL"),
            ids::DOCKER_RUN => ("Docker → Compose", "docker run 粗转"),
            ids::XML_TOOL => ("XML / JSON", "美化 · 压缩 · 互转"),
            ids::JSON_CSV => ("JSON ↔ CSV", "对象数组"),
            ids::YAML_TOML => ("YAML ↔ TOML", "自动识别 · 给出 JSON/YAML/TOML"),
            ids::JSON_DIFF => ("JSON diff", "用 --- 分隔左右"),
            ids::PASSWORD_STRENGTH => ("口令强度", "离线启发式"),
            ids::KEYCODE => ("按键码", "名称 · keyCode · code"),
            ids::PHONE => ("电话整理", "E.164 猜测"),
            ids::ASCII_DRAW => ("ASCII 横幅", "极简字模"),
            ids::RANDOM_PORT => ("随机端口", "点击生成 · 可设数量"),
            ids::UNICODE_ESC => ("Unicode 转义", "\\u / U+"),
            ids::QR_CODE => ("QR 码", "SVG · ASCII"),
            ids::WIFI_QR => ("Wi‑Fi QR", "SSID / 加密 / 密码"),
            ids::META_TAGS => ("Meta 标签", "OG / Twitter"),
            ids::SVG_PLACEHOLDER => ("SVG 占位图", "宽x高"),
            ids::CRON_PRESETS => ("Cron 模板", "常用表达式 · 字段说明"),
            ids::MATH_EVAL => ("数学求值", "安全表达式"),
            ids::GIT_MEMO => ("Git 速查", "关键词过滤"),
            ids::REGEX_MEMO => ("正则速查", "与测试工具分开"),
            ids::BASE64_HEX => ("Base64 ↔ Hex", "粘贴载荷"),
            ids::BCRYPT => ("bcrypt", "哈希 / 校验"),
            ids::ENCRYPT => ("AES 加解密", "口令派生 · 学习用"),
            ids::RSA_KEYGEN => ("RSA 密钥", "点击生成 PEM · 数量≤5"),
            ids::BIP39 => ("BIP39", "生成助记词 · 可粘贴校验 · 非钱包"),
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
        } else if self.tool_id == ids::BIP39 {
            row = row.child(div().text_xs().text_color(theme.muted).child("词数"));
            for n in [12usize, 24] {
                let active = self.bip39_words == n;
                let id = format!("bip39-words-{n}");
                row = row.child(chip(theme, id, n.to_string(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.bip39_words = n;
                        cx.notify();
                    },
                )));
            }
            row = row.child(div().text_xs().text_color(theme.muted).child("数量"));
            for n in quantity::CHOICES {
                let n = *n;
                let active = self.quantity == n;
                let id = format!("batch-qty-{n}");
                row = row.child(chip(theme, id, n.to_string(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.quantity = n;
                        cx.notify();
                    },
                )));
            }
            row = row
                .child(
                    primary_button(theme, "batch-generate", "生成").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                            this.generate_random(cx);
                        }),
                    ),
                )
                .child(button(theme, "batch-clear", "清空结果").on_click(
                    cx.listener(|this, _, window, cx| this.clear(window, cx)),
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child("下方可粘贴助记词校验"),
                );
        } else if self.tool_id == ids::RSA_KEYGEN {
            row = row.child(div().text_xs().text_color(theme.muted).child("数量"));
            for n in [1usize, 5] {
                let active = self.quantity == n;
                let id = format!("rsa-qty-{n}");
                row = row.child(chip(theme, id, n.to_string(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.quantity = n;
                        cx.notify();
                    },
                )));
            }
            row = row
                .child(
                    primary_button(theme, "batch-generate", "生成").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                            this.generate_random(cx);
                        }),
                    ),
                )
                .child(button(theme, "batch-clear", "清空结果").on_click(
                    cx.listener(|this, _, window, cx| this.clear(window, cx)),
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child("RSA 较慢，数量上限 5"),
                );
        } else if devtools_tools::is_batch_generate_tool(self.tool_id) {
            row = row.child(div().text_xs().text_color(theme.muted).child("数量"));
            for n in quantity::CHOICES {
                let n = *n;
                let active = self.quantity == n;
                let id = format!("batch-qty-{n}");
                row = row.child(chip(theme, id, n.to_string(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.quantity = n;
                        cx.notify();
                    },
                )));
            }
            row = row
                .child(
                    primary_button(theme, "batch-generate", "生成").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                            this.generate_random(cx);
                        }),
                    ),
                )
                .child(button(theme, "batch-clear", "清空结果").on_click(
                    cx.listener(|this, _, window, cx| this.clear(window, cx)),
                ))
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child("每行可单独复制"),
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
            .child(pane_title(theme, input_pane_title(self.tool_id)))
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

    /// Multi-line / generate tools: values may wrap; enclosed in a pane.
    fn result_pane(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let header = pane_header(theme).child(pane_title(theme, "结果"));
        let mut pane_view = pane(theme).flex_1().child(header);
        if self.rows.is_empty() && self.error.is_none() {
            let hint = if self.tool_id == ids::BIP39 {
                "点击「生成」，或在下方粘贴助记词校验"
            } else if devtools_tools::is_batch_generate_tool(self.tool_id) {
                "点击「生成」显示结果"
            } else {
                "输入后显示全部格式"
            };
            pane_view = pane_view.child(empty_hint(theme, hint));
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
                        .items_start()
                        .gap(px(theme::space::SM))
                        .px(px(theme::space::MD))
                        .py(px(theme::space::XS))
                        .child(
                            div()
                                .w(px(128.))
                                .flex_shrink_0()
                                .pt(px(theme::space::XS))
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
                                .child(multiline_mono(theme, value.as_str())),
                        )
                        .child(
                            div()
                                .pt(px(theme::space::XS))
                                .child(copy_chip(format!("batch-copy-{i}"), copy, cx)),
                        )
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

    /// it-tools date-converter style: no enclosing result pane; compact
    /// single-line rows (label | value | 复制) under the input.
    fn single_line_results(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let list = div()
            .id("batch-single-results")
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_y_scroll()
            .track_scroll(&self.result_scroll)
            .flex()
            .flex_col()
            .gap(px(theme::space::SM));
        if self.rows.is_empty() && self.error.is_none() {
            return list.child(
                div()
                    .pt(px(theme::space::LG))
                    .text_sm()
                    .text_color(theme.faint)
                    .child("输入后显示全部格式"),
            );
        }
        let rows: Vec<AnyElement> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let copy = value.clone();
                div()
                    .id(SharedString::from(format!("batch-sl-row-{i}")))
                    .flex()
                    .items_center()
                    .gap(px(theme::space::SM))
                    .child(
                        div()
                            .w(px(120.))
                            .flex_shrink_0()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.muted)
                            .child(single_line(*label)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h(px(36.))
                            .px(px(theme::space::SM))
                            .flex()
                            .items_center()
                            .rounded(px(theme::radius::SM))
                            .bg(theme.elevated)
                            .border_1()
                            .border_color(theme.border_subtle)
                            .font(crate::theme::monospace())
                            .text_sm()
                            .text_color(theme.text)
                            .child(single_line(value.as_str())),
                    )
                    .child(copy_chip(format!("batch-sl-copy-{i}"), copy, cx))
                    .into_any_element()
            })
            .collect();
        list.children(rows)
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
        ids::BASE64 => "粘贴文本或 Base64",
        ids::URL_ENCODE => "粘贴组件文本或百分号编码",
        ids::HTML_ESCAPE => "粘贴 HTML 或实体文本",
        ids::HASH => "粘贴要计算摘要的文本",
        ids::CASE => "粘贴标识符，例如 hello_world",
        ids::SLUGIFY => "粘贴标题，例如 Hello World",
        ids::NUMBER_BASE => "例如 255、0xff、0b1111",
        ids::TEXT_STATS => "粘贴要统计的文本",
        ids::ROMAN => "例如 1994 或 MCMXCIV",
        ids::PERCENTAGE => "两个数字，例如 25 200",
        ids::CHMOD => "例如 755 或 rwxr-xr-x",
        ids::BINARY => "文本，或二进制 / 十六进制",
        ids::COLOR => "#336699 或 rgb(51, 102, 153)",
        ids::BASIC_AUTH => "user:password",
        ids::HMAC => "第一行：密钥\n其余：消息",
        ids::OTP => "Base32 密钥；可选第二行验证码",
        ids::EMAIL_NORM => "name+tag@gmail.com",
        ids::LIST_CONV => "一行一项，或逗号分隔",
        ids::JSON_YAML | ids::JSON_TOML | ids::YAML_TOML => "粘贴 JSON / YAML / TOML",
        ids::SQL_FORMAT => "SELECT * FROM t WHERE …",
        ids::MARKDOWN_HTML => "# 标题\n\n**粗体**",
        ids::IPV4_SUBNET => "192.168.1.10/24",
        ids::IPV4_ADDR => "192.168.1.1 或 3232235777",
        ids::IPV4_RANGE => "192.168.1.0-192.168.1.255",
        ids::IPV6_ULA | ids::MAC_GEN | ids::RANDOM_PORT | ids::RSA_KEYGEN => "点击「生成」",
        ids::BIP39 => "可选：粘贴助记词进行校验",
        ids::QR_CODE => "要编码的文本或 URL",
        ids::WIFI_QR => "第1行 SSID\n第2行 WPA/WEP/nopass\n第3行密码",
        ids::SVG_PLACEHOLDER => "宽x高，例如 320x180\n可选文字",
        ids::MIME => "png 或 image/png",
        ids::HTTP_STATUS => "404 或 not found",
        ids::UA_PARSE => "粘贴 User-Agent 字符串",
        ids::IBAN => "GB82 WEST 1234 5698 7654 32",
        ids::TEMPERATURE => "25C / 77F / 298K",
        ids::ETA => "1.5GB 20MB/s",
        ids::NUMERONYM => "internationalization",
        ids::OBFUSCATE => "要遮罩的密钥或邮箱",
        ids::NATO => "SOS",
        ids::SAFELINK => "Outlook SafeLinks URL",
        ids::DOCKER_RUN => "docker run -d --name web -p 80:80 nginx",
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
        ids::BASE64 => base64::convert_all(input),
        ids::URL_ENCODE => url_encode::convert_all(input),
        ids::HTML_ESCAPE => html_escape::convert_all(input),
        ids::HASH => hash::convert_all(input),
        ids::CASE => case::convert_all(input),
        ids::SLUGIFY => slugify::convert_all(input),
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
        ids::BASIC_AUTH => basic_auth::convert_all(input),
        ids::HMAC => hmac_tool::convert_all(input),
        ids::OTP => otp_tool::convert_all(input),
        ids::EMAIL_NORM => email_norm::convert_all(input),
        ids::LIST_CONV => list_conv::convert_all(input),
        ids::JSON_YAML => data_format::json_yaml_all(input),
        ids::JSON_TOML => data_format::json_toml_all(input),
        ids::SQL_FORMAT => sql_format::convert_all(input),
        ids::MARKDOWN_HTML => markdown_html::convert_all(input),
        ids::IPV4_SUBNET => ipv4_tool::subnet_all(input),
        ids::IPV4_ADDR => ipv4_tool::addr_all(input),
        ids::IPV4_RANGE => ipv4_tool::range_all(input),
        ids::MIME => mime_tool::convert_all(input),
        ids::HTTP_STATUS => http_status::convert_all(input),
        ids::UA_PARSE => ua_parse::convert_all(input),
        ids::IBAN => iban_tool::convert_all(input),
        ids::TEMPERATURE => temperature::convert_all(input),
        ids::ETA => eta_tool::convert_all(input),
        ids::NUMERONYM => numeronym::convert_all(input),
        ids::OBFUSCATE => obfuscator::convert_all(input),
        ids::NATO => nato_alpha::convert_all(input),
        ids::SAFELINK => safelink::convert_all(input),
        ids::DOCKER_RUN => docker_compose::convert_all(input),
        ids::XML_TOOL => xml_tool::convert_all(input),
        ids::JSON_CSV => csv_tool::convert_all(input),
        ids::YAML_TOML => data_format::yaml_toml_all(input),
        ids::JSON_DIFF => json_diff_tool::convert_all(input),
        ids::PASSWORD_STRENGTH => password_strength::convert_all(input),
        ids::KEYCODE => keycode_info::convert_all(input),
        ids::PHONE => phone_tool::convert_all(input),
        ids::ASCII_DRAW => ascii_drawer::convert_all(input),
        ids::UNICODE_ESC => unicode_escape::convert_all(input),
        ids::QR_CODE => qr_tool::convert_all(input),
        ids::WIFI_QR => wifi_qr::convert_all(input),
        ids::META_TAGS => meta_tags::convert_all(input),
        ids::SVG_PLACEHOLDER => svg_placeholder::convert_all(input),
        ids::CRON_PRESETS => cron_presets::convert_all(input),
        ids::MATH_EVAL => math_eval::convert_all(input),
        ids::GIT_MEMO => git_memo::convert_all(input),
        ids::REGEX_MEMO => regex_memo::convert_all(input),
        ids::BASE64_HEX => base64_hex::convert_all(input),
        ids::BCRYPT => bcrypt_tool::convert_all(input),
        ids::ENCRYPT => encrypt_tool::convert_all(input),
        ids::BIP39 => bip39_tool::convert_all(input),
        _ => Err("未知批量工具".into()),
    }
}

fn run_batch_generate(
    tool_id: &str,
    bip39_words: usize,
    qty: usize,
) -> Result<Vec<(&'static str, String)>, String> {
    match tool_id {
        ids::IPV6_ULA => Ok(ipv6_ula::generate_many(qty)),
        ids::MAC_GEN => Ok(mac_gen::generate_many(true, true, qty)),
        ids::RANDOM_PORT => Ok(random_port::generate_many(qty)),
        ids::RSA_KEYGEN => rsa_tool::generate_many(2048, qty),
        ids::BIP39 => bip39_tool::generate_many(bip39_words, qty),
        _ => Err("此工具不支持生成".into()),
    }
}

fn input_pane_title(tool_id: &str) -> &'static str {
    match tool_id {
        ids::QR_CODE => "文本 / URL",
        ids::WIFI_QR => "SSID / 加密 / 密码",
        ids::SVG_PLACEHOLDER => "尺寸",
        ids::BASIC_AUTH => "账号",
        ids::OTP => "密钥",
        ids::BCRYPT | ids::ENCRYPT => "口令",
        ids::BIP39 => "助记词",
        ids::HMAC => "密钥 + 消息",
        _ => "输入",
    }
}

impl Render for BatchWorkbench {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = self.title();
        div()
            .id("batch-workbench")
            .on_action(cx.listener(|this, _: &PrimaryRun, _, cx| {
                if devtools_tools::is_batch_generate_tool(this.tool_id) {
                    this.generate_random(cx);
                } else {
                    this.live_refresh(cx);
                }
            }))
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
            .when(self.tool_id == ids::BIP39, |view| {
                view.child(
                    pane(theme)
                        .h(px(120.))
                        .flex_shrink_0()
                        .child(self.input_header(cx))
                        .child(
                            div()
                                .flex_1()
                                .min_h_0()
                                .min_w_0()
                                .px_3()
                                .pb_2()
                                .child(self.input.clone()),
                        ),
                )
            })
            .when(
                !devtools_tools::is_batch_generate_tool(self.tool_id)
                    && devtools_tools::is_batch_single_line_tool(self.tool_id),
                |view| {
                    // Top-bottom (it-tools date-converter): compact input, then
                    // labeled single-line rows — no big enclosing result pane.
                    view.child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .gap(px(theme::space::SM))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(theme::space::SM))
                                    .child(
                                        paste_button(theme, "batch-paste").on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(
                                                |this, _: &MouseDownEvent, window, cx| {
                                                    cx.stop_propagation();
                                                    this.paste_into_input(window, cx);
                                                },
                                            ),
                                        ),
                                    )
                                    .when(self.tool_id == ids::TIMESTAMP, |row| {
                                        row.child(
                                            primary_button(theme, "batch-now", "现在")
                                                .on_mouse_down(
                                                    MouseButton::Left,
                                                    cx.listener(
                                                        |this, _: &MouseDownEvent, window, cx| {
                                                            cx.stop_propagation();
                                                            this.seed_now(window, cx);
                                                        },
                                                    ),
                                                ),
                                        )
                                    })
                                    .child(button(theme, "batch-clear", "清空").on_click(
                                        cx.listener(|this, _, window, cx| {
                                            this.clear(window, cx)
                                        }),
                                    )),
                            )
                            .child(
                                div()
                                    .h(px(40.))
                                    .min_w_0()
                                    .w_full()
                                    .px(px(theme::space::SM))
                                    .rounded(px(theme::radius::MD))
                                    .bg(theme.panel)
                                    .border_1()
                                    .border_color(theme.border)
                                    .child(self.input.clone()),
                            ),
                    )
                    .child(self.single_line_results(cx))
                },
            )
            .when(
                !devtools_tools::is_batch_single_line_tool(self.tool_id),
                |view| {
                    view.child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .flex()
                            .gap(px(theme::space::MD))
                            .when(
                                !devtools_tools::is_batch_generate_tool(self.tool_id),
                                |inner| {
                                    inner.child(
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
                                },
                            )
                            .child(self.result_pane(cx)),
                    )
                },
            )
    }
}
