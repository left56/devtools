//! PEM / X.509 certificate decode (offline text).

use x509_parser::prelude::*;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::CERT_DECODE,
        name: "Certificate",
        category: Category::Security,
        summary: "解码 PEM/DER 文本证书：主题、颁发者、有效期、序列号、SAN 等。不校验证书链。",
        keywords: &["certificate", "x509", "pem", "der", "证书", "ssl", "tls"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("粘贴 PEM 证书（-----BEGIN CERTIFICATE-----）或 Base64 DER".into());
    }
    let mut rows = Vec::new();
    let mut index = 0usize;

    if t.contains("BEGIN") {
        for pem in x509_parser::pem::Pem::iter_from_buffer(t.as_bytes()) {
            let pem = pem.map_err(|e| format!("PEM 解析失败：{e}"))?;
            index += 1;
            let prefix = if index == 1 {
                ""
            } else {
                // keep labels static: only annotate in values for multi-cert
                ""
            };
            let _ = prefix;
            let cert = pem
                .parse_x509()
                .map_err(|e| format!("X.509 解析失败：{e}"))?;
            append_cert_rows(&mut rows, &cert, index, index > 1)?;
        }
        if index == 0 {
            return Err("未找到 PEM 块".into());
        }
    } else {
        // Try raw base64 DER
        let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
        let der = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &compact)
            .map_err(|e| format!("既不是 PEM 也不是 Base64 DER：{e}"))?;
        let (_, cert) = X509Certificate::from_der(&der).map_err(|e| format!("DER 解析失败：{e}"))?;
        append_cert_rows(&mut rows, &cert, 1, false)?;
    }
    Ok(rows)
}

fn append_cert_rows(
    rows: &mut Vec<(&'static str, String)>,
    cert: &X509Certificate<'_>,
    index: usize,
    multi: bool,
) -> Result<(), String> {
    if multi {
        rows.push(("证书序号", index.to_string()));
    }
    rows.push(("版本", format!("v{}", cert.version().0 + 1)));
    rows.push(("序列号", cert.raw_serial_as_string()));
    rows.push(("签名算法", cert.signature_algorithm.algorithm.to_string()));
    rows.push(("主题", cert.subject().to_string()));
    rows.push(("颁发者", cert.issuer().to_string()));
    let validity = cert.validity();
    rows.push((
        "生效",
        validity.not_before.to_datetime().to_string(),
    ));
    rows.push((
        "到期",
        validity.not_after.to_datetime().to_string(),
    ));
    let now_ok = validity.is_valid();
    rows.push(("当前是否在有效期内", if now_ok { "是".into() } else { "否".into() }));

    if let Ok(Some(san)) = cert.subject_alternative_name() {
        let names: Vec<String> = san
            .value
            .general_names
            .iter()
            .map(|gn| format!("{gn}"))
            .collect();
        if !names.is_empty() {
            rows.push(("SAN", names.join(", ")));
        }
    }
    if let Ok(Some(ku)) = cert.key_usage() {
        rows.push(("密钥用法", format!("{}", ku.value)));
    }
    if let Ok(Some(ext)) = cert.basic_constraints() {
        let ca = if ext.value.ca { "是" } else { "否" };
        let path = ext
            .value
            .path_len_constraint
            .map(|n| n.to_string())
            .unwrap_or_else(|| "—".into());
        rows.push(("CA", format!("{ca} · pathLen={path}")));
    }
    rows.push((
        "说明",
        "仅解码展示，未验证签名或证书链".into(),
    ));
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../tests/fixtures/devtools-test.pem");

    #[test]
    fn decodes_pem_fixture() {
        let rows = convert_all(SAMPLE).unwrap();
        let subject = rows.iter().find(|r| r.0 == "主题").unwrap().1.clone();
        assert!(subject.contains("devtools.test"), "{subject}");
        assert!(rows.iter().any(|r| r.0 == "到期"));
    }

    #[test]
    fn empty_rejected() {
        assert!(convert_all("").is_err());
    }
}
