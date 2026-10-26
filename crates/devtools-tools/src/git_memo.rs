//! Static git cheatsheet.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::GIT_MEMO,
        name: "Git memo",
        category: Category::Text,
        summary: "常用 Git 命令速查。输入关键词过滤；留空显示全部。",
        keywords: &["git", "memo", "cheatsheet", "速查"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const ENTRIES: &[(&str, &str)] = &[
    ("clone", "git clone <url>"),
    ("status", "git status"),
    ("add", "git add <path> | git add -p"),
    ("commit", "git commit -m \"msg\""),
    ("commit amend", "git commit --amend --no-edit"),
    ("log", "git log --oneline --graph -n 20"),
    ("diff", "git diff | git diff --staged"),
    ("branch", "git branch -vv | git switch -c <name>"),
    ("checkout", "git switch <branch>"),
    ("merge", "git merge <branch>"),
    ("rebase", "git rebase <branch> | git rebase -i HEAD~3"),
    ("remote", "git remote -v | git push -u origin HEAD"),
    ("pull", "git pull --rebase"),
    ("push", "git push | git push --force-with-lease"),
    ("stash", "git stash push -m msg | git stash pop"),
    ("reset soft", "git reset --soft HEAD~1"),
    ("reset hard", "git reset --hard HEAD"),
    ("restore", "git restore <path> | git restore --staged <path>"),
    ("tag", "git tag -a v1.0 -m msg | git push --tags"),
    ("cherry-pick", "git cherry-pick <sha>"),
    ("blame", "git blame <path>"),
    ("clean", "git clean -fd"),
    ("config user", "git config user.name / user.email"),
    ("ignore", "编辑 .gitignore 后 git rm -r --cached ."),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let q = input.trim().to_ascii_lowercase();
    let hits: Vec<_> = ENTRIES
        .iter()
        .filter(|(k, v)| q.is_empty() || k.contains(&q) || v.to_ascii_lowercase().contains(&q))
        .collect();
    if hits.is_empty() {
        return Err("没有匹配的条目".into());
    }
    Ok(hits
        .into_iter()
        .take(30)
        .map(|(k, v)| (*k, (*v).to_string()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn filter_commit() {
        let rows = convert_all("commit").unwrap();
        assert!(rows.iter().any(|r| r.1.contains("commit")));
    }
}
