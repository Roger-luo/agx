use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::cli::SkillValidateArgs;
use crate::output;

use super::{
    init::SKILLS_ROOT,
    metadata::{ensure_optional_openai_yaml_valid, read_skill_metadata},
};

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
        output::print_log(format!("ok {}", skill.display()));
    }

    if failures.is_empty() {
        output::print_log(format!("validated {} skill(s)", skills.len()));
        return Ok(());
    }

    for failure in failures {
        output::print_error(failure);
    }
    bail!("skill validation failed")
}

pub(crate) fn discover_skill_paths(target: &Path) -> Result<Vec<PathBuf>> {
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

    skills.sort();
    Ok(skills)
}

fn validate_skill(skill_path: &Path) -> Result<()> {
    let metadata = read_skill_metadata(skill_path)?;

    let folder_name = skill_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid skill directory name"))?;
    if folder_name != metadata.name {
        bail!(
            "skill folder `{folder_name}` does not match frontmatter name `{}`",
            metadata.name
        );
    }

    ensure_optional_openai_yaml_valid(skill_path)?;
    Ok(())
}
