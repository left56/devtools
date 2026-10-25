//! HTTP request tool. Network is contacted only when the user explicitly runs.
//! Opening the tool must not send a request.

use std::fmt;
use std::time::Duration;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_uppercase().as_str() {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PUT" => Some(Self::Put),
            "DELETE" => Some(Self::Delete),
            "PATCH" => Some(Self::Patch),
            "HEAD" => Some(Self::Head),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpError {
    pub message: String,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedRequest {
    pub method: Method,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub url: String,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::HTTP_REQUEST,
        name: "HTTP Client",
        category: Category::Web,
        summary: "HTTP 客户端：显式发送请求。打开工具不会联网；需点击「运行」。",
        keywords: &["http", "request", "client", "curl", "api", "请求", "客户端", "联网"],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

/// Parse `Key: Value` lines. Empty lines skipped. Lines without `:` are errors.
pub fn parse_headers(text: &str) -> Result<Vec<(String, String)>, HttpError> {
    let mut out = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(HttpError {
                message: format!("第 {} 行缺少 ':'：{line}", index + 1),
            });
        };
        let name = name.trim();
        if name.is_empty() {
            return Err(HttpError {
                message: format!("第 {} 行请求头名为空", index + 1),
            });
        }
        out.push((name.to_owned(), value.trim().to_owned()));
    }
    Ok(out)
}

/// Validate and assemble a request without contacting the network.
pub fn prepare(
    method: Method,
    url: &str,
    headers_text: &str,
    body: &str,
) -> Result<PreparedRequest, HttpError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(HttpError {
            message: "URL 为空".into(),
        });
    }
    let parsed = url::Url::parse(url).map_err(|e| HttpError {
        message: format!("无法解析 URL：{e}"),
    })?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(HttpError {
            message: format!("仅支持 http/https，当前是 {}", parsed.scheme()),
        });
    }
    let headers = parse_headers(headers_text)?;
    Ok(PreparedRequest {
        method,
        url: parsed.to_string(),
        headers,
        body: body.to_owned(),
    })
}

/// Live request. Caller must only invoke after the user explicitly confirms.
pub fn execute(prepared: &PreparedRequest) -> Result<HttpResponse, HttpError> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .build();

    let mut request = match prepared.method {
        Method::Get => agent.get(&prepared.url),
        Method::Post => agent.post(&prepared.url),
        Method::Put => agent.put(&prepared.url),
        Method::Delete => agent.delete(&prepared.url),
        Method::Patch => agent.request("PATCH", &prepared.url),
        Method::Head => agent.request("HEAD", &prepared.url),
    };

    for (name, value) in &prepared.headers {
        request = request.set(name, value);
    }

    let send_body = matches!(
        prepared.method,
        Method::Post | Method::Put | Method::Patch | Method::Delete
    ) && !prepared.body.is_empty();
    let result = if send_body {
        request.send_string(&prepared.body)
    } else {
        request.call()
    };

    match result {
        Ok(response) => read_response(response),
        Err(ureq::Error::Status(code, response)) => {
            let mut http = read_response(response)?;
            http.status = code;
            Ok(http)
        }
        Err(err) => Err(HttpError {
            message: format!("请求失败：{err}"),
        }),
    }
}

fn read_response(response: ureq::Response) -> Result<HttpResponse, HttpError> {
    let status = response.status();
    let status_text = response.status_text().to_owned();
    let url = response.get_url().to_owned();
    let mut headers = Vec::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            headers.push((name, value.to_owned()));
        }
    }
    let body = response.into_string().unwrap_or_default();
    Ok(HttpResponse {
        status,
        status_text,
        headers,
        body,
        url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_is_explicit_network() {
        let m = meta();
        assert_eq!(m.id, ids::HTTP_REQUEST);
        assert_eq!(m.network, NetworkUse::Explicit);
        assert_eq!(m.availability, Availability::Ready);
    }

    #[test]
    fn prepare_rejects_empty_and_non_http() {
        assert!(prepare(Method::Get, "", "", "").is_err());
        assert!(prepare(Method::Get, "ftp://example.com/", "", "").is_err());
    }

    #[test]
    fn parse_headers_typical_and_bad() {
        let headers = parse_headers("Accept: application/json\nX-Test: 1\n").unwrap();
        assert_eq!(
            headers,
            vec![
                ("Accept".into(), "application/json".into()),
                ("X-Test".into(), "1".into())
            ]
        );
        assert!(parse_headers("no-colon").is_err());
    }

    #[test]
    fn prepare_keeps_https_url() {
        let prepared = prepare(
            Method::Post,
            "https://example.com/api?x=1",
            "Content-Type: application/json",
            "{\"a\":1}",
        )
        .unwrap();
        assert_eq!(prepared.method, Method::Post);
        assert!(prepared.url.starts_with("https://example.com/"));
        assert_eq!(prepared.headers.len(), 1);
        assert_eq!(prepared.body, "{\"a\":1}");
    }

    #[test]
    fn method_parse() {
        assert_eq!(Method::parse("get"), Some(Method::Get));
        assert_eq!(Method::parse("PATCH"), Some(Method::Patch));
        assert!(Method::parse("TRACE").is_none());
    }
}
