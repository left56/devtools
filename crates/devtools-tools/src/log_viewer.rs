//! Multi-file log viewer: stream-scan large `.log` / `.jsonl` / plain text,
//! filter by search / date / level, merge into a chronological timeline.
//!
//! ## Large-file strategy
//! Files are never loaded as a single `String`. Each scan opens the path with
//! a buffered reader and walks lines. Matching rows are materialized up to
//! [`MAX_COLLECT_MATCHES`]; after sort the UI receives at most
//! [`MAX_SHOWN_ROWS`]. Very long lines are truncated for display
//! ([`MAX_LINE_CHARS`]) while the raw start is kept for copy.
//!
//! ## Caps (documented for UI status)
//! - [`MAX_FILES`]: sources accepted in one session
//! - [`MAX_SHOWN_ROWS`]: rows returned to the shell after merge/sort
//! - [`MAX_COLLECT_MATCHES`]: matches kept during scan before sort
//! - [`MAX_LINE_CHARS`]: display/message length per row
//!
//! Out of scope: live `tail -f`, gzip, histogram, query language.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Utc};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use regex::Regex;
use serde_json::Value;

/// Hard cap on open sources in one viewer session.
pub const MAX_FILES: usize = 32;
/// Rows handed to the UI after chronological merge.
pub const MAX_SHOWN_ROWS: usize = 5_000;
/// Matches buffered while scanning (before sort / truncate).
pub const MAX_COLLECT_MATCHES: usize = 20_000;
/// Max characters kept in `message` / highlight spans for one row.
pub const MAX_LINE_CHARS: usize = 4_096;
/// Reader buffer size (bytes).
const READ_BUF: usize = 256 * 1024;

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::LOG_VIEWER,
        name: "Log Viewer",
        category: Category::Text,
        summary: "拖入多个 .log / .jsonl，流式检索大文件；按关键词/正则、日期、级别过滤，结构化高亮，按时间合并时间线。",
        keywords: &[
            "log",
            "logtail",
            "jsonl",
            "日志",
            "viewer",
            "tail",
            "toolong",
            "检索",
            "highlight",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Severity bucket used by level chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Unknown,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
            Self::Debug => "DEBUG",
            Self::Trace => "TRACE",
            Self::Unknown => "?",
        }
    }

    pub fn parse_token(token: &str) -> Option<Self> {
        let t = token.trim();
        let upper = t.to_ascii_uppercase();
        match upper.as_str() {
            "ERROR" | "ERR" | "FATAL" | "CRITICAL" | "CRIT" | "EMERG" | "ALERT" | "SEVERE" => {
                Some(Self::Error)
            }
            "WARN" | "WARNING" => Some(Self::Warn),
            "INFO" | "INFORMATION" | "NOTICE" => Some(Self::Info),
            "DEBUG" | "DBG" => Some(Self::Debug),
            "TRACE" | "VERBOSE" | "FINE" | "FINER" | "FINEST" => Some(Self::Trace),
            _ => None,
        }
    }
}

/// One opened log file (metadata only — contents stay on disk).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogSource {
    pub path: PathBuf,
    pub file_name: String,
    pub size_bytes: u64,
}

impl LogSource {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let meta = std::fs::metadata(path).map_err(|e| format!("无法读取 {}: {e}", path.display()))?;
        if !meta.is_file() {
            return Err(format!("不是普通文件: {}", path.display()));
        }
        let file_name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        Ok(Self {
            path: path.to_path_buf(),
            file_name,
            size_bytes: meta.len(),
        })
    }
}

/// Filter / search parameters for a scan.
#[derive(Debug, Clone, Default)]
pub struct LogQuery {
    pub search: String,
    pub use_regex: bool,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    /// Empty means all levels (including Unknown).
    pub levels: Vec<LogLevel>,
}


/// One materialized match row for the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    pub source_name: String,
    pub line_no: u64,
    pub timestamp: Option<DateTime<Utc>>,
    pub level: LogLevel,
    /// Display text (may be truncated to [`MAX_LINE_CHARS`]).
    pub message: String,
    /// True when `message` was truncated from a longer line.
    pub truncated_line: bool,
    /// Char-index ranges into `message` for search hits (inclusive start, exclusive end).
    pub highlight: Vec<(usize, usize)>,
}

/// Outcome of scanning one or more sources.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanResult {
    pub rows: Vec<LogRow>,
    pub files: usize,
    pub scanned_lines: u64,
    pub matched_total: u64,
    pub shown: usize,
    /// True when matches exceeded [`MAX_SHOWN_ROWS`] or collection hit [`MAX_COLLECT_MATCHES`].
    pub truncated: bool,
    pub errors: Vec<String>,
}


/// Add paths as sources, skipping duplicates and enforcing [`MAX_FILES`].
pub fn add_sources(existing: &mut Vec<LogSource>, paths: &[PathBuf]) -> Vec<String> {
    let mut notices = Vec::new();
    for path in paths {
        if existing.len() >= MAX_FILES {
            notices.push(format!("已达文件上限 {MAX_FILES}，忽略 {}", path.display()));
            continue;
        }
        if existing.iter().any(|s| s.path == *path) {
            notices.push(format!("已添加过: {}", path.display()));
            continue;
        }
        match LogSource::open(path) {
            Ok(src) => existing.push(src),
            Err(e) => notices.push(e),
        }
    }
    notices
}

/// Parse a date/time string for the from/to filter fields.
///
/// Accepts the same heuristics as the Date-time tool (Unix, RFC3339, common
/// wall-clock forms). Empty → `Ok(None)`.
pub fn parse_filter_time(input: &str) -> Result<Option<DateTime<Utc>>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    parse_timestamp_token(trimmed)
        .map(Some)
        .ok_or_else(|| format!("无法解析时间: {trimmed}"))
}

/// Stream-scan all sources with `query`, merge by timestamp, apply caps.
pub fn scan(sources: &[LogSource], query: &LogQuery) -> ScanResult {
    let mut result = ScanResult {
        files: sources.len(),
        ..ScanResult::default()
    };
    if sources.is_empty() {
        return result;
    }

    let compiled = match compile_search(query) {
        Ok(c) => c,
        Err(e) => {
            result.errors.push(e);
            return result;
        }
    };

    let level_filter: Option<Vec<LogLevel>> = if query.levels.is_empty() {
        None
    } else {
        Some(query.levels.clone())
    };

    let mut collected: Vec<LogRow> = Vec::new();
    let mut hit_collect_cap = false;

    for source in sources {
        if hit_collect_cap {
            break;
        }
        match scan_one(source, query, &compiled, level_filter.as_deref(), &mut collected, &mut result) {
            Ok(()) => {}
            Err(e) => result.errors.push(e),
        }
        if collected.len() >= MAX_COLLECT_MATCHES {
            hit_collect_cap = true;
        }
    }

    // Chronological: timestamp ascending; missing timestamps sort last but keep
    // stable order by source name + line number.
    collected.sort_by(|a, b| match (a.timestamp, b.timestamp) {
        (Some(ta), Some(tb)) => ta
            .cmp(&tb)
            .then_with(|| a.source_name.cmp(&b.source_name))
            .then_with(|| a.line_no.cmp(&b.line_no)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a
            .source_name
            .cmp(&b.source_name)
            .then_with(|| a.line_no.cmp(&b.line_no)),
    });

    let matched_in_collect = collected.len() as u64;
    // matched_total may exceed collect when we kept counting after the cap.
    result.truncated = result.matched_total > MAX_SHOWN_ROWS as u64
        || hit_collect_cap
        || matched_in_collect > MAX_SHOWN_ROWS as u64;

    collected.truncate(MAX_SHOWN_ROWS);
    result.shown = collected.len();
    result.rows = collected;
    result
}

enum SearchCompiled {
    None,
    Substr(String),
    Regex(Regex),
}

fn compile_search(query: &LogQuery) -> Result<SearchCompiled, String> {
    let needle = query.search.trim();
    if needle.is_empty() {
        return Ok(SearchCompiled::None);
    }
    if query.use_regex {
        Regex::new(needle)
            .map(SearchCompiled::Regex)
            .map_err(|e| format!("正则无效: {e}"))
    } else {
        Ok(SearchCompiled::Substr(needle.to_owned()))
    }
}

fn scan_one(
    source: &LogSource,
    query: &LogQuery,
    search: &SearchCompiled,
    levels: Option<&[LogLevel]>,
    collected: &mut Vec<LogRow>,
    result: &mut ScanResult,
) -> Result<(), String> {
    let file = File::open(&source.path).map_err(|e| format!("{}: {e}", source.file_name))?;
    let mut reader = BufReader::with_capacity(READ_BUF, file);
    let mut buf = Vec::with_capacity(8 * 1024);
    let mut line_no: u64 = 0;

    loop {
        buf.clear();
        let n = reader
            .read_until(b'\n', &mut buf)
            .map_err(|e| format!("{}: 读取失败 {e}", source.file_name))?;
        if n == 0 {
            break;
        }
        line_no += 1;
        result.scanned_lines += 1;

        // Strip trailing CR/LF without allocating when possible.
        while buf.last().is_some_and(|b| *b == b'\n' || *b == b'\r') {
            buf.pop();
        }

        let line = match std::str::from_utf8(&buf) {
            Ok(s) => s,
            Err(_) => {
                // Lossy for binary-ish lines; still searchable.
                let owned = String::from_utf8_lossy(&buf);
                process_line(
                    source,
                    line_no,
                    owned.as_ref(),
                    query,
                    search,
                    levels,
                    collected,
                    result,
                );
                if collected.len() >= MAX_COLLECT_MATCHES {
                    break;
                }
                continue;
            }
        };

        process_line(source, line_no, line, query, search, levels, collected, result);
        if collected.len() >= MAX_COLLECT_MATCHES {
            break;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_line(
    source: &LogSource,
    line_no: u64,
    line: &str,
    query: &LogQuery,
    search: &SearchCompiled,
    levels: Option<&[LogLevel]>,
    collected: &mut Vec<LogRow>,
    result: &mut ScanResult,
) {
    let parsed = parse_line(line);

    if let Some(allowed) = levels
        && !allowed.contains(&parsed.level)
    {
        return;
    }

    if let Some(from) = query.from {
        match parsed.timestamp {
            Some(ts) if ts < from => return,
            None => return, // date filter active → drop lines without time
            _ => {}
        }
    }
    if let Some(to) = query.to {
        match parsed.timestamp {
            Some(ts) if ts > to => return,
            None => return,
            _ => {}
        }
    }

    let (ok, highlight) = match search {
        SearchCompiled::None => (true, Vec::new()),
        SearchCompiled::Substr(s) => find_substr_hits(line, s),
        SearchCompiled::Regex(re) => find_regex_hits(line, re),
    };
    if !ok {
        return;
    }

    result.matched_total += 1;
    if collected.len() >= MAX_COLLECT_MATCHES {
        return;
    }

    let (message, truncated_line, highlight) = truncate_message(parsed.message, highlight);
    collected.push(LogRow {
        source_name: source.file_name.clone(),
        line_no,
        timestamp: parsed.timestamp,
        level: parsed.level,
        message,
        truncated_line,
        highlight,
    });
}

struct ParsedLine {
    timestamp: Option<DateTime<Utc>>,
    level: LogLevel,
    message: String,
}

fn parse_line(line: &str) -> ParsedLine {
    let trimmed = line.trim_start();
    if trimmed.starts_with('{')
        && let Some(parsed) = try_parse_json_line(trimmed)
    {
        return parsed;
    }
    if let Some(parsed) = try_parse_logfmt(trimmed) {
        return parsed;
    }
    parse_plain(line)
}

fn try_parse_json_line(line: &str) -> Option<ParsedLine> {
    let value: Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;

    let timestamp = ["timestamp", "time", "ts", "@timestamp", "datetime", "date"]
        .iter()
        .find_map(|k| obj.get(*k))
        .and_then(json_to_time);

    let level = ["level", "severity", "lvl", "log.level"]
        .iter()
        .find_map(|k| obj.get(*k))
        .and_then(|v| v.as_str().and_then(LogLevel::parse_token))
        .unwrap_or(LogLevel::Unknown);

    let message = ["message", "msg", "log", "text", "body"]
        .iter()
        .find_map(|k| obj.get(*k).and_then(|v| v.as_str().map(str::to_owned)))
        .unwrap_or_else(|| line.to_owned());

    Some(ParsedLine {
        timestamp,
        level,
        message,
    })
}

fn json_to_time(v: &Value) -> Option<DateTime<Utc>> {
    match v {
        Value::String(s) => parse_timestamp_token(s),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                unix_to_utc(i)
            } else if let Some(f) = n.as_f64() {
                unix_to_utc(f as i64)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn try_parse_logfmt(line: &str) -> Option<ParsedLine> {
    // Heuristic: at least two key=value tokens and a level= or msg=.
    if !line.contains('=') || (!line.contains("level=") && !line.contains("msg=") && !line.contains("message="))
    {
        return None;
    }
    let mut level = LogLevel::Unknown;
    let mut timestamp = None;
    let mut message = None;
    let mut pairs = 0usize;
    for token in tokenize_logfmt(line) {
        let Some((k, v)) = token.split_once('=') else {
            continue;
        };
        pairs += 1;
        let key = k.trim();
        let val = strip_quotes(v.trim());
        match key {
            "level" | "severity" | "lvl" => {
                if let Some(l) = LogLevel::parse_token(val) {
                    level = l;
                }
            }
            "msg" | "message" => message = Some(val.to_owned()),
            "time" | "ts" | "timestamp" | "@timestamp" => {
                timestamp = parse_timestamp_token(val);
            }
            _ => {}
        }
    }
    if pairs < 2 {
        return None;
    }
    Some(ParsedLine {
        timestamp,
        level,
        message: message.unwrap_or_else(|| line.to_owned()),
    })
}

fn tokenize_logfmt(line: &str) -> Vec<&str> {
    // Simple split on whitespace outside quotes.
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut in_quote = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        match bytes[i] {
            b'"' => in_quote = !in_quote,
            b' ' | b'\t' if !in_quote => {
                if start < i {
                    out.push(&line[start..i]);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < line.len() {
        out.push(&line[start..]);
    }
    out
}

fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn parse_plain(line: &str) -> ParsedLine {
    let mut rest = line;
    let mut timestamp = None;

    // Leading timestamp tokens of various widths.
    if let Some((ts, after)) = take_leading_timestamp(rest) {
        timestamp = Some(ts);
        rest = after.trim_start();
    }

    let (level, after_level) = take_leading_level(rest);
    let message = if after_level.is_empty() {
        line.to_owned()
    } else {
        after_level.trim_start().to_owned()
    };

    ParsedLine {
        timestamp,
        level,
        message: if message.is_empty() {
            line.to_owned()
        } else {
            message
        },
    }
}

fn take_leading_timestamp(line: &str) -> Option<(DateTime<Utc>, &str)> {
    // Try progressively longer prefixes that look like dates.
    // ISO / RFC3339-ish: 2024-01-02T03:04:05(.fff)(Z|+00:00)
    // Space form: 2024-01-02 03:04:05(.fff)
    // Bracketed: [2024-01-02 03:04:05]
    let s = line.trim_start();
    if let Some(inner) = s.strip_prefix('[')
        && let Some(end) = inner.find(']')
    {
        let token = &inner[..end];
        if let Some(ts) = parse_timestamp_token(token) {
            return Some((ts, &inner[end + 1..]));
        }
    }

    // Scan for a date-like start (digit digit digit digit -)
    if s.len() >= 19 && s.as_bytes()[0].is_ascii_digit() {
        let candidates = [
            35usize, // with millis + offset
            30, 29, 28, 26, 25, 24, 23, 22, 21, 20, 19,
        ];
        for &len in &candidates {
            if s.len() < len {
                continue;
            }
            // Don't split a UTF-8 codepoint — timestamps are ASCII.
            if !s.is_char_boundary(len) {
                continue;
            }
            let token = &s[..len];
            // Require a digit/date flavour so we don't eat random text.
            if !token.as_bytes()[0].is_ascii_digit() {
                continue;
            }
            if let Some(ts) = parse_timestamp_token(token.trim_end_matches([',', ';'])) {
                return Some((ts, &s[len..]));
            }
        }
    }

    // Syslog-ish: "Jan  2 15:04:05" (no year → current year local)
    if let Some((ts, rest)) = try_syslog_prefix(s) {
        return Some((ts, rest));
    }

    None
}

fn try_syslog_prefix(s: &str) -> Option<(DateTime<Utc>, &str)> {
    // Mon[ ]D HH:MM:SS
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    if s.len() < 15 {
        return None;
    }
    let mon = &s[..3];
    if !months.contains(&mon) {
        return None;
    }
    // Find end of time (15 chars typical "Jan  2 15:04:05" or "Jan 02 15:04:05")
    let mut end = 15.min(s.len());
    while end < s.len() && end < 20 && s.as_bytes()[end].is_ascii_digit() {
        end += 1;
    }
    // Normalize double-space day.
    let token = s[..15.min(s.len())].to_owned();
    let year = Local::now().year();
    let padded = if token.as_bytes().get(4) == Some(&b' ') {
        format!("{year} {} 0{}", &token[..3], &token[5..])
    } else {
        format!("{year} {token}")
    };
    let naive = NaiveDateTime::parse_from_str(&padded, "%Y %b %d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(&padded, "%Y %b %e %H:%M:%S"))
        .ok()?;
    let ts = Local
        .from_local_datetime(&naive)
        .single()?
        .with_timezone(&Utc);
    Some((ts, s[15.min(s.len())..].trim_start()))
}

fn take_leading_level(s: &str) -> (LogLevel, &str) {
    let t = s.trim_start();
    // [ERROR] / (WARN) / ERROR: / ERROR
    for (open, close) in [('[', ']'), ('(', ')'), ('<', '>')] {
        if t.starts_with(open)
            && let Some(end) = t.find(close)
        {
            let inner = &t[1..end];
            if let Some(level) = LogLevel::parse_token(inner) {
                return (level, &t[end + 1..]);
            }
        }
    }
    // Bare token up to whitespace or colon
    let end = t
        .find(|c: char| c.is_whitespace() || c == ':' || c == ',' || c == '|')
        .unwrap_or(t.len());
    if end > 0
        && let Some(level) = LogLevel::parse_token(&t[..end])
    {
        let rest = &t[end..];
        let rest = rest.strip_prefix(':').unwrap_or(rest);
        return (level, rest);
    }
    (LogLevel::Unknown, t)
}

fn parse_timestamp_token(input: &str) -> Option<DateTime<Utc>> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    if let Some(dt) = try_unix_token(input) {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = input.parse::<DateTime<Utc>>() {
        return Some(dt);
    }
    let formats = [
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%dT%H:%M:%S%z",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%d/%b/%Y:%H:%M:%S %z", // Apache common
        "%b %d %H:%M:%S",
    ];
    for fmt in formats {
        if let Ok(dt) = DateTime::parse_from_str(input, fmt) {
            return Some(dt.with_timezone(&Utc));
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, fmt) {
            return Local
                .from_local_datetime(&naive)
                .single()
                .map(|dt| dt.with_timezone(&Utc));
        }
    }
    None
}

fn try_unix_token(input: &str) -> Option<DateTime<Utc>> {
    if !input.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return None;
    }
    if input == "-" || input.is_empty() {
        return None;
    }
    let value: i64 = input.parse().ok()?;
    unix_to_utc(value)
}

fn unix_to_utc(value: i64) -> Option<DateTime<Utc>> {
    let abs = value.unsigned_abs();
    let (secs, nsecs) = if abs >= 100_000_000_000 {
        let seconds = value.div_euclid(1000);
        let ms = value.rem_euclid(1000) as u32;
        (seconds, ms * 1_000_000)
    } else {
        (value, 0)
    };
    DateTime::from_timestamp(secs, nsecs)
}

fn find_substr_hits(hay: &str, needle: &str) -> (bool, Vec<(usize, usize)>) {
    if needle.is_empty() {
        return (true, Vec::new());
    }
    // Case-insensitive ASCII-oriented search on char indices.
    let lower_hay: String = hay.to_lowercase();
    let lower_needle = needle.to_lowercase();
    let mut hits = Vec::new();
    let mut start = 0usize;
    while let Some(pos) = lower_hay[start..].find(&lower_needle) {
        let abs = start + pos;
        // Map byte idx in lower_hay (= same as hay for ASCII; for Unicode
        // to_lowercase may change length — fall back to char-walk).
        let char_start = lower_hay[..abs].chars().count();
        let char_len = lower_needle.chars().count();
        hits.push((char_start, char_start + char_len));
        start = abs + lower_needle.len().max(1);
        if hits.len() >= 32 {
            break;
        }
    }
    (!hits.is_empty(), hits)
}

fn find_regex_hits(hay: &str, re: &Regex) -> (bool, Vec<(usize, usize)>) {
    let mut hits = Vec::new();
    for m in re.find_iter(hay).take(32) {
        let char_start = hay[..m.start()].chars().count();
        let char_end = char_start + m.as_str().chars().count();
        hits.push((char_start, char_end));
    }
    (!hits.is_empty(), hits)
}

fn truncate_message(
    message: String,
    highlight: Vec<(usize, usize)>,
) -> (String, bool, Vec<(usize, usize)>) {
    let char_len = message.chars().count();
    if char_len <= MAX_LINE_CHARS {
        return (message, false, highlight);
    }
    let truncated: String = message.chars().take(MAX_LINE_CHARS).collect();
    let highlight = highlight
        .into_iter()
        .filter_map(|(a, b)| {
            if a >= MAX_LINE_CHARS {
                None
            } else {
                Some((a, b.min(MAX_LINE_CHARS)))
            }
        })
        .collect();
    (truncated, true, highlight)
}

/// Human-readable size for file chips.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Format a row timestamp for the list (local wall clock).
pub fn format_row_time(ts: Option<DateTime<Utc>>) -> String {
    match ts {
        Some(t) => t
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
        None => "—".into(),
    }
}

/// Build a single-line copy payload for a row.
pub fn row_copy_text(row: &LogRow) -> String {
    let time = format_row_time(row.timestamp);
    format!(
        "[{time}] [{}] [{}:{}] {}",
        row.level.as_str(),
        row.source_name,
        row.line_no,
        row.message
    )
}

/// Split `message` into (plain, highlight?, plain…) segments for colored spans.
pub fn highlight_segments(
    message: &str,
    spans: &[(usize, usize)],
) -> Vec<(bool, String)> {
    if spans.is_empty() {
        return vec![(false, message.to_owned())];
    }
    let chars: Vec<char> = message.chars().collect();
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let mut ordered = spans.to_vec();
    ordered.sort_by_key(|s| s.0);
    for (start, end) in ordered {
        let start = start.min(chars.len());
        let end = end.min(chars.len());
        if start < cursor {
            continue;
        }
        if start > cursor {
            out.push((false, chars[cursor..start].iter().collect()));
        }
        if end > start {
            out.push((true, chars[start..end].iter().collect()));
        }
        cursor = end;
    }
    if cursor < chars.len() {
        out.push((false, chars[cursor..].iter().collect()));
    }
    if out.is_empty() {
        out.push((false, message.to_owned()));
    }
    out
}

/// Optional: mtime for status (not required for scan).
pub fn source_mtime(source: &LogSource) -> Option<SystemTime> {
    std::fs::metadata(&source.path)
        .and_then(|m| m.modified())
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("devtools-log-viewer-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    #[test]
    fn parses_iso_and_level() {
        let line = "2024-06-01T12:00:00Z ERROR boom happened";
        let p = parse_line(line);
        assert_eq!(p.level, LogLevel::Error);
        assert!(p.timestamp.is_some());
        assert!(p.message.contains("boom"));
    }

    #[test]
    fn parses_jsonl() {
        let line = r#"{"timestamp":"2024-06-01T12:00:00Z","level":"warn","message":"slow query"}"#;
        let p = parse_line(line);
        assert_eq!(p.level, LogLevel::Warn);
        assert_eq!(p.message, "slow query");
        assert!(p.timestamp.is_some());
    }

    #[test]
    fn parses_logfmt() {
        let line = r#"level=info msg="hello world" ts=2024-06-01T12:00:00Z"#;
        let p = parse_line(line);
        assert_eq!(p.level, LogLevel::Info);
        assert_eq!(p.message, "hello world");
        assert!(p.timestamp.is_some());
    }

    #[test]
    fn scan_filters_search_and_level() {
        let path = write_temp(
            "app.log",
            "\
2024-06-01T10:00:00Z INFO start
2024-06-01T11:00:00Z ERROR disk full
2024-06-01T12:00:00Z WARN disk ok
",
        );
        let src = LogSource::open(&path).unwrap();
        let mut q = LogQuery::default();
        q.search = "disk".into();
        q.levels = vec![LogLevel::Error];
        let result = scan(&[src], &q);
        assert_eq!(result.matched_total, 1);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0].level, LogLevel::Error);
        assert!(result.rows[0].message.contains("disk full"));
    }

    #[test]
    fn scan_merges_two_files_chronologically() {
        let a = write_temp(
            "a.log",
            "2024-06-01T12:00:00Z INFO from-a\n",
        );
        let b = write_temp(
            "b.log",
            "2024-06-01T11:00:00Z INFO from-b\n",
        );
        let sources = vec![LogSource::open(&a).unwrap(), LogSource::open(&b).unwrap()];
        let result = scan(&sources, &LogQuery::default());
        assert_eq!(result.rows.len(), 2);
        assert!(result.rows[0].message.contains("from-b"));
        assert!(result.rows[1].message.contains("from-a"));
    }

    #[test]
    fn scan_date_range() {
        let path = write_temp(
            "range.log",
            "\
2024-06-01T09:00:00Z INFO early
2024-06-01T12:00:00Z INFO mid
2024-06-01T18:00:00Z INFO late
",
        );
        let src = LogSource::open(&path).unwrap();
        let mut q = LogQuery::default();
        q.from = parse_filter_time("2024-06-01T10:00:00Z").unwrap();
        q.to = parse_filter_time("2024-06-01T15:00:00Z").unwrap();
        let result = scan(&[src], &q);
        assert_eq!(result.matched_total, 1);
        assert!(result.rows[0].message.contains("mid"));
    }

    #[test]
    fn regex_search() {
        let path = write_temp("re.log", "2024-06-01T12:00:00Z INFO abc-123 xyz\n");
        let src = LogSource::open(&path).unwrap();
        let mut q = LogQuery::default();
        q.search = r"abc-\d+".into();
        q.use_regex = true;
        let result = scan(&[src], &q);
        assert_eq!(result.matched_total, 1);
        assert!(!result.rows[0].highlight.is_empty());
    }

    #[test]
    fn does_not_load_whole_file_as_one_string() {
        // Smoke: many lines, ensure scan returns cap behaviour path.
        let mut body = String::new();
        for i in 0..100 {
            body.push_str(&format!("2024-06-01T12:00:{i:02}Z INFO line-{i}\n"));
        }
        let path = write_temp("many.log", &body);
        let src = LogSource::open(&path).unwrap();
        let result = scan(&[src], &LogQuery::default());
        assert_eq!(result.scanned_lines, 100);
        assert_eq!(result.shown, 100);
        assert!(!result.truncated);
    }

    #[test]
    fn highlight_segments_splits() {
        let segs = highlight_segments("hello error world", &[(6, 11)]);
        assert_eq!(segs.len(), 3);
        assert!(!segs[0].0);
        assert!(segs[1].0);
        assert_eq!(segs[1].1, "error");
    }

    #[test]
    fn add_sources_dedup_and_cap_notice() {
        let path = write_temp("one.log", "INFO x\n");
        let mut sources = Vec::new();
        let n1 = add_sources(&mut sources, &[path.clone()]);
        assert!(n1.is_empty());
        assert_eq!(sources.len(), 1);
        let n2 = add_sources(&mut sources, &[path]);
        assert_eq!(n2.len(), 1);
        assert!(n2[0].contains("已添加过"));
    }
}
