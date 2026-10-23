//! Unix seconds / milliseconds ↔ datetime. Default zone is local; UTC is an option.
//!
//! Uses `chrono` only — no platform calendar APIs — so the same logic runs on
//! macOS, Linux, and Windows once the shell builds there.

use std::fmt;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Seconds,
    Milliseconds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Local,
    Utc,
}

impl Unit {
    pub fn label(self) -> &'static str {
        match self {
            Self::Seconds => "秒",
            Self::Milliseconds => "毫秒",
        }
    }
}

impl Zone {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "本机时区",
            Self::Utc => "UTC",
        }
    }
}

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

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::TIMESTAMP,
        name: "Timestamp",
        category: Category::Time,
        summary: "Unix 秒/毫秒与日期时间互转。默认本机时区，可改 UTC。",
        keywords: &["timestamp", "unix", "epoch", "时间戳", "毫秒", "时区"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";
const FORMAT_NO_FRAC: &str = "%Y-%m-%d %H:%M:%S";

/// Convert a numeric Unix timestamp into a datetime string.
pub fn from_unix(input: &str, unit: Unit, zone: Zone) -> Result<String, TimestampError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TimestampError {
            message: "输入为空".into(),
        });
    }
    if !trimmed.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return Err(TimestampError {
            message: "时间戳只能是整数".into(),
        });
    }
    let value: i64 = trimmed.parse().map_err(|_| TimestampError {
        message: "时间戳超出范围或不是整数".into(),
    })?;
    let datetime = match unit {
        Unit::Seconds => match zone {
            Zone::Local => Local
                .timestamp_opt(value, 0)
                .single()
                .map(|dt| dt.format(FORMAT_NO_FRAC).to_string()),
            Zone::Utc => Utc
                .timestamp_opt(value, 0)
                .single()
                .map(|dt| dt.format(FORMAT_NO_FRAC).to_string()),
        },
        Unit::Milliseconds => {
            let seconds = value.div_euclid(1000);
            let nanos = (value.rem_euclid(1000) * 1_000_000) as u32;
            match zone {
                Zone::Local => Local
                    .timestamp_opt(seconds, nanos)
                    .single()
                    .map(|dt| dt.format(FORMAT).to_string()),
                Zone::Utc => Utc
                    .timestamp_opt(seconds, nanos)
                    .single()
                    .map(|dt| dt.format(FORMAT).to_string()),
            }
        }
    };
    datetime.ok_or_else(|| TimestampError {
        message: "无法把该时间戳转成日期时间".into(),
    })
}

/// Convert a datetime string into a Unix timestamp.
///
/// Accepted forms: `YYYY-MM-DD HH:MM:SS` or with fractional seconds
/// (`.` or `,` separator, up to 9 digits). Zone is interpreted per `zone`.
pub fn to_unix(input: &str, unit: Unit, zone: Zone) -> Result<String, TimestampError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(TimestampError {
            message: "输入为空".into(),
        });
    }
    let normalized = trimmed.replace(',', ".");
    let naive = parse_naive(&normalized).ok_or_else(|| TimestampError {
        message: "无法解析日期时间，期望形如 2024-01-15 12:30:00 或带小数秒".into(),
    })?;
    let millis = match zone {
        Zone::Local => Local
            .from_local_datetime(&naive)
            .single()
            .map(|dt: DateTime<Local>| dt.timestamp_millis()),
        Zone::Utc => Some(naive.and_utc().timestamp_millis()),
    }
    .ok_or_else(|| TimestampError {
        message: "该本机时间不存在或有歧义（例如夏令时跳变）".into(),
    })?;
    Ok(match unit {
        Unit::Seconds => (millis.div_euclid(1000)).to_string(),
        Unit::Milliseconds => millis.to_string(),
    })
}

fn parse_naive(input: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(input, FORMAT)
        .or_else(|_| NaiveDateTime::parse_from_str(input, FORMAT_NO_FRAC))
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%.f"))
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_rejected() {
        assert!(from_unix("", Unit::Seconds, Zone::Utc).is_err());
        assert!(to_unix("", Unit::Seconds, Zone::Utc).is_err());
    }

    #[test]
    fn unix_epoch_utc() {
        assert_eq!(
            from_unix("0", Unit::Seconds, Zone::Utc).unwrap(),
            "1970-01-01 00:00:00"
        );
        assert_eq!(
            to_unix("1970-01-01 00:00:00", Unit::Seconds, Zone::Utc).unwrap(),
            "0"
        );
    }

    #[test]
    fn milliseconds_round_trip_utc() {
        let stamp = "1700000000123";
        let text = from_unix(stamp, Unit::Milliseconds, Zone::Utc).unwrap();
        assert!(text.starts_with("2023-11-14"));
        assert_eq!(
            to_unix(&text, Unit::Milliseconds, Zone::Utc).unwrap(),
            stamp
        );
    }

    #[test]
    fn invalid_timestamp_text() {
        assert!(from_unix("abc", Unit::Seconds, Zone::Utc).is_err());
        assert!(to_unix("not a date", Unit::Seconds, Zone::Utc).is_err());
    }
}
