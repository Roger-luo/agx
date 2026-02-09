use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::cli::SkillValidateArgs;

use super::init::SKILLS_ROOT;

/// Validate one skill or all skills under a skills root directory.
pub(crate) fn run(args: SkillValidateArgs) -> Result<()> {
    let target = args
        .name
        .as_deref()
        .map(|name| PathBuf::from(SKILLS_ROOT).join(name))
        .unwrap_or_else(|| PathBuf::from(SKILLS_ROOT));
    let skills = discover_skill_paths(&target)?;

    let mut failures = Vec::new();
    for skill in &skills {
        if let Err(error) = validate_skill(skill) {
            failures.push(format!("{}: {error:#}", skill.display()));
            continue;
        }
        println!("ok {}", skill.display());
    }

    if failures.is_empty() {
        println!("validated {} skill(s)", skills.len());
        return Ok(());
    }

    for failure in failures {
        eprintln!("error: {failure}");
    }
    bail!("skill validation failed")
}

fn discover_skill_paths(target: &Path) -> Result<Vec<PathBuf>> {
    if target.join("SKILL.md").is_file() {
        return Ok(vec![target.to_path_buf()]);
    }

    if !target.is_dir() {
        bail!(
            "expected a skill directory or skills root directory, found `{}`",
            target.display()
        );
    }

    let mut skills = Vec::new();
    for entry in
        fs::read_dir(target).with_context(|| format!("failed to read `{}`", target.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.join("SKILL.md").is_file() {
            skills.push(path);
        }
    }

    if skills.is_empty() {
        bail!("no skills found under `{}`", target.display());
    }
    Ok(skills)
}

fn validate_skill(skill_path: &Path) -> Result<()> {
    let skill_md_path = skill_path.join("SKILL.md");
    let source = fs::read_to_string(&skill_md_path)
        .with_context(|| format!("failed to read `{}`", skill_md_path.display()))?;
    let frontmatter = extract_frontmatter(&source)?;
    let metadata = parse_frontmatter_map(frontmatter)?;

    validate_frontmatter_keys(&metadata)?;

    let name = metadata
        .get("name")
        .ok_or_else(|| anyhow::anyhow!("missing required `name` in frontmatter"))?;
    validate_skill_name(name)?;

    let folder_name = skill_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid skill directory name"))?;
    if folder_name != name {
        bail!("skill folder `{folder_name}` does not match frontmatter name `{name}`");
    }

    let description = metadata
        .get("description")
        .ok_or_else(|| anyhow::anyhow!("missing required `description` in frontmatter"))?;
    if description.trim().is_empty() {
        bail!("frontmatter `description` cannot be empty");
    }

    let openai_yaml = skill_path.join("agents/openai.yaml");
    if openai_yaml.exists() {
        let openai_text = fs::read_to_string(&openai_yaml)
            .with_context(|| format!("failed to read `{}`", openai_yaml.display()))?;
        if !openai_text.contains("interface:") {
            bail!(
                "`{}` exists but does not contain `interface:`",
                openai_yaml.display()
            );
        }
    }

    Ok(())
}

fn extract_frontmatter(source: &str) -> Result<&str> {
    if !source.starts_with("---\n") {
        bail!("SKILL.md must start with YAML frontmatter marker `---`");
    }

    let rest = &source[4..];
    if let Some(end) = rest.find("\n---\n") {
        return Ok(&rest[..end]);
    }
    if let Some(end) = rest.find("\n---") {
        return Ok(&rest[..end]);
    }

    bail!("SKILL.md is missing closing YAML frontmatter marker `---`")
}

fn parse_frontmatter_map(frontmatter: &str) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (index, raw_line) in frontmatter.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once(':') else {
            bail!("invalid frontmatter line {}: `{}`", index + 1, raw_line);
        };
        let key = raw_key.trim();
        let value = raw_value.trim();
        if key.is_empty() {
            bail!("invalid frontmatter line {}: empty key", index + 1);
        }
        if value.is_empty() {
            bail!("invalid frontmatter line {}: empty value", index + 1);
        }

        let value = value.trim_matches('"').trim_matches('\'').trim().to_owned();
        map.insert(key.to_owned(), value);
    }

    Ok(map)
}

fn validate_frontmatter_keys(metadata: &HashMap<String, String>) -> Result<()> {
    for key in metadata.keys() {
        if key == "name" || key == "description" {
            continue;
        }
        bail!("unexpected frontmatter key `{key}`; allowed keys are `name` and `description`");
    }
    Ok(())
}

fn validate_skill_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 63 {
        bail!("skill name must be between 1 and 63 characters");
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        bail!("skill name must not start/end with `-` or contain consecutive `-`");
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        bail!("skill name must contain only lowercase letters, digits, and `-`");
    }
    Ok(())
}
