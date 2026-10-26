//! docker run → docker-compose.yml (heuristic).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::DOCKER_RUN,
        name: "Docker run → Compose",
        category: Category::Data,
        summary: "把 `docker run ...` 粗转为 docker-compose 片段（启发式）。",
        keywords: &["docker", "compose", "run", "容器"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("粘贴 docker run 命令".into());
    }
    let tokens = tokenize(t);
    if tokens.is_empty() {
        return Err("无法解析命令".into());
    }
    let mut i = 0;
    if tokens[0] == "docker" {
        i = 1;
    }
    if i < tokens.len() && tokens[i] == "run" {
        i += 1;
    }
    let mut name = None;
    let mut image = None;
    let mut ports = Vec::new();
    let mut volumes = Vec::new();
    let mut env = Vec::new();
    let mut restart = None;
    let mut detached = false;
    let mut command = Vec::new();
    while i < tokens.len() {
        let tok = &tokens[i];
        match tok.as_str() {
            "-d" | "--detach" => {
                detached = true;
                i += 1;
            }
            "--name" => {
                name = tokens.get(i + 1).cloned();
                i += 2;
            }
            "-p" | "--publish" => {
                if let Some(p) = tokens.get(i + 1) {
                    ports.push(p.clone());
                }
                i += 2;
            }
            "-v" | "--volume" => {
                if let Some(v) = tokens.get(i + 1) {
                    volumes.push(v.clone());
                }
                i += 2;
            }
            "-e" | "--env" => {
                if let Some(e) = tokens.get(i + 1) {
                    env.push(e.clone());
                }
                i += 2;
            }
            "--restart" => {
                restart = tokens.get(i + 1).cloned();
                i += 2;
            }
            _ if tok.starts_with('-') => {
                // skip unknown flag (± value)
                i += 1;
                if i < tokens.len() && !tokens[i].starts_with('-') && image.is_none() {
                    // might be flag value; peek — if next looks like image later ok
                    if !tokens[i].contains('/') && !tokens[i].contains(':') && tokens[i].len() < 3 {
                        i += 1;
                    }
                }
            }
            _ => {
                if image.is_none() {
                    image = Some(tok.clone());
                    i += 1;
                } else {
                    command.push(tok.clone());
                    i += 1;
                }
            }
        }
    }
    let service = name.unwrap_or_else(|| "app".into());
    let image = image.ok_or_else(|| "未找到镜像名".to_string())?;
    let mut yaml = format!("services:\n  {service}:\n    image: {image}\n");
    if detached {
        // compose has no -d; noted
    }
    if let Some(r) = restart {
        yaml.push_str(&format!("    restart: {r}\n"));
    }
    if !ports.is_empty() {
        yaml.push_str("    ports:\n");
        for p in &ports {
            yaml.push_str(&format!("      - \"{p}\"\n"));
        }
    }
    if !volumes.is_empty() {
        yaml.push_str("    volumes:\n");
        for v in &volumes {
            yaml.push_str(&format!("      - {v}\n"));
        }
    }
    if !env.is_empty() {
        yaml.push_str("    environment:\n");
        for e in &env {
            if let Some((k, v)) = e.split_once('=') {
                yaml.push_str(&format!("      {k}: \"{v}\"\n"));
            } else {
                yaml.push_str(&format!("      - {e}\n"));
            }
        }
    }
    if !command.is_empty() {
        yaml.push_str(&format!(
            "    command: {}\n",
            command.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(" ")
        ));
    }
    Ok(vec![
        ("docker-compose.yml", yaml),
        ("服务名", service),
        ("镜像", image),
    ])
}

fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            ' ' | '\t' | '\n' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            '"' | '\'' => {
                let quote = ch;
                for c in chars.by_ref() {
                    if c == quote {
                        break;
                    }
                    cur.push(c);
                }
                out.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typical_run() {
        let rows = convert_all(
            "docker run -d --name web -p 8080:80 -e FOO=bar nginx:alpine",
        )
        .unwrap();
        assert!(rows[0].1.contains("image: nginx:alpine"));
        assert!(rows[0].1.contains("8080:80"));
        assert!(rows[0].1.contains("FOO"));
    }
}
