use std::{fs, path::Path};

use anyhow::{Context, Result, bail};

use crate::cli::SkillNewArgs;

pub(crate) const SKILLS_ROOT: &str = ".agents/skills";

/// Initialize `.agents/skills`.
pub(crate) fn run() -> Result<()> {
    fs::create_dir_all(SKILLS_ROOT).with_context(|| format!("failed to create `{SKILLS_ROOT}`"))?;
    println!("{SKILLS_ROOT}");
    Ok(())
}

/// Create a new skill scaffold under `.agents/skills`.
pub(crate) fn run_new(args: SkillNewArgs) -> Result<()> {
    fs::create_dir_all(SKILLS_ROOT).with_context(|| format!("failed to create `{SKILLS_ROOT}`"))?;
    println!("{SKILLS_ROOT}");
    scaffold_skill(&args.name)
}

fn scaffold_skill(name: &str) -> Result<()> {
    validate_skill_name(name)?;

    let skill_dir = Path::new(SKILLS_ROOT).join(name);
    let agents_dir = skill_dir.join("agents");
    fs::create_dir_all(&agents_dir)
        .with_context(|| format!("failed to create `{}`", agents_dir.display()))?;
    println!("{}", skill_dir.display());
    println!("{}", agents_dir.display());

    let skill_file = skill_dir.join("SKILL.md");
    write_if_missing(
        &skill_file,
        &format!(
            "---\nname: {name}\ndescription: Describe what this skill does and when to use it.\n---\n\n# {title}\n",
            title = title_case(name)
        ),
    )?;

    let openai_yaml = agents_dir.join("openai.yaml");
    write_if_missing(
        &openai_yaml,
        &format!(
            "interface:\n  display_name: \"{title}\"\n  short_description: \"Describe this skill briefly\"\n  default_prompt: \"Use ${name} to help with this task.\"\n",
            title = title_case(name)
        ),
    )?;

    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        println!("{}", path.display());
        return Ok(());
    }

    fs::write(path, content).with_context(|| format!("failed to write `{}`", path.display()))?;
    println!("{}", path.display());
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

fn title_case(name: &str) -> String {
    name.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
