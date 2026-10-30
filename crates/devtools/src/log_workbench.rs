//! Log Viewer workbench: multi-file drop, stream scan, structured highlight.

use crate::{
    text_field::{Edited, TextField},
    theme::{self, Theme},
    ui::{
        banner, button, chip, empty_hint, pane, pane_header, pane_title, primary_button,
        single_line, status_chip, workbench_title, StatusKind,
    },
};
use devtools_core::ids;
use devtools_tools::log_viewer::{
    self, format_row_time, format_size, highlight_segments, row_copy_text, LogLevel, LogQuery,
    LogRow, LogSource, ScanResult, MAX_FILES, MAX_SHOWN_ROWS,
};
use gpui::{prelude::FluentBuilder, *};
use std::path::PathBuf;

pub struct LogWorkbench {
    sources: Vec<LogSource>,
    search: Entity<TextField>,
    from_time: Entity<TextField>,
    to_time: Entity<TextField>,
    use_regex: bool,
    /// Empty = all levels.
    levels: Vec<LogLevel>,
    result: ScanResult,
    notice: String,
    error: Option<String>,
    result_scroll: ScrollHandle,
}

impl LogWorkbench {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextField::new(false, false, "搜索（子串或正则）…", cx));
        let from_time = cx.new(|cx| TextField::new(false, false, "起始时间（可选）", cx));
        let to_time = cx.new(|cx| TextField::new(false, false, "结束时间（可选）", cx));
        for field in [&search, &from_time, &to_time] {
            cx.subscribe(field, |this, _, _: &Edited, cx| {
                this.notice.clear();
                cx.notify();
            })
            .detach();
        }
        Self {
            sources: Vec::new(),
            search,
            from_time,
            to_time,
            use_regex: false,
            levels: Vec::new(),
            result: ScanResult::default(),
            notice: String::new(),
            error: None,
            result_scroll: ScrollHandle::new(),
        }
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        self.search.read(cx).focus(window);
    }

    pub fn set_tool(&mut self, _tool_id: &'static str, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_input(window, cx);
        cx.notify();
    }

    /// Paste / insert: treat each non-empty line as a filesystem path.
    pub fn set_input_text(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        let paths: Vec<PathBuf> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(PathBuf::from)
            .collect();
        if paths.is_empty() {
            self.notice = "未识别到路径".into();
            cx.notify();
            return;
        }
        self.add_paths(&paths, window, cx);
    }

    fn add_paths(&mut self, paths: &[PathBuf], window: &mut Window, cx: &mut Context<Self>) {
        let notices = log_viewer::add_sources(&mut self.sources, paths);
        if !notices.is_empty() {
            self.notice = notices.join(" · ");
        } else if !paths.is_empty() {
            self.notice = format!("已添加 {} 个文件", paths.len());
        }
        self.error = None;
        self.rescan(cx);
        self.focus_input(window, cx);
        cx.notify();
    }

    fn apply_dropped_paths(
        &mut self,
        paths: &ExternalPaths,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let list: Vec<PathBuf> = paths.paths().to_vec();
        if list.is_empty() {
            self.notice = "未收到文件路径".into();
            cx.notify();
            return;
        }
        self.add_paths(&list, window, cx);
    }

    fn remove_source(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.sources.len() {
            self.sources.remove(index);
            self.notice = "已移除文件".into();
            self.rescan(cx);
            cx.notify();
        }
    }

    fn clear_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.sources.clear();
        self.result = ScanResult::default();
        self.error = None;
        self.notice.clear();
        self.search.update(cx, |f, cx| f.clear(cx));
        self.from_time.update(cx, |f, cx| f.clear(cx));
        self.to_time.update(cx, |f, cx| f.clear(cx));
        self.use_regex = false;
        self.levels.clear();
        self.focus_input(window, cx);
        cx.notify();
    }

    fn toggle_regex(&mut self, cx: &mut Context<Self>) {
        self.use_regex = !self.use_regex;
        self.rescan(cx);
        cx.notify();
    }

    fn toggle_level(&mut self, level: LogLevel, cx: &mut Context<Self>) {
        if let Some(pos) = self.levels.iter().position(|l| *l == level) {
            self.levels.remove(pos);
        } else {
            self.levels.push(level);
        }
        self.rescan(cx);
        cx.notify();
    }

    fn clear_levels(&mut self, cx: &mut Context<Self>) {
        self.levels.clear();
        self.rescan(cx);
        cx.notify();
    }

    fn build_query(&self, cx: &App) -> Result<LogQuery, String> {
        let search = self.search.read(cx).buffer.text().to_owned();
        let from_raw = self.from_time.read(cx).buffer.text().to_owned();
        let to_raw = self.to_time.read(cx).buffer.text().to_owned();
        let from = log_viewer::parse_filter_time(&from_raw)?;
        let to = log_viewer::parse_filter_time(&to_raw)?;
        Ok(LogQuery {
            search,
            use_regex: self.use_regex,
            from,
            to,
            levels: self.levels.clone(),
        })
    }

    fn rescan(&mut self, cx: &mut Context<Self>) {
        if self.sources.is_empty() {
            self.result = ScanResult::default();
            self.error = None;
            return;
        }
        match self.build_query(cx) {
            Ok(query) => {
                self.result = log_viewer::scan(&self.sources, &query);
                if let Some(first) = self.result.errors.first() {
                    self.error = Some(first.clone());
                } else {
                    self.error = None;
                }
            }
            Err(e) => {
                self.error = Some(e);
                self.result = ScanResult::default();
            }
        }
    }

    fn run_scan(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.notice = "已刷新".into();
        self.rescan(cx);
        cx.notify();
    }

    fn copy_row(&mut self, row: &LogRow, cx: &mut Context<Self>) {
        let text = row_copy_text(row);
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.notice = "已复制行".into();
        cx.notify();
    }

    fn copy_all_shown(&mut self, cx: &mut Context<Self>) {
        if self.result.rows.is_empty() {
            self.notice = "无结果可复制".into();
            cx.notify();
            return;
        }
        let text = self
            .result
            .rows
            .iter()
            .map(row_copy_text)
            .collect::<Vec<_>>()
            .join("\n");
        cx.write_to_clipboard(ClipboardItem::new_string(text));
        self.notice = format!("已复制 {} 行", self.result.rows.len());
        cx.notify();
    }

    fn level_color(theme: Theme, level: LogLevel) -> Rgba {
        match level {
            LogLevel::Error => theme.error,
            LogLevel::Warn => theme.warning,
            LogLevel::Info => theme.accent,
            LogLevel::Debug => theme.muted,
            LogLevel::Trace => theme.faint,
            LogLevel::Unknown => theme.faint,
        }
    }

    fn drop_zone(&self, theme: Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        let accent = theme.accent;
        let accent_muted = theme.accent_muted;
        div()
            .id("log-drop-zone")
            .w_full()
            .rounded(px(theme::radius::MD))
            .border_2()
            .border_color(theme.border)
            .border_dashed()
            .px(px(theme::space::MD))
            .py(px(theme::space::LG))
            .flex()
            .flex_col()
            .gap(px(theme::space::SM))
            .items_center()
            .justify_center()
            .can_drop(|data, _, _| data.is::<ExternalPaths>())
            .drag_over::<ExternalPaths>(move |style, _, _, _| {
                style.bg(accent_muted).border_color(accent)
            })
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                this.apply_dropped_paths(paths, window, cx);
            }))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text)
                    .child("拖入多个 .log / .jsonl / 文本日志"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child(single_line(format!(
                        "支持大文件流式扫描 · 最多 {MAX_FILES} 个文件 · 展示上限 {MAX_SHOWN_ROWS} 行"
                    ))),
            )
    }

    fn file_chips(&self, theme: Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        let mut row = div()
            .id("log-file-chips")
            .w_full()
            .flex()
            .flex_wrap()
            .gap(px(theme::space::SM));
        for (i, src) in self.sources.iter().enumerate() {
            let label = format!("{} ({})", src.file_name, format_size(src.size_bytes));
            row = row.child(
                div()
                    .id(SharedString::from(format!("log-src-{i}")))
                    .flex()
                    .items_center()
                    .gap(px(theme::space::XS))
                    .px(px(theme::space::SM))
                    .py(px(2.))
                    .rounded(px(theme::radius::SM))
                    .bg(theme.elevated)
                    .border_1()
                    .border_color(theme.border_subtle)
                    .child(
                        div()
                            .text_xs()
                            .font(theme::monospace())
                            .text_color(theme.text)
                            .child(single_line(label)),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("log-rm-{i}")))
                            .cursor_pointer()
                            .text_xs()
                            .text_color(theme.muted)
                            .hover(|s| s.text_color(theme.error))
                            .child("×")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_source(i, cx);
                            })),
                    ),
            );
        }
        row
    }

    fn toolbar(&self, theme: Theme, cx: &mut Context<Self>) -> Div {
        let levels = [
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
        ];
        let mut level_row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::space::XS))
            .child(div().text_xs().text_color(theme.muted).child("级别"))
            .child(
                chip(theme, "log-lvl-all", "全部", self.levels.is_empty()).on_click(cx.listener(
                    |this, _, _, cx| this.clear_levels(cx),
                )),
            );
        for level in levels {
            let active = self.levels.contains(&level);
            level_row = level_row.child(
                chip(theme, level.as_str(), level.as_str(), active).on_click(cx.listener(
                    move |this, _, _, cx| this.toggle_level(level, cx),
                )),
            );
        }

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(theme::space::SM))
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .gap(px(theme::space::SM))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h(px(32.))
                            .px(px(theme::space::SM))
                            .rounded(px(theme::radius::MD))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.elevated)
                            .child(self.search.clone()),
                    )
                    .child(
                        chip(theme, "log-regex", "正则", self.use_regex).on_click(cx.listener(
                            |this, _, _, cx| this.toggle_regex(cx),
                        )),
                    )
                    .child(primary_button(theme, "log-scan", "检索").on_click(cx.listener(
                        |this, _, window, cx| this.run_scan(window, cx),
                    )))
                    .child(button(theme, "log-copy-all", "复制结果").on_click(cx.listener(
                        |this, _, _, cx| this.copy_all_shown(cx),
                    )))
                    .child(button(theme, "log-clear", "清空").on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| this.clear_all(window, cx)),
                    )),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::space::SM))
                    .child(div().text_xs().text_color(theme.muted).child("从"))
                    .child(
                        div()
                            .w(px(200.))
                            .h(px(28.))
                            .px(px(theme::space::SM))
                            .rounded(px(theme::radius::MD))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.elevated)
                            .child(self.from_time.clone()),
                    )
                    .child(div().text_xs().text_color(theme.muted).child("到"))
                    .child(
                        div()
                            .w(px(200.))
                            .h(px(28.))
                            .px(px(theme::space::SM))
                            .rounded(px(theme::radius::MD))
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.elevated)
                            .child(self.to_time.clone()),
                    )
                    .child(level_row),
            )
    }

    fn status_bar(&self, theme: Theme) -> Div {
        let mut row = div()
            .w_full()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::space::SM))
            .child(status_chip(
                theme,
                format!("文件 {}", self.result.files),
                StatusKind::Neutral,
            ))
            .child(status_chip(
                theme,
                format!("扫描 {} 行", self.result.scanned_lines),
                StatusKind::Neutral,
            ))
            .child(status_chip(
                theme,
                format!(
                    "匹配 {} · 展示 {}",
                    self.result.matched_total, self.result.shown
                ),
                StatusKind::Accent,
            ));
        if self.result.truncated {
            row = row.child(status_chip(
                theme,
                format!("已截断（上限 {MAX_SHOWN_ROWS}）"),
                StatusKind::Warning,
            ));
        }
        if !self.notice.is_empty() {
            row = row.child(status_chip(theme, self.notice.clone(), StatusKind::Success));
        }
        row
    }

    fn result_list(&self, theme: Theme, cx: &mut Context<Self>) -> AnyElement {
        if self.sources.is_empty() {
            return div()
                .flex_1()
                .min_h_0()
                .flex()
                .items_center()
                .justify_center()
                .child(empty_hint(theme, "拖入日志文件开始检索"))
                .into_any_element();
        }
        if self.result.rows.is_empty() && self.error.is_none() {
            return div()
                .flex_1()
                .min_h_0()
                .flex()
                .items_center()
                .justify_center()
                .child(empty_hint(theme, "无匹配行 · 调整搜索或过滤器"))
                .into_any_element();
        }

        let mut list = div()
            .id("log-result-list")
            .track_scroll(&self.result_scroll)
            .overflow_y_scroll()
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(2.))
            .p(px(theme::space::SM));

        for (i, row) in self.result.rows.iter().enumerate() {
            list = list.child(self.row_view(theme, i, row, cx));
        }
        list.into_any_element()
    }

    fn row_view(
        &self,
        theme: Theme,
        index: usize,
        row: &LogRow,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let time = format_row_time(row.timestamp);
        let level_fg = Self::level_color(theme, row.level);
        let copy_row = row.clone();
        let mut msg = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_row()
            .flex_wrap()
            .font(theme::monospace())
            .text_sm()
            .text_color(theme.text);
        for (hit, text) in highlight_segments(&row.message, &row.highlight) {
            if text.is_empty() {
                continue;
            }
            msg = msg.child(
                div()
                    .text_color(if hit { theme.accent_fg } else { theme.text })
                    .when(hit, |d| d.bg(theme.accent).rounded(px(2.)))
                    .child(single_line(text)),
            );
        }
        if row.truncated_line {
            msg = msg.child(
                div()
                    .text_xs()
                    .text_color(theme.muted)
                    .child("…"),
            );
        }

        div()
            .id(SharedString::from(format!("log-row-{index}")))
            .w_full()
            .flex()
            .items_start()
            .gap(px(theme::space::SM))
            .px(px(theme::space::SM))
            .py(px(theme::space::XS))
            .rounded(px(theme::radius::SM))
            .hover(|s| s.bg(theme.selected))
            .child(
                div()
                    .w(px(168.))
                    .flex_shrink_0()
                    .font(theme::monospace())
                    .text_xs()
                    .text_color(theme.muted)
                    .child(single_line(time)),
            )
            .child(
                div()
                    .w(px(56.))
                    .flex_shrink_0()
                    .font(theme::monospace())
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(level_fg)
                    .child(row.level.as_str()),
            )
            .child(
                div()
                    .w(px(120.))
                    .flex_shrink_0()
                    .font(theme::monospace())
                    .text_xs()
                    .text_color(theme.faint)
                    .child(single_line(format!("{}:{}", row.source_name, row.line_no))),
            )
            .child(msg)
            .child(
                div()
                    .id(SharedString::from(format!("log-copy-{index}")))
                    .cursor_pointer()
                    .flex_shrink_0()
                    .px(px(theme::space::SM))
                    .py(px(2.))
                    .rounded(px(theme::radius::SM))
                    .text_xs()
                    .bg(theme.elevated)
                    .text_color(theme.muted)
                    .border_1()
                    .border_color(theme.border)
                    .hover(move |s| s.bg(theme.selected).text_color(theme.text))
                    .child("复制")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.copy_row(&copy_row, cx);
                    })),
            )
    }

    fn render_body(&self, cx: &mut Context<Self>) -> Div {
        let theme = Theme::get(cx);
        let mut body = div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(theme::space::SM))
            .p(px(theme::space::MD))
            .child(self.drop_zone(theme, cx));

        if !self.sources.is_empty() {
            body = body.child(self.file_chips(theme, cx));
        }
        body = body.child(self.toolbar(theme, cx));
        if let Some(err) = &self.error {
            body = body.child(banner(theme, StatusKind::Error, err.clone()));
        }
        for err in self.result.errors.iter().skip(1).take(3) {
            body = body.child(banner(theme, StatusKind::Warning, err.clone()));
        }
        body = body
            .child(self.status_bar(theme))
            .child(
                pane(theme)
                    .flex_1()
                    .min_h_0()
                    .child(
                        pane_header(theme)
                            .child(pane_title(theme, "时间线"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted)
                                    .child(single_line("按时间合并 · 无 follow / gzip")),
                            ),
                    )
                    .child(self.result_list(theme, cx)),
            );
        body
    }
}

impl Render for LogWorkbench {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::get(cx);
        let (title, subtitle) = (
            "Log Viewer",
            "多文件日志检索 · 大文件流式扫描 · 离线",
        );
        div()
            .id(ids::LOG_VIEWER)
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.bg)
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(theme::space::MD))
                    .pt(px(theme::space::SM))
                    .child(workbench_title(theme, title, subtitle)),
            )
            .child(self.render_body(cx))
    }
}
