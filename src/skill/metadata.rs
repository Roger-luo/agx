use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub(crate) struct SkillMetadata {
    pub(crate) name: String,
    pub(crate) description: String,
}

pub(crate) fn read_skill_metadata(skill_path: &Path) -> Result<SkillMetadata> {
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

    let description = metadata
        .get("description")
        .ok_or_else(|| anyhow::anyhow!("missing required `description` in frontmatter"))?;
    if description.trim().is_empty() {
        bail!("frontmatter `description` cannot be empty");
    }

    Ok(SkillMetadata {
        name: name.clone(),
        description: description.clone(),
    })
}

pub(crate) fn ensure_optional_openai_yaml_valid(skill_path: &Path) -> Result<()> {
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

pub(crate) fn validate_skill_name(name: &str) -> Result<()> {
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
