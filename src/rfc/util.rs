use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};

pub(crate) const RFC_DIR: &str = "rfc";
pub(crate) const TEMPLATE_PATH: &str = "rfc/0000-template.md";
pub(crate) const INITIAL_REVISION_CHANGE: &str = "Initial draft";
pub(crate) const REVISED_REVISION_CHANGE: &str = "Revised";

pub(crate) fn resolve_default_author() -> Result<String> {
    let output = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()
        .context("failed to execute `git config --get user.name`")?;

    if !output.status.success() {
        bail!("--author is required and git user.name is not configured");
    }

    let name = String::from_utf8(output.stdout)
        .context("git user.name is not valid UTF-8")?
        .trim()
        .to_owned();
    if name.is_empty() {
        bail!("--author is required and git user.name is empty");
    }

    Ok(name)
}

pub(crate) fn next_rfc_id(rfc_dir: &Path) -> Result<String> {
    let entries = fs::read_dir(rfc_dir)
        .with_context(|| format!("failed to read RFC directory {}", rfc_dir.display()))?;
    let mut max_seen = 0u32;

    for entry in entries {
        let entry = entry?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !file_name.ends_with(".md") {
            continue;
        }

        let prefix: String = file_name.chars().take(4).collect();
        if prefix.len() != 4 || !prefix.chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }

        let parsed = prefix.parse::<u32>()?;
        if parsed > max_seen {
            max_seen = parsed;
        }
    }

    Ok(format!("{:04}", max_seen + 1))
}

pub(crate) fn timestamp_now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub(crate) fn toml_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub(crate) fn slugify(input: &str) -> String {
    let mut output = String::new();
    let mut saw_dash = false;

    for ch in input.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else if ch.is_ascii_whitespace() || ch == '-' || ch == '_' {
            '-'
        } else {
            continue;
        };

        if mapped == '-' {
            if saw_dash || output.is_empty() {
                continue;
            }
            saw_dash = true;
            output.push(mapped);
        } else {
            saw_dash = false;
            output.push(mapped);
        }
    }

    while output.ends_with('-') {
        output.pop();
    }
    if output.is_empty() {
        return "untitled".to_owned();
    }

    output
}

pub(crate) fn dedupe<T: Eq + Clone>(values: &[T]) -> Vec<T> {
    let mut deduped = Vec::new();
    for value in values {
        if deduped.iter().any(|entry| entry == value) {
            continue;
        }
        deduped.push(value.clone());
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::{dedupe, slugify};

    #[test]
    fn slugify_normalizes_words() {
        assert_eq!(slugify("Hello, AGX"), "hello-agx");
        assert_eq!(slugify("with__mixed---separators"), "with-mixed-separators");
    }

    #[test]
    fn slugify_falls_back_to_untitled() {
        assert_eq!(slugify("!!!"), "untitled");
    }

    #[test]
    fn dedupe_preserves_first_seen_order() {
        let values = vec![
            "roger".to_owned(),
            "codex".to_owned(),
            "roger".to_owned(),
            "atlas".to_owned(),
            "codex".to_owned(),
        ];
        assert_eq!(
            dedupe(&values),
            vec!["roger".to_owned(), "codex".to_owned(), "atlas".to_owned()]
        );
    }

    #[test]
    fn dedupe_supports_integer_values() {
        let values = vec![4_u32, 1_u32, 4_u32, 2_u32, 1_u32];
        assert_eq!(dedupe(&values), vec![4_u32, 1_u32, 2_u32]);
    }
}
