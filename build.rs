use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Serialize;
use toml_edit::{Array, DocumentMut, Item};

const BUILTIN_MANIFEST: &str = ".agents/skills/builtin-manifest.toml";

fn main() {
    if let Err(error) = run() {
        panic!("failed to generate builtin skills catalog: {error:#}");
    }
}

fn run() -> Result<()> {
    println!("cargo:rerun-if-changed={BUILTIN_MANIFEST}");

    let manifest_path = Path::new(BUILTIN_MANIFEST);
    let manifest = load_manifest(manifest_path)?;

    let mut catalog_skills = Vec::with_capacity(manifest.len());
    for name in manifest {
        validate_skill_name(&name)?;
        let skill_root = Path::new(".agents/skills").join(&name);
        println!("cargo:rerun-if-changed={}", skill_root.display());

        let skill = read_skill_definition(&name, &skill_root)?;
        catalog_skills.push(skill);
    }

    let catalog = BuiltinCatalogJson {
        schema_version: 1,
        skills: catalog_skills,
    };

    let out_dir = PathBuf::from(env::var("OUT_DIR").context("OUT_DIR is not set")?);
    let out_path = out_dir.join("builtin_skills.json");
    let encoded = serde_json::to_string(&catalog).context("failed to serialize builtin skills")?;
    fs::write(&out_path, encoded)
        .with_context(|| format!("failed to write `{}`", out_path.display()))?;
    println!("cargo:rerun-if-changed=.agents/skills");

    Ok(())
}

fn load_manifest(manifest_path: &Path) -> Result<Vec<String>> {
    let source = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read `{}`", manifest_path.display()))?;
    let document = source
        .parse::<DocumentMut>()
        .with_context(|| format!("failed to parse `{}`", manifest_path.display()))?;

    let skills = match document.get("skills") {
        Some(item) => parse_skills_array(item)?,
        None => bail!("`{}` must define a `skills` array", manifest_path.display()),
    };
    if skills.is_empty() {
        bail!(
            "`{}` must include at least one skill",
            manifest_path.display()
        );
    }
    Ok(skills)
}

fn parse_skills_array(item: &Item) -> Result<Vec<String>> {
    let Some(values) = item.as_array() else {
        bail!("manifest key `skills` must be an array of strings");
    };

    extract_array_strings(values)
}

fn extract_array_strings(values: &Array) -> Result<Vec<String>> {
    let mut output = Vec::with_capacity(values.len());
    for value in values {
        let Some(name) = value.as_str() else {
            bail!("manifest `skills` entries must be strings");
        };
        output.push(name.to_owned());
    }
    Ok(output)
}

fn read_skill_definition(name: &str, skill_root: &Path) -> Result<BuiltinSkillJson> {
    if !skill_root.is_dir() {
        bail!(
            "manifest skill `{name}` points to missing directory `{}`",
            skill_root.display()
        );
    }

    let skill_md = skill_root.join("SKILL.md");
    let skill_source = fs::read_to_string(&skill_md)
        .with_context(|| format!("failed to read `{}`", skill_md.display()))?;
    let metadata = parse_skill_metadata(&skill_source)?;

    let parsed_name = metadata
        .get("name")
        .ok_or_else(|| anyhow::anyhow!("missing required `name` in frontmatter"))?;
    if parsed_name != name {
        bail!("manifest entry `{name}` does not match SKILL.md frontmatter `name: {parsed_name}`");
    }

    let description = metadata
        .get("description")
        .ok_or_else(|| anyhow::anyhow!("missing required `description` in frontmatter"))?;
    if description.trim().is_empty() {
        bail!("skill `{name}` frontmatter `description` cannot be empty");
    }

    let openai_yaml = skill_root.join("agents/openai.yaml");
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

    let mut files = Vec::new();
    collect_skill_files(skill_root, skill_root, &mut files)?;
    if files.is_empty() {
        bail!("skill `{name}` has no files to package");
    }

    Ok(BuiltinSkillJson {
        name: name.to_owned(),
        description: description.to_owned(),
        files,
    })
}

fn parse_skill_metadata(source: &str) -> Result<HashMap<String, String>> {
    let frontmatter = extract_frontmatter(source)?;
    let metadata = parse_frontmatter_map(frontmatter)?;
    validate_frontmatter_keys(&metadata)?;
    Ok(metadata)
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

fn collect_skill_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<BuiltinSkillFileJson>,
) -> Result<()> {
    let mut entries = fs::read_dir(current)
        .with_context(|| format!("failed to read `{}`", current.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("failed to read `{}`", current.display()))?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_skill_files(root, &path, files)?;
            continue;
        }
        if !path.is_file() {
            continue;
        }

        println!("cargo:rerun-if-changed={}", path.display());
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("failed to resolve relative path for `{}`", path.display()))?;
        let relative_path = relative
            .iter()
            .map(|component| component.to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read `{}` as UTF-8 text", path.display()))?;
        files.push(BuiltinSkillFileJson {
            path: relative_path,
            content,
        });
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

#[derive(Debug, Serialize)]
struct BuiltinCatalogJson {
    schema_version: u32,
    skills: Vec<BuiltinSkillJson>,
}

#[derive(Debug, Serialize)]
struct BuiltinSkillJson {
    name: String,
    description: String,
    files: Vec<BuiltinSkillFileJson>,
}

#[derive(Debug, Serialize)]
struct BuiltinSkillFileJson {
    path: String,
    content: String,
}
