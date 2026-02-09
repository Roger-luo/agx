use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::cli::SkillListOrigin;

use super::{
    builtin::BuiltinSkill,
    metadata::{ensure_optional_openai_yaml_valid, read_skill_metadata},
};

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceSkill {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PreferredOrigin {
    Builtin,
    Workspace,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillDiscoveryEntry {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) builtin_available: bool,
    pub(crate) workspace_path: Option<String>,
    pub(crate) preferred_origin: PreferredOrigin,
}

pub(crate) fn discover_workspace_skills(skills_root: &Path) -> Result<Vec<WorkspaceSkill>> {
    if !skills_root.exists() {
        return Ok(Vec::new());
    }
    if !skills_root.is_dir() {
        bail!(
            "expected workspace skills root directory `{}`",
            skills_root.display()
        );
    }

    let mut skill_dirs = Vec::new();
    for entry in fs::read_dir(skills_root)
        .with_context(|| format!("failed to read `{}`", skills_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            skill_dirs.push(path);
        }
    }
    skill_dirs.sort();

    let mut skills = Vec::with_capacity(skill_dirs.len());
    for skill_path in skill_dirs {
        let metadata = read_skill_metadata(&skill_path)?;
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
        ensure_optional_openai_yaml_valid(&skill_path)?;

        skills.push(WorkspaceSkill {
            name: metadata.name,
            description: metadata.description,
            path: skill_path,
        });
    }

    Ok(skills)
}

pub(crate) fn discover_skills(
    origin: SkillListOrigin,
    builtin_skills: &[BuiltinSkill],
    workspace_skills: &[WorkspaceSkill],
) -> Vec<SkillDiscoveryEntry> {
    match origin {
        SkillListOrigin::Builtin => builtin_skills
            .iter()
            .map(|skill| SkillDiscoveryEntry {
                name: skill.name.clone(),
                description: skill.description.clone(),
                builtin_available: true,
                workspace_path: None,
                preferred_origin: PreferredOrigin::Builtin,
            })
            .collect(),
        SkillListOrigin::Workspace => {
            let builtin = builtin_index(builtin_skills);
            workspace_skills
                .iter()
                .map(|skill| SkillDiscoveryEntry {
                    name: skill.name.clone(),
                    description: skill.description.clone(),
                    builtin_available: builtin.contains_key(&skill.name),
                    workspace_path: Some(path_to_string(&skill.path)),
                    preferred_origin: PreferredOrigin::Workspace,
                })
                .collect()
        }
        SkillListOrigin::All => {
            let mut index = BTreeMap::<String, SkillDiscoveryEntry>::new();
            for skill in builtin_skills {
                index.insert(
                    skill.name.clone(),
                    SkillDiscoveryEntry {
                        name: skill.name.clone(),
                        description: skill.description.clone(),
                        builtin_available: true,
                        workspace_path: None,
                        preferred_origin: PreferredOrigin::Builtin,
                    },
                );
            }
            for skill in workspace_skills {
                let builtin_available = index.contains_key(&skill.name);
                index.insert(
                    skill.name.clone(),
                    SkillDiscoveryEntry {
                        name: skill.name.clone(),
                        description: skill.description.clone(),
                        builtin_available,
                        workspace_path: Some(path_to_string(&skill.path)),
                        preferred_origin: PreferredOrigin::Workspace,
                    },
                );
            }

            index.into_values().collect()
        }
    }
}

fn builtin_index(skills: &[BuiltinSkill]) -> BTreeMap<String, &BuiltinSkill> {
    skills
        .iter()
        .map(|skill| (skill.name.clone(), skill))
        .collect()
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
