use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use super::builtin::BuiltinSkill;

#[derive(Debug, Clone)]
pub(crate) struct MaterializedSkill {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
}

pub(crate) fn materialize_skills(
    skills: &[BuiltinSkill],
    target_root: &Path,
    force: bool,
) -> Result<Vec<MaterializedSkill>> {
    preflight_materialize(skills, target_root, force)?;
    fs::create_dir_all(target_root)
        .with_context(|| format!("failed to create `{}`", target_root.display()))?;

    let mut materialized = Vec::with_capacity(skills.len());
    for skill in skills {
        let skill_dir = target_root.join(&skill.name);
        fs::create_dir_all(&skill_dir)
            .with_context(|| format!("failed to create `{}`", skill_dir.display()))?;

        for file in &skill.files {
            let file_path = resolve_skill_file_destination(&skill_dir, &file.path)?;
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create `{}`", parent.display()))?;
            }
            fs::write(&file_path, &file.content)
                .with_context(|| format!("failed to write `{}`", file_path.display()))?;
        }

        materialized.push(MaterializedSkill {
            name: skill.name.clone(),
            path: skill_dir,
        });
    }

    Ok(materialized)
}

fn preflight_materialize(skills: &[BuiltinSkill], target_root: &Path, force: bool) -> Result<()> {
    let mut conflicts = Vec::new();

    for skill in skills {
        let skill_dir = target_root.join(&skill.name);
        if skill_dir.exists() {
            if !skill_dir.is_dir() {
                conflicts.push(format!(
                    "target path `{}` exists and is not a directory",
                    skill_dir.display()
                ));
            } else if !force {
                conflicts.push(format!(
                    "target skill `{}` already exists at `{}` (use --force to overwrite)",
                    skill.name,
                    skill_dir.display()
                ));
            }
        }

        for file in &skill.files {
            let destination = resolve_skill_file_destination(&skill_dir, &file.path)?;
            if !force && destination.exists() {
                conflicts.push(format!(
                    "target file `{}` already exists (use --force to overwrite)",
                    destination.display()
                ));
            }
        }
    }

    if conflicts.is_empty() {
        return Ok(());
    }
    bail!(conflicts.join("\n"))
}

fn resolve_skill_file_destination(skill_dir: &Path, relative_path: &str) -> Result<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.is_absolute() {
        bail!("skill file path `{relative_path}` must be relative");
    }
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            bail!("skill file path `{relative_path}` must not contain traversal components");
        }
    }
    Ok(skill_dir.join(relative))
}
