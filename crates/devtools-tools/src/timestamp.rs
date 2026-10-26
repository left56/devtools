//! Date-time converter: one input, auto-detect, all formats out.
//!
//! Uses `chrono` only — no platform calendar APIs — so the same logic runs on
//! macOS, Linux, and Windows once the shell builds there.

use std::fmt;

use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc, Weekday};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampError {
    pub message: String,
}

impl fmt::Display for TimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TimestampError {}

/// One labeled conversion result (UI shows a row + copy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatRow {
    pub label: &'static str,
    pub value: String,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::TIMESTAMP,
        name: "Date-time",
        category: Category::Time,
        summary: "自动识别 Unix 秒/毫秒、ISO、常见日期字符串，批量输出各格式。默认填入当前本机时间。",
        keywords: &[
            "timestamp",
            "unix",
            "epoch",
            "时间戳",
            "日期",
            "date",
            "time",
            "iso",
            "rfc",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Seed text for an empty open / clear-to-default: current local wall time.
pub fn now_seed() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Parse any supported input and emit every format row.
pub fn convert_all(input: &str) -> Result<Vec<FormatRow>, TimestampError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TimestampError {
            message: "输入为空".into(),
        });
    }
    let utc = parse_to_utc(trimmed)?;
    Ok(format_all(utc))
}

fn parse_to_utc(input: &str) -> Result<DateTime<Utc>, TimestampError> {
    if let Some(dt) = try_unix(input) {
        return Ok(dt);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.with_timezone(&Utc));
    }
    // Accept Z / offset ISO without full RFC3339 strictness via chrono helpers.
    if let Ok(dt) = input.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }
    if let Ok(dt) = DateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%.f%z") {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = DateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S%.f%z") {
        return Ok(dt.with_timezone(&Utc));
    }
    // RFC 7231 / HTTP-date
    if let Ok(dt) = DateTime::parse_from_str(input, "%a, %d %b %Y %H:%M:%S GMT") {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Some(dt) = try_excel(input) {
        return Ok(dt);
    }
    if let Some(naive) = parse_naive(input) {
        return Local
            .from_local_datetime(&naive)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| TimestampError {
                message: "该本机时间不存在或有歧义（例如夏令时跳变）".into(),
            });
    }
    Err(TimestampError {
        message: "无法识别：支持 Unix 秒/毫秒、ISO 8601、RFC 3339、YYYY-MM-DD HH:MM:SS、Excel 序列".into(),
    })
}

fn try_unix(input: &str) -> Option<DateTime<Utc>> {
    if !input.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return None;
    }
    // Reject bare signs / empty after trim already handled.
    if input == "-" || input.is_empty() {
        return None;
    }
    let value: i64 = input.parse().ok()?;
    let abs = value.unsigned_abs();
    // 11+ digit magnitude → milliseconds; else seconds. Negative allowed.
    let (secs, nsecs) = if abs >= 100_000_000_000 {
        let seconds = value.div_euclid(1000);
        let nanos = (value.rem_euclid(1000) * 1_000_000) as u32;
        (seconds, nanos)
    } else {
        (value, 0)
    };
    Utc.timestamp_opt(secs, nsecs).single()
}

fn try_excel(input: &str) -> Option<DateTime<Utc>> {
    let serial: f64 = input.parse().ok()?;
    // Excel serial day 0 = 1899-12-30 (Lotus 1-2-3 compatibility).
    if !(1.0..3_000_000.0).contains(&serial) {
        return None;
    }
    // Only treat as Excel when it has a fractional part or is clearly a day count
    // in a sensible range — avoid stealing small unix seconds like "1700000000".
    if input.bytes().all(|b| b.is_ascii_digit() || b == b'-') && !input.contains('.') {
        // Pure integer: only Excel if outside unix-second heuristic range and
        // looks like a day serial (roughly 1..60000 for years 1900–2064).
        if !(1..=120_000).contains(&(serial as i64)) {
            return None;
        }
        // Prefer unix for 9–10 digit integers.
        let digits = input.trim_start_matches('-').len();
        if (9..=13).contains(&digits) {
            return None;
        }
    }
    let days = serial.floor() as i64;
    let frac = serial - serial.floor();
    let millis = (frac * 86_400_000.0).round() as i64;
    let excel_epoch = NaiveDateTime::parse_from_str("1899-12-30 00:00:00", "%Y-%m-%d %H:%M:%S").ok()?;
    let base = excel_epoch + chrono::Duration::days(days) + chrono::Duration::milliseconds(millis);
    Some(DateTime::<Utc>::from_naive_utc_and_offset(base, Utc))
}

fn parse_naive(input: &str) -> Option<NaiveDateTime> {
    let normalized = input.replace(',', ".");
    const FORMS: &[&str] = &[
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];
    for form in FORMS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, form) {
            return Some(dt);
        }
        // Date-only: midnight
        if (*form == "%Y-%m-%d" || *form == "%Y/%m/%d")
            && let Ok(d) = chrono::NaiveDate::parse_from_str(&normalized, form)
        {
            return d.and_hms_opt(0, 0, 0);
        }
    }
    None
}

fn format_all(utc: DateTime<Utc>) -> Vec<FormatRow> {
    let local: DateTime<Local> = utc.with_timezone(&Local);
    let js_locale = format_js_locale(&local);
    let iso8601 = local.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let iso9075 = local.format("%Y-%m-%d %H:%M:%S").to_string();
    let rfc3339 = local.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let rfc7231 = utc.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let unix_s = utc.timestamp().to_string();
    let unix_ms = utc.timestamp_millis().to_string();
    let utc_str = utc.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let mongo = format!("{:08x}0000000000000000", utc.timestamp() as u32);
    let excel = excel_serial(utc);

    vec![
        FormatRow {
            label: "本机时间",
            value: js_locale,
        },
        FormatRow {
            label: "ISO 8601",
            value: iso8601,
        },
        FormatRow {
            label: "ISO 9075",
            value: iso9075,
        },
        FormatRow {
            label: "RFC 3339",
            value: rfc3339,
        },
        FormatRow {
            label: "RFC 7231",
            value: rfc7231,
        },
        FormatRow {
            label: "Unix 秒",
            value: unix_s,
        },
        FormatRow {
            label: "Unix 毫秒",
            value: unix_ms,
        },
        FormatRow {
            label: "UTC",
            value: utc_str,
        },
        FormatRow {
            label: "Mongo ObjectID",
            value: mongo,
        },
        FormatRow {
            label: "Excel",
            value: excel,
        },
    ]
}

fn format_js_locale(local: &DateTime<Local>) -> String {
    // Approximate `Date.prototype.toString()` style with Chinese CST label when +08.
    let wd = weekday_short(local.weekday());
    let mon = month_short(local.month());
    let offset = local.offset().to_string(); // e.g. +08:00
    let gmt = offset.replace(':', "");
    let zone_name = if offset == "+08:00" {
        "中国标准时间"
    } else {
        "本地时间"
    };
    format!(
        "{wd} {mon} {:02} {} {:02}:{:02}:{:02} GMT{gmt} ({zone_name})",
        local.day(),
        local.year(),
        local.hour(),
        local.minute(),
        local.second(),
    )
}

fn weekday_short(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}

fn month_short(m: u32) -> &'static str {
    match m {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

fn excel_serial(utc: DateTime<Utc>) -> String {
    let excel_epoch =
        NaiveDateTime::parse_from_str("1899-12-30 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
    let naive = utc.naive_utc();
    let delta = naive - excel_epoch;
    let days = delta.num_milliseconds() as f64 / 86_400_000.0;
    // Trim trailing zeros lightly.
    let s = format!("{days:.10}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_rejected() {
        assert!(convert_all("").is_err());
        assert!(convert_all("   ").is_err());
    }

    #[test]
    fn unix_epoch_seconds() {
        let rows = convert_all("0").unwrap();
        let by = |label| {
            rows.iter()
                .find(|r| r.label == label)
                .map(|r| r.value.as_str())
                .unwrap()
        };
        assert_eq!(by("Unix 秒"), "0");
        assert_eq!(by("Unix 毫秒"), "0");
        assert_eq!(by("RFC 7231"), "Thu, 01 Jan 1970 00:00:00 GMT");
        assert_eq!(by("Mongo ObjectID"), "000000000000000000000000");
    }

    #[test]
    fn unix_milliseconds() {
        let rows = convert_all("1700000000123").unwrap();
        let ms = rows.iter().find(|r| r.label == "Unix 毫秒").unwrap();
        assert_eq!(ms.value, "1700000000123");
        let iso = rows.iter().find(|r| r.label == "ISO 9075").unwrap();
        assert!(iso.value.starts_with("2023-11-1") || iso.value.starts_with("2023-11-15") || iso.value.starts_with("2023-11-14"));
    }

    #[test]
    fn iso8601_round_formats() {
        let rows = convert_all("2026-10-24T04:11:48Z").unwrap();
        assert_eq!(
            rows.iter().find(|r| r.label == "Unix 秒").unwrap().value,
            "1792815108"
        );
        assert_eq!(
            rows.iter().find(|r| r.label == "RFC 7231").unwrap().value,
            "Sat, 24 Oct 2026 04:11:48 GMT"
        );
    }

    #[test]
    fn naive_local_parses() {
        let seed = now_seed();
        assert!(convert_all(&seed).is_ok());
        assert!(convert_all("2024-01-15 12:30:00").is_ok());
    }

    #[test]
    fn invalid_text() {
        assert!(convert_all("not a date").is_err());
        assert!(convert_all("abc").is_err());
    }

    #[test]
    fn now_seed_non_empty() {
        assert!(!now_seed().is_empty());
    }

    #[test]
    fn excel_serial_epochish() {
        // Excel day 1 ≈ 1900-01-01
        let rows = convert_all("1.0").unwrap();
        let rfc = rows.iter().find(|r| r.label == "RFC 7231").unwrap();
        assert!(rfc.value.contains("1899") || rfc.value.contains("1900"));
    }
}
