//! 5-field cron → next run times. Zone is explicit Local or UTC.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Local, Utc};
use cron::Schedule;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    Local,
    Utc,
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
pub struct CronError {
    pub message: String,
}

impl fmt::Display for CronError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CronError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronResult {
    pub next: Vec<String>,
    pub zone_label: &'static str,
    pub normalized_expr: String,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::CRON,
        name: "Cron",
        category: Category::Time,
        summary: "5 段 cron，给出之后 5 次运行时间。时区可选本机或 UTC。",
        keywords: &["cron", "schedule", "crontab", "定时", "调度"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Parse a 5-field expression (`min hour dom month dow`) and return the next
/// `count` firings after `now` in the chosen zone.
pub fn next_runs(expr: &str, zone: Zone, count: usize) -> Result<CronResult, CronError> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(CronError {
            message: "输入为空".into(),
        });
    }
    let fields: Vec<_> = trimmed.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(CronError {
            message: format!(
                "需要恰好 5 段（分 时 日 月 周），当前 {} 段",
                fields.len()
            ),
        });
    }
    // `cron` crate expects seconds as the first field.
    let with_seconds = format!("0 {}", fields.join(" "));
    let schedule = Schedule::from_str(&with_seconds).map_err(|e| CronError {
        message: format!("无法解析 cron：{e}"),
    })?;

    let next: Vec<String> = match zone {
        Zone::Utc => {
            let now = Utc::now();
            schedule
                .after(&now)
                .take(count)
                .map(|dt: DateTime<Utc>| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .collect()
        }
        Zone::Local => {
            // cron::Schedule iterates in UTC; convert each firing into Local for display.
            let now = Utc::now();
            schedule
                .after(&now)
                .take(count)
                .map(|dt: DateTime<Utc>| {
                    let local: DateTime<Local> = dt.with_timezone(&Local);
                    format!("{} ({})", local.format("%Y-%m-%d %H:%M:%S"), Zone::Local.label())
                })
                .collect()
        }
    };

    if next.is_empty() {
        return Err(CronError {
            message: "在所选时区下找不到后续运行时间".into(),
        });
    }

    Ok(CronResult {
        next,
        zone_label: zone.label(),
        normalized_expr: fields.join(" "),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rejected() {
        assert!(next_runs("", Zone::Utc, 5).is_err());
    }

    #[test]
    fn five_field_hourly_utc() {
        let out = next_runs("0 * * * *", Zone::Utc, 5).unwrap();
        assert_eq!(out.next.len(), 5);
        assert_eq!(out.zone_label, "UTC");
        for line in &out.next {
            assert!(line.ends_with("UTC"));
            assert!(line.contains(":00:00"));
        }
    }

    #[test]
    fn wrong_field_count() {
        let err = next_runs("* * *", Zone::Local, 5).unwrap_err();
        assert!(err.message.contains("5 段"));
    }

    #[test]
    fn local_zone_labelled() {
        let out = next_runs("*/15 * * * *", Zone::Local, 3).unwrap();
        assert_eq!(out.next.len(), 3);
        assert!(out.next[0].contains("本机时区"));
    }
}
