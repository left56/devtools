//! Lightweight download throughput probe (Explicit). Not a full Ookla-class test.

use std::io::Read;
use std::time::{Duration, Instant};

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SPEED_TEST,
        name: "Speed Test",
        category: Category::Web,
        summary: "测速（搁置，不在侧栏展示）。后续可接全球测速接口。",
        keywords: &["speed", "bandwidth", "latency", "测速", "带宽"],
        network: NetworkUse::Explicit,
        availability: Availability::LogicOnly,
    }
}

pub fn run() -> Result<Vec<(String, String)>, String> {
    let mut rows = Vec::new();

    let t0 = Instant::now();
    let _ = ureq::get("https://speed.cloudflare.com/__down?bytes=0")
        .timeout(Duration::from_secs(10))
        .set("User-Agent", "devtools/0.1 (Explicit run)")
        .call()
        .map_err(|e| format!("延迟探测失败：{e}"))?;
    let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
    rows.push(("延迟".into(), format!("{latency_ms:.0} ms（到 Cloudflare）")));

    let bytes = 1_048_576u64;
    let url = format!("https://speed.cloudflare.com/__down?bytes={bytes}");
    let t1 = Instant::now();
    let resp = ureq::get(&url)
        .timeout(Duration::from_secs(30))
        .set("User-Agent", "devtools/0.1 (Explicit run)")
        .call()
        .map_err(|e| format!("下载失败：{e}"))?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(bytes + 64)
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取失败：{e}"))?;
    let elapsed = t1.elapsed().as_secs_f64().max(1e-6);
    let got = buf.len() as f64;
    let mbps = (got * 8.0) / elapsed / 1_000_000.0;
    rows.push(("下载字节".into(), format!("{} / 目标 {bytes}", buf.len())));
    rows.push(("耗时".into(), format!("{elapsed:.2} s")));
    rows.push(("粗测下载".into(), format!("{mbps:.2} Mbps")));
    rows.push((
        "说明".into(),
        "单线程短下载，结果受路由/CPU/限速影响；上传测速未实现。".into(),
    ));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_is_explicit() {
        assert_eq!(meta().network, NetworkUse::Explicit);
        assert_eq!(meta().id, ids::SPEED_TEST);
    }
}
