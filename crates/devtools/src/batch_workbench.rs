//! One-input → many labeled outputs (batch date-time layout).
//!
//! Live-updates for pure transforms. Generators are not hosted here.

use crate::{
    actions::PrimaryRun,
    text_field::{Edited, TextField},
    theme::{self, Theme},
    ui::{
        button, chip, color_swatch, empty_hint, extract_hex_color, looks_like_ascii_art,
        looks_like_svg_markup, multiline_mono, multiline_mono_art, pane,
        pane_header, pane_title, paste_button, primary_button, single_line, status_chip,
        svg_preview, workbench_title, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::{
    ascii_drawer,
    base64::{self, Alphabet as B64Alphabet, Base64Mode},
    base64_file::{self, Base64FileMode},
    base64_hex,
    base64_image::{self, Base64ImageMode},
    basic_auth, bcrypt_tool, binary_tool, bip39_tool, case, cert_decode, chmod_calc,
    color_palettes, color_tool, cron_presets, csv_tool,
    data_format::{self, FormatKind},
    docker_compose, email_norm,
    encrypt_tool::{self, EncryptMode},
    eta_tool, git_memo,
    hash::{self, HashEncoding},
    hmac_tool,
    html_escape::{self, HtmlMode},
    http_status, iban_tool, ipv4_tool, ipv6_ula, json_diff_tool,
    keycode_info, list_conv, mac_gen, markdown_html, math_eval, meta_tags, mime_tool, nato_alpha,
    number_base,
    number_base::Base,
    numeronym, obfuscator, otp_tool, password_strength, percentage, phone_tool, quantity, qr_tool,
    random_port, regex_memo, roman, rsa_tool, safelink, slugify,
    sql_format::{self, SqlMode},
    ruler_tool,
    svg_edit::{self, SvgMode},
    svg_placeholder,
    tailwind_colors,
    temperature, text_stats, timestamp, ua_parse,
    unicode_escape::{self, UnicodeMode},
    url_encode::{self, UrlMode},
    wifi_qr,
    xml_tool::{self, XmlMode},
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
    /// `None` = auto-detect for 数据格式.
    data_in: Option<FormatKind>,
    data_out: FormatKind,
    sql_mode: SqlMode,
    url_mode: UrlMode,
    url_mode_manual: bool,
    html_mode: HtmlMode,
    html_mode_manual: bool,
    b64_mode: Base64Mode,
    b64_mode_manual: bool,
    b64_alphabet: B64Alphabet,
    xml_mode: XmlMode,
    unicode_mode: UnicodeMode,
    unicode_mode_manual: bool,
    encrypt_mode: EncryptMode,
    hash_encoding: HashEncoding,
    b64_file_mode: Base64FileMode,
    b64_file_mode_manual: bool,
    b64_image_mode: Base64ImageMode,
    b64_image_mode_manual: bool,
    svg_mode: SvgMode,
    /// Separate passphrase for Encrypt (and shown only then).
    password: Entity<TextField>,
    result_scroll: ScrollHandle,
}

impl BatchWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextField::new(false, false, "在这里粘贴输入", cx));
        let password = cx.new(|cx| TextField::new(false, false, "口令", cx));
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
        cx.subscribe(&password, |this, _, _: &Edited, cx| {
            this.notice = "";
            if this.tool_id == ids::ENCRYPT {
                this.live_refresh(cx);
            } else {
                cx.notify();
            }
        })
        .detach();
        Self {
            tool_id: ids::TIMESTAMP,
            input,
            password,
            rows: Vec::new(),
            error: None,
            notice: "",
            num_from: Base::DEC,
            bip39_words: 12,
            quantity: 1,
            data_in: None,
            data_out: FormatKind::default_output(),
            sql_mode: SqlMode::default_mode(),
            url_mode: UrlMode::default_mode(),
            url_mode_manual: false,
            html_mode: HtmlMode::default_mode(),
            html_mode_manual: false,
            b64_mode: Base64Mode::default_mode(),
            b64_mode_manual: false,
            b64_alphabet: B64Alphabet::Standard,
            xml_mode: XmlMode::default_mode(),
            unicode_mode: UnicodeMode::default_mode(),
            unicode_mode_manual: false,
            encrypt_mode: EncryptMode::default_mode(),
            hash_encoding: HashEncoding::default_encoding(),
            b64_file_mode: Base64FileMode::default_mode(),
            b64_file_mode_manual: false,
            b64_image_mode: Base64ImageMode::default_mode(),
            b64_image_mode_manual: false,
            svg_mode: SvgMode::default_mode(),
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
            self.data_in = None;
            self.data_out = FormatKind::default_output();
            self.sql_mode = SqlMode::default_mode();
            self.url_mode = UrlMode::default_mode();
            self.url_mode_manual = false;
            self.html_mode = HtmlMode::default_mode();
            self.html_mode_manual = false;
            self.b64_mode = Base64Mode::default_mode();
            self.b64_mode_manual = false;
            self.b64_alphabet = B64Alphabet::Standard;
            self.xml_mode = XmlMode::default_mode();
            self.unicode_mode = UnicodeMode::default_mode();
            self.unicode_mode_manual = false;
            self.encrypt_mode = EncryptMode::default_mode();
            self.hash_encoding = HashEncoding::default_encoding();
            self.b64_file_mode = Base64FileMode::default_mode();
            self.b64_file_mode_manual = false;
            self.b64_image_mode = Base64ImageMode::default_mode();
            self.b64_image_mode_manual = false;
            self.svg_mode = SvgMode::default_mode();
            self.password.update(cx, |field, cx| {
                field.clear(cx);
                field.set_placeholder("口令", cx);
                field.set_multiline(false, cx);
            });
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
            if self.tool_id == ids::ENCRYPT {
                self.password.update(cx, |field, cx| field.clear(cx));
            }
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
        if self.tool_id == ids::URL_ENCODE && !self.url_mode_manual {
            self.url_mode = url_encode::suggest_mode(&input);
        } else if self.tool_id == ids::HTML_ESCAPE && !self.html_mode_manual {
            self.html_mode = html_escape::suggest_mode(&input);
        } else if self.tool_id == ids::BASE64 && !self.b64_mode_manual {
            self.b64_mode = base64::suggest_mode(&input);
        } else if self.tool_id == ids::UNICODE_ESC && !self.unicode_mode_manual {
            self.unicode_mode = unicode_escape::suggest_mode(&input);
        } else if self.tool_id == ids::BASE64_FILE && !self.b64_file_mode_manual {
            self.b64_file_mode = base64_file::suggest_mode(&input);
        } else if self.tool_id == ids::BASE64_IMAGE && !self.b64_image_mode_manual {
            self.b64_image_mode = base64_image::suggest_mode(&input);
        }
        let password = self.password.read(cx).buffer.text().to_owned();
        match run_batch(
            self.tool_id,
            &input,
            BatchRunOpts {
                num_from: self.num_from,
                data_in: self.data_in,
                data_out: self.data_out,
                sql_mode: self.sql_mode,
                url_mode: self.url_mode,
                html_mode: self.html_mode,
                b64_mode: self.b64_mode,
                b64_alphabet: self.b64_alphabet,
                xml_mode: self.xml_mode,
                unicode_mode: self.unicode_mode,
                encrypt_mode: self.encrypt_mode,
                hash_encoding: self.hash_encoding,
                b64_file_mode: self.b64_file_mode,
                b64_image_mode: self.b64_image_mode,
                svg_mode: self.svg_mode,
                password,
            },
        ) {
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
            ids::BASE64 => ("Base64", "编码 / 解码切换 · 像 Base64 时默认解码"),
            ids::URL_ENCODE => ("URL Encode", "编码 / 解码切换 · 含 % 时默认解码"),
            ids::HTML_ESCAPE => ("HTML Escape", "转义 / 反转义切换 · 含实体时默认反转义"),
            ids::HASH => ("Hasher", "MD5/SHA/SHA3/Keccak/RIPEMD · Hex/HEX/Base64/Bin"),
            ids::CASE => ("Case", "snake / camel / Pascal / kebab / CONSTANT / Title"),
            ids::SLUGIFY => ("Slugify", "Slug 与规范化变体"),
            ids::NUMBER_BASE => ("进制转换", "2–36 · 预设含 32/36 · 可用 N#数字"),
            ids::TEXT_STATS => ("文本统计", "字符 · 词 · 行 · 字节"),
            ids::ROMAN => ("罗马数字", "1–3999 · 自动识别方向"),
            ids::PERCENTAGE => ("百分比", "输入两个数字，例如 25 200"),
            ids::CHMOD => ("Chmod", "755 ↔ rwxr-xr-x"),
            ids::BINARY => ("文本 / 二进制", "二进制 · 十六进制 · 码点"),
            ids::COLOR => ("Color Inspector", "#hex · rgb · hsl · hsv · cmyk"),
            ids::COLOR_PALETTES => ("Color Palettes", "互补 / 类似 / 三元 / 单色阶"),
            ids::RULER => ("Ruler", "px / rem / pt / cm · 非屏幕叠尺"),
            ids::SVG_EDIT => ("SVG Editor", "美化 / 压缩 · 可预览"),
            ids::TAILWIND_COLORS => ("Tailwind Colors", "色板搜索 · 最近色"),
            ids::BASE64_IMAGE => ("Base64 Image", "路径 ↔ data URL · 离线"),
            ids::BASIC_AUTH => ("Basic Auth", "user:password → Authorization"),
            ids::HMAC => ("HMAC", "第一行密钥，其余为消息"),
            ids::OTP => ("OTP / TOTP", "Base32 密钥 · 可选第二行验证码"),
            ids::EMAIL_NORM => ("邮箱规范化", "小写 · Gmail 点号 · +tag"),
            ids::LIST_CONV => ("列表转换", "排序 · 反转 · 去重 · 连接"),
            ids::DATA_FORMAT => ("数据格式", "自动识别 JSON/YAML/TOML/XML · 可选输入/输出类型"),
                        ids::SQL_FORMAT => ("SQL 格式化", "美化 / 压缩切换 · 默认美化"),
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
                        ids::JSON_CSV => ("JSON ↔ CSV", "对象数组"),
                        ids::JSON_DIFF => ("JSON diff", "用 --- 分隔左右"),
            ids::PASSWORD_STRENGTH => ("口令强度", "离线启发式"),
            ids::KEYCODE => ("按键码", "名称 · keyCode · code"),
            ids::PHONE => ("电话整理", "E.164 猜测"),
            ids::ASCII_DRAW => ("ASCII 横幅", "极简字模"),
            ids::RANDOM_PORT => ("随机端口", "点击生成 · 可设数量"),
            ids::UNICODE_ESC => ("Unicode 转义", "转义 / 反转义切换"),
            ids::QR_CODE => ("QR 码", "SVG 预览"),
            ids::WIFI_QR => ("Wi‑Fi QR", "SSID / 加密 / 密码 · SVG 预览"),
            ids::META_TAGS => ("Meta 标签", "OG / Twitter"),
            ids::SVG_PLACEHOLDER => ("SVG 占位图", "宽x高"),
            ids::CRON_PRESETS => ("Cron 模板", "常用表达式 · 字段说明"),
            ids::MATH_EVAL => ("数学求值", "安全表达式"),
            ids::GIT_MEMO => ("Git 速查", "关键词过滤"),
            ids::REGEX_MEMO => ("正则速查", "与测试工具分开"),
            ids::BASE64_HEX => ("Base64 ↔ Hex", "粘贴载荷"),
            ids::BCRYPT => ("bcrypt", "哈希 / 校验"),
            ids::ENCRYPT => ("AES 加解密", "加密 / 解密 · 独立口令 · 学习用"),
            ids::RSA_KEYGEN => ("RSA 密钥", "点击生成 PEM · 数量≤5"),
            ids::BIP39 => ("BIP39", "生成助记词 · 可粘贴校验 · 非钱包"),
            ids::XML_TOOL => ("XML", "美化 / 压缩 / ↔ JSON"),
            ids::CERT_DECODE => ("证书解码", "PEM / X.509 · 不校验证书链"),
            ids::BASE64_FILE => ("Base64 文件", "路径 ↔ Base64 · 粘贴路径"),
            _ => ("工具", "离线"),
        }
    }

    fn options_bar(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let mut row = div().flex().flex_wrap().items_center().gap_2();
        if self.tool_id == ids::URL_ENCODE {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in UrlMode::ALL {
                let active = self.url_mode == mode;
                let id = format!("url-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.url_mode = mode;
                        this.url_mode_manual = true;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::HTML_ESCAPE {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in HtmlMode::ALL {
                let active = self.html_mode == mode;
                let id = format!("html-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.html_mode = mode;
                        this.html_mode_manual = true;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::BASE64 {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in Base64Mode::ALL {
                let active = self.b64_mode == mode;
                let id = format!("b64-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.b64_mode = mode;
                        this.b64_mode_manual = true;
                        this.live_refresh(cx);
                    },
                )));
            }
            row = row.child(div().text_xs().text_color(theme.muted).child("字母表"));
            for alpha in B64Alphabet::ALL {
                let active = self.b64_alphabet == alpha;
                let id = format!("b64-alpha-{}", alpha.label());
                row = row.child(chip(theme, id, alpha.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.b64_alphabet = alpha;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::SQL_FORMAT {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in SqlMode::ALL {
                let active = self.sql_mode == mode;
                let id = format!("sql-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.sql_mode = mode;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::XML_TOOL {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in XmlMode::ALL {
                let active = self.xml_mode == mode;
                let id = format!("xml-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.xml_mode = mode;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::UNICODE_ESC {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in UnicodeMode::ALL {
                let active = self.unicode_mode == mode;
                let id = format!("uni-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.unicode_mode = mode;
                        this.unicode_mode_manual = true;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::ENCRYPT {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in EncryptMode::ALL {
                let active = self.encrypt_mode == mode;
                let id = format!("enc-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.encrypt_mode = mode;
                        this.live_refresh(cx);
                    },
                )));
            }
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("口令在下方独立栏 · 仅供学习"),
            );
        } else if self.tool_id == ids::HASH {
            row = row.child(div().text_xs().text_color(theme.muted).child("编码"));
            for enc in HashEncoding::ALL {
                let active = self.hash_encoding == enc;
                let id = format!("hash-enc-{}", enc.label());
                row = row.child(chip(theme, id, enc.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.hash_encoding = enc;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::BASE64_FILE {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in Base64FileMode::ALL {
                let active = self.b64_file_mode == mode;
                let id = format!("b64file-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.b64_file_mode = mode;
                        this.b64_file_mode_manual = true;
                        this.live_refresh(cx);
                    },
                )));
            }
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("拖入文件或粘贴路径 · 解码可「路径\\nBase64」写出"),
            );
        } else if self.tool_id == ids::BASE64_IMAGE {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in Base64ImageMode::ALL {
                let active = self.b64_image_mode == mode;
                let id = format!("b64img-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.b64_image_mode = mode;
                        this.b64_image_mode_manual = true;
                        this.live_refresh(cx);
                    },
                )));
            }
            row = row.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("支持拖拽图片文件"),
            );
        } else if self.tool_id == ids::SVG_EDIT {
            row = row.child(div().text_xs().text_color(theme.muted).child("模式"));
            for mode in SvgMode::ALL {
                let active = self.svg_mode == mode;
                let id = format!("svgedit-mode-{}", mode.label());
                row = row.child(chip(theme, id, mode.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.svg_mode = mode;
                        this.live_refresh(cx);
                    },
                )));
            }
        } else if self.tool_id == ids::DATA_FORMAT {
            row = row.child(div().text_xs().text_color(theme.muted).child("输入"));
            let auto_active = self.data_in.is_none();
            row = row.child(chip(theme, "data-in-auto", "自动", auto_active).on_click(
                cx.listener(|this, _, _, cx| {
                    this.data_in = None;
                    this.live_refresh(cx);
                }),
            ));
            for kind in FormatKind::ALL {
                let active = self.data_in == Some(kind);
                let id = format!("data-in-{}", kind.label());
                row = row.child(chip(theme, id, kind.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.data_in = Some(kind);
                        this.live_refresh(cx);
                    },
                )));
            }
            row = row.child(div().text_xs().text_color(theme.muted).child("输出"));
            for kind in FormatKind::ALL {
                let active = self.data_out == kind;
                let id = format!("data-out-{}", kind.label());
                row = row.child(chip(theme, id, kind.label(), active).on_click(cx.listener(
                    move |this, _, _, cx| {
                        this.data_out = kind;
                        this.live_refresh(cx);
                    },
                )));
            }
            if let Some(detected) = {
                let input = self.input.read(cx).buffer.text().to_owned();
                data_format::detect(&input)
            } {
                row = row.child(
                    div()
                        .text_xs()
                        .text_color(theme.muted)
                        .child(format!("识别为 {}", detected.label())),
                );
            }
        } else if self.tool_id == ids::NUMBER_BASE {
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
                    .child("也可 0x/0b/0o 或 N#数字（2–36）"),
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
                    primary_button(theme, "batch-generate", "生成").on_click(
                        cx.listener(|this, _, _, cx| this.generate_random(cx)),
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
                    primary_button(theme, "batch-generate", "生成").on_click(
                        cx.listener(|this, _, _, cx| this.generate_random(cx)),
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
                    primary_button(theme, "batch-generate", "生成").on_click(
                        cx.listener(|this, _, _, cx| this.generate_random(cx)),
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


    /// Full-pane document output (SQL 美化/压缩): header + 复制, body fills the pane.
    fn full_document_result_pane(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let value = self.rows.first().map(|(_, v)| v.as_str()).unwrap_or("");
        let copy = value.to_owned();
        let mut header = pane_header(theme).child(pane_title(theme, "结果"));
        if !value.is_empty() {
            header = header.child(copy_chip("batch-full-copy".into(), copy, cx));
        }
        let mut pane_view = pane(theme).flex_1().child(header);
        if self.rows.is_empty() && self.error.is_none() {
            pane_view = pane_view.child(empty_hint(theme, "输入后显示结果"));
        } else if !value.is_empty() {
            pane_view = pane_view.child(
                div()
                    .id("batch-full-result")
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.result_scroll)
                    .p_3()
                    .child(multiline_mono(theme, value)),
            );
        }
        pane_view
    }

    fn is_color_tool(tool_id: &str) -> bool {
        matches!(
            tool_id,
            ids::COLOR | ids::COLOR_PALETTES | ids::TAILWIND_COLORS
        )
    }

    fn is_file_drop_tool(tool_id: &str) -> bool {
        matches!(tool_id, ids::BASE64_IMAGE | ids::BASE64_FILE)
    }

    fn apply_dropped_paths(
        &mut self,
        paths: &ExternalPaths,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(path) = paths.paths().first() else {
            self.notice = "未收到文件路径";
            cx.notify();
            return;
        };
        let text = path.to_string_lossy().to_string();
        self.input.update(cx, |field, cx| {
            field.set_input(&text, cx);
            field.focus(window);
        });
        self.notice = "已拖入文件";
        if self.tool_id == ids::BASE64_IMAGE {
            self.b64_image_mode = Base64ImageMode::Encode;
            self.b64_image_mode_manual = true;
        } else if self.tool_id == ids::BASE64_FILE {
            self.b64_file_mode = Base64FileMode::Encode;
            self.b64_file_mode_manual = true;
        }
        self.live_refresh(cx);
    }

    fn apply_color_hex(&mut self, hex: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |field, cx| {
            field.set_input(hex, cx);
            field.focus(window);
        });
        self.notice = "";
        self.live_refresh(cx);
    }

    /// Live color preview with hue strip and lightness chips for quick pick.
    fn color_picker_panel(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let theme = Theme::get(cx);
        let current = self.input.read(cx).buffer.text().to_owned();
        let preview_hex = color_tool::parse_color(&current)
            .ok()
            .map(color_tool::to_hex)
            .or_else(|| extract_hex_color(&current))
            .unwrap_or_else(|| "#3366CC".into());
        let hues = color_tool::hue_wheel(24, 0.85, 0.5);
        let lights = [
            ("亮", 0.85_f64),
            ("浅", 0.7),
            ("中", 0.5),
            ("深", 0.32),
            ("暗", 0.18),
        ];

        let mut hue_row = div()
            .id("color-hue-strip")
            .flex()
            .flex_wrap()
            .gap(px(2.))
            .w_full();
        for (i, hex) in hues.into_iter().enumerate() {
            let pick = hex.clone();
            hue_row = hue_row.child(
                div()
                    .id(SharedString::from(format!("hue-{i}")))
                    .cursor_pointer()
                    .w(px(18.))
                    .h(px(18.))
                    .rounded(px(theme::radius::SM))
                    .bg({
                        let s = hex.trim_start_matches('#');
                        let n = u32::from_str_radix(s, 16).unwrap_or(0);
                        rgb(n)
                    })
                    .border_1()
                    .border_color(theme.border_subtle)
                    .hover(|s| s.border_color(theme.accent))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.apply_color_hex(&pick, window, cx);
                    })),
            );
        }

        let mut light_row = div()
            .id("color-light-strip")
            .flex()
            .items_center()
            .gap(px(theme::space::SM))
            .flex_wrap();
        // Derive hue of current preview for lightness chips
        let base_h = color_tool::parse_color(&preview_hex)
            .ok()
            .and_then(|rgb| {
                // convert_all includes HSL — parse degrees from "hsl(N, …)"
                color_tool::convert_all(&color_tool::to_hex(rgb))
                    .ok()
                    .and_then(|rows| {
                        rows.into_iter()
                            .find(|(k, _)| *k == "HSL")
                            .and_then(|(_, v)| {
                                v.trim()
                                    .trim_start_matches("hsl(")
                                    .split(',')
                                    .next()
                                    .and_then(|s| s.trim().parse::<f64>().ok())
                            })
                    })
            })
            .unwrap_or(210.0);
        for (i, (label, l)) in lights.into_iter().enumerate() {
            let hex = color_tool::hsl_hex(base_h, 0.85, l);
            let pick = hex.clone();
            light_row = light_row.child(
                div()
                    .id(SharedString::from(format!("light-{i}")))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .px(px(theme::space::SM))
                    .h(px(28.))
                    .rounded(px(theme::radius::MD))
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.elevated)
                    .hover(|s| s.bg(theme.selected))
                    .child(color_swatch(&hex, 14.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.muted)
                            .child(label),
                    )
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.apply_color_hex(&pick, window, cx);
                    })),
            );
        }

        // Common quick-palette presets
        let presets = [
            "#EF4444", "#F97316", "#EAB308", "#22C55E", "#14B8A6", "#3B82F6", "#6366F1",
            "#A855F7", "#EC4899", "#64748B", "#0F172A", "#F8FAFC",
        ];
        let mut preset_row = div()
            .id("color-presets")
            .flex()
            .flex_wrap()
            .gap(px(theme::space::SM));
        for (i, hex) in presets.into_iter().enumerate() {
            let pick = hex.to_string();
            preset_row = preset_row.child(
                div()
                    .id(SharedString::from(format!("preset-{i}")))
                    .cursor_pointer()
                    .child(color_swatch(hex, 22.))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.apply_color_hex(&pick, window, cx);
                    })),
            );
        }

        div()
            .id("color-picker")
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(theme::space::SM))
            .p(px(theme::space::MD))
            .rounded(px(theme::radius::MD))
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme::space::MD))
                    .child(color_swatch(&preview_hex, 56.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text)
                                    .child("实时预览"),
                            )
                            .child(
                                div()
                                    .font(crate::theme::monospace())
                                    .text_sm()
                                    .text_color(theme.muted)
                                    .child(single_line(preview_hex.as_str())),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.faint)
                                    .child("点色盘 / 明暗 / 预设快速填入"),
                            ),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted)
                    .child("色相"),
            )
            .child(hue_row)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted)
                    .child("明度"),
            )
            .child(light_row)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.muted)
                    .child("预设"),
            )
            .child(preset_row)
    }

    fn file_drop_zone(
        &self,
        theme: Theme,
        cx: &mut Context<Self>,
        inner: Div,
    ) -> Stateful<Div> {
        let accent = theme.accent;
        let accent_muted = theme.accent_muted;
        div()
            .id("file-drop-zone")
            .flex_1()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .rounded(px(theme::radius::MD))
            .border_2()
            .border_color(theme.border)
            .border_dashed()
            .can_drop(|data, _, _| data.is::<ExternalPaths>())
            .drag_over::<ExternalPaths>(move |style, _, _, _| {
                style.bg(accent_muted).border_color(accent)
            })
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                this.apply_dropped_paths(paths, window, cx);
            }))
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(theme::space::MD))
                    .pt(px(theme::space::SM))
                    .text_xs()
                    .text_color(theme.muted)
                    .child("拖拽文件到此处，或粘贴路径"),
            )
            .child(inner)
    }

    /// Tools whose single mode result should fill the whole result pane
    /// (no nested label | mini-box | 复制 row). Mode chips live in the options bar.
    fn uses_full_result_pane(tool_id: &str, row_count: usize) -> bool {
        // Mode chips select one document result — fill the pane like input (no nested row).
        // Only when the selected mode actually produced a single row (Base64 encode used
        // to emit two alphabets; Unicode escape still emits several styles).
        row_count <= 1
            && matches!(
                tool_id,
                ids::SQL_FORMAT
                    | ids::DATA_FORMAT
                    | ids::URL_ENCODE
                    | ids::HTML_ESCAPE
                    | ids::BASE64
                    | ids::XML_TOOL
                    | ids::MARKDOWN_HTML
                    | ids::UNICODE_ESC
                    | ids::SVG_EDIT
            )
    }

    /// Multi-line / generate tools: values may wrap; enclosed in a pane.
    fn result_pane(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        // SQL (and similar): one document fills the pane — mirror the input side.
        if Self::uses_full_result_pane(self.tool_id, self.rows.len()) {
            return self.full_document_result_pane(cx);
        }
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
                    let body = if looks_like_svg_markup(label, value) {
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(theme::space::SM))
                            .child(
                                div()
                                    .p(px(theme::space::SM))
                                    .rounded(px(theme::radius::SM))
                                    .bg(gpui::rgb(0xffffff))
                                    .border_1()
                                    .border_color(theme.border_subtle)
                                    .child(svg_preview(value, 200.)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .child(single_line("预览 · 点复制取 SVG 源码")),
                            )
                            .into_any_element()
                    } else if looks_like_ascii_art(label, value) {
                        multiline_mono_art(theme, value.as_str()).into_any_element()
                    } else {
                        multiline_mono(theme, value.as_str()).into_any_element()
                    };
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
                                .child(single_line(*label)),
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
                                .child(body),
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

    /// Compact batch date-time layout: no enclosing result pane;
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
                let is_preview = *label == "预览" || *label == "种子";
                let swatch_hex = extract_hex_color(value);
                let row_h = if is_preview { 48. } else { 36. };
                let swatch_side = if is_preview { 32. } else { 20. };
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
                            .h(px(row_h))
                            .px(px(theme::space::SM))
                            .flex()
                            .items_center()
                            .gap(px(theme::space::SM))
                            .rounded(px(theme::radius::SM))
                            .bg(theme.elevated)
                            .border_1()
                            .border_color(theme.border_subtle)
                            .when_some(swatch_hex.as_ref(), |d, hex| {
                                d.child(color_swatch(hex, swatch_side))
                            })
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .font(crate::theme::monospace())
                                    .text_sm()
                                    .text_color(theme.text)
                                    .child(single_line(value.as_str())),
                            ),
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
        ids::COLOR => "#336699 / rgb() / hsl()，或点下方色盘",
        ids::BASIC_AUTH => "user:password",
        ids::HMAC => "第一行：密钥 · 其余：消息",
        ids::OTP => "Base32 密钥；可选第二行验证码",
        ids::EMAIL_NORM => "name+tag@gmail.com",
        ids::LIST_CONV => "一行一项，或逗号分隔",
        ids::DATA_FORMAT => "粘贴 JSON / YAML / TOML / XML",
        ids::SQL_FORMAT => "SELECT * FROM t WHERE …",
        ids::MARKDOWN_HTML => "# 标题  **粗体**",
        ids::IPV4_SUBNET => "192.168.1.10/24",
        ids::IPV4_ADDR => "192.168.1.1 或 3232235777",
        ids::IPV4_RANGE => "192.168.1.0-192.168.1.255",
        ids::IPV6_ULA | ids::MAC_GEN | ids::RANDOM_PORT | ids::RSA_KEYGEN => "点击「生成」",
        ids::BIP39 => "可选：粘贴助记词进行校验",
        ids::QR_CODE => "要编码的文本或 URL",
        ids::WIFI_QR => "第1行 SSID · 第2行 WPA/WEP/nopass · 第3行密码",
        ids::SVG_PLACEHOLDER => "宽x高，例如 320x180 · 可选第二行文字",
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
        ids::XML_TOOL => "<root><item>1</item></root>",
        ids::CERT_DECODE => "-----BEGIN CERTIFICATE----- …",
        ids::BASE64_FILE => "拖入文件，或粘贴路径 / Base64 正文",
        ids::BASE64_IMAGE => "拖入图片，或粘贴路径 / data:image/…;base64,…",
        ids::COLOR_PALETTES => "#3366cc，或点下方色盘",
        ids::RULER => "16px 或 1.5rem 或 12pt",
        ids::SVG_EDIT => "<svg>…</svg>",
        ids::TAILWIND_COLORS => "blue-500 / #3b82f6 / blue，或点色盘",
        ids::ENCRYPT => "明文或 Base64 密文",
        _ => "在这里粘贴输入",
    }
}

struct BatchRunOpts {
    num_from: Base,
    data_in: Option<FormatKind>,
    data_out: FormatKind,
    sql_mode: SqlMode,
    url_mode: UrlMode,
    html_mode: HtmlMode,
    b64_mode: Base64Mode,
    b64_alphabet: B64Alphabet,
    xml_mode: XmlMode,
    unicode_mode: UnicodeMode,
    encrypt_mode: EncryptMode,
    hash_encoding: HashEncoding,
    b64_file_mode: Base64FileMode,
    b64_image_mode: Base64ImageMode,
    svg_mode: SvgMode,
    password: String,
}

fn run_batch(
    tool_id: &str,
    input: &str,
    opts: BatchRunOpts,
) -> Result<Vec<(&'static str, String)>, String> {
    let BatchRunOpts {
        num_from,
        data_in,
        data_out,
        sql_mode,
        url_mode,
        html_mode,
        b64_mode,
        b64_alphabet,
        xml_mode,
        unicode_mode,
        encrypt_mode,
        hash_encoding,
        b64_file_mode,
        b64_image_mode,
        svg_mode,
        password,
    } = opts;
    match tool_id {
        ids::TIMESTAMP => timestamp::convert_all(input)
            .map(|rows| {
                rows.into_iter()
                    .map(|r| (r.label, r.value))
                    .collect()
            })
            .map_err(|e| e.to_string()),
        ids::BASE64 => base64::convert_mode(input, b64_mode, b64_alphabet),
        ids::URL_ENCODE => url_encode::convert_mode(input, url_mode),
        ids::HTML_ESCAPE => html_escape::convert_mode(input, html_mode),
        ids::HASH => hash::convert_with_encoding(input, hash_encoding),
        ids::CASE => case::convert_all(input),
        ids::SLUGIFY => slugify::convert_all(input),
        ids::NUMBER_BASE => {
            let (detected, normalized) = number_base::normalize_input(input);
            let lower = input.trim().to_ascii_lowercase();
            let from = if number_base::parse_custom_prefix(input).is_some()
                || lower.starts_with("0x")
                || lower.starts_with("0b")
                || lower.starts_with("0o")
                || lower.starts_with("-0x")
                || lower.starts_with("-0b")
                || lower.starts_with("-0o")
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
        ids::DATA_FORMAT => data_format::convert_row(input, data_in, data_out),
        ids::SQL_FORMAT => sql_format::convert_mode(input, sql_mode),
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
                ids::JSON_CSV => csv_tool::convert_all(input),
                ids::JSON_DIFF => json_diff_tool::convert_all(input),
        ids::PASSWORD_STRENGTH => password_strength::convert_all(input),
        ids::KEYCODE => keycode_info::convert_all(input),
        ids::PHONE => phone_tool::convert_all(input),
        ids::ASCII_DRAW => ascii_drawer::convert_all(input),
        ids::UNICODE_ESC => unicode_escape::convert_mode(input, unicode_mode),
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
        ids::ENCRYPT => {
            let pass = if password.trim().is_empty() {
                // fall back to legacy two-line input
                return encrypt_tool::convert_all(input);
            } else {
                password.as_str()
            };
            encrypt_tool::convert_mode(input, pass, encrypt_mode)
        }
        ids::BIP39 => bip39_tool::convert_all(input),
        ids::XML_TOOL => xml_tool::convert_mode(input, xml_mode),
        ids::CERT_DECODE => cert_decode::convert_all(input),
        ids::BASE64_FILE => base64_file::convert_mode(input, b64_file_mode),
        ids::BASE64_IMAGE => base64_image::convert_mode(input, b64_image_mode),
        ids::COLOR_PALETTES => color_palettes::convert_all(input),
        ids::RULER => ruler_tool::convert_all(input),
        ids::SVG_EDIT => svg_edit::convert_mode(input, svg_mode),
        ids::TAILWIND_COLORS => tailwind_colors::convert_all(input),
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
        ids::BCRYPT => "口令",
        ids::ENCRYPT => "正文",
        ids::CERT_DECODE => "PEM",
        ids::BASE64_FILE => "路径 / Base64",
        ids::XML_TOOL => "XML / JSON",
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
                    // Top-bottom batch layout: compact input, then
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
                    .when(Self::is_color_tool(self.tool_id), |view| {
                        view.child(self.color_picker_panel(cx))
                    })
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
                                            .when(self.tool_id == ids::ENCRYPT, |p| {
                                                p.child(
                                                    pane_header(theme)
                                                        .child(pane_title(theme, "口令"))
                                                        .child(
                                                            paste_button(theme, "batch-pass-paste")
                                                                .on_mouse_down(
                                                                    MouseButton::Left,
                                                                    cx.listener(
                                                                        |this,
                                                                         _: &MouseDownEvent,
                                                                         window,
                                                                         cx| {
                                                                            cx.stop_propagation();
                                                                            let text = cx
                                                                                .read_from_clipboard()
                                                                                .and_then(|item| {
                                                                                    item.text()
                                                                                })
                                                                                .unwrap_or_default();
                                                                            this.password.update(
                                                                                cx,
                                                                                |field, cx| {
                                                                                    field.set_input(
                                                                                        &text, cx,
                                                                                    );
                                                                                    field.focus(
                                                                                        window,
                                                                                    );
                                                                                },
                                                                            );
                                                                            this.notice =
                                                                                if text.is_empty() {
                                                                                    "剪贴板为空"
                                                                                } else {
                                                                                    "已粘贴口令"
                                                                                };
                                                                            this.live_refresh(cx);
                                                                        },
                                                                    ),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .h(px(40.))
                                                        .flex_shrink_0()
                                                        .min_w_0()
                                                        .px_3()
                                                        .pb_1()
                                                        .child(self.password.clone()),
                                                )
                                            })
                                            .child(self.input_header(cx))
                                            .when(Self::is_file_drop_tool(self.tool_id), |p| {
                                                let inner = div()
                                                    .flex_1()
                                                    .min_h_0()
                                                    .min_w_0()
                                                    .p_3()
                                                    .child(self.input.clone());
                                                p.child(self.file_drop_zone(theme, cx, inner))
                                            })
                                            .when(!Self::is_file_drop_tool(self.tool_id), |p| {
                                                p.child(
                                                    div()
                                                        .flex_1()
                                                        .min_h_0()
                                                        .min_w_0()
                                                        .p_3()
                                                        .child(self.input.clone()),
                                                )
                                            }),
                                    )
                                },
                            )
                            .child(self.result_pane(cx)),
                    )
                },
            )
    }
}
