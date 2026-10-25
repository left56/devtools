//! Lightweight SVG beautify / minify (not a full vector editor).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgMode {
    Beautify,
    Minify,
}

impl SvgMode {
    pub const ALL: [Self; 2] = [Self::Beautify, Self::Minify];
    pub fn label(self) -> &'static str {
        match self {
            Self::Beautify => "美化",
            Self::Minify => "压缩",
        }
    }
    pub fn default_mode() -> Self {
        Self::Beautify
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SVG_EDIT,
        name: "SVG Editor",
        category: Category::Graphics,
        summary: "SVG 美化 / 压缩，结果区可预览。完整路径编辑器 / GPU 画布未做。",
        keywords: &["svg", "vector", "minify", "beautify", "矢量"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_mode(input: &str, mode: SvgMode) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("粘贴 SVG 标记".into());
    }
    if !t.contains("<svg") && !t.contains("<SVG") {
        return Err("未检测到 <svg> 根元素".into());
    }
    let out = match mode {
        SvgMode::Beautify => beautify(t),
        SvgMode::Minify => minify(t),
    };
    Ok(vec![("SVG", out)])
}

fn minify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut space_pending = false;
    for ch in input.chars() {
        match ch {
            '<' => {
                space_pending = false;
                in_tag = true;
                out.push('<');
            }
            '>' => {
                in_tag = false;
                space_pending = false;
                out.push('>');
            }
            c if c.is_whitespace() => {
                if in_tag
                    && !out.ends_with('<')
                    && !out.ends_with(' ')
                    && !out.ends_with('=')
                {
                    space_pending = true;
                }
                // drop text-node whitespace runs between tags
            }
            c => {
                if space_pending {
                    out.push(' ');
                    space_pending = false;
                }
                out.push(c);
            }
        }
    }
    out
}

fn beautify(input: &str) -> String {
    // Cheap token indent on '<' boundaries; good enough for icons.
    let mini = minify(input);
    let mut out = String::new();
    let mut depth = 0i32;
    let bytes = mini.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            let closing = i + 1 < bytes.len() && bytes[i + 1] == b'/';
            let mut j = i;
            while j < bytes.len() && bytes[j] != b'>' {
                j += 1;
            }
            if j >= bytes.len() {
                out.push_str(&mini[i..]);
                break;
            }
            let tag = &mini[i..=j];
            let self_close = tag.ends_with("/>") || tag.starts_with("<?") || tag.starts_with("<!");
            if closing {
                depth = (depth - 1).max(0);
            }
            for _ in 0..depth {
                out.push_str("  ");
            }
            out.push_str(tag);
            out.push('\n');
            if !closing && !self_close {
                depth += 1;
            }
            i = j + 1;
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let text = mini[start..i].trim();
            if !text.is_empty() {
                for _ in 0..depth {
                    out.push_str("  ");
                }
                out.push_str(text);
                out.push('\n');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minify_strips_gaps() {
        let rows = convert_mode("<svg>  <rect /> </svg>", SvgMode::Minify).unwrap();
        assert!(!rows[0].1.contains('\n'));
        assert!(rows[0].1.contains("<svg>"));
    }

    #[test]
    fn beautify_indents() {
        let rows = convert_mode("<svg><g><rect/></g></svg>", SvgMode::Beautify).unwrap();
        assert!(rows[0].1.contains('\n'));
    }
}
