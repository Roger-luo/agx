use anyhow::{Context, Result};
use serde::Deserialize;

const BUILTIN_CATALOG_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/builtin_skills.json"));

#[derive(Debug, Clone)]
pub(crate) struct BuiltinSkill {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) files: Vec<BuiltinSkillFile>,
}

#[derive(Debug, Clone)]
pub(crate) struct BuiltinSkillFile {
    pub(crate) path: String,
    pub(crate) content: String,
}

pub(crate) fn load_skills() -> Result<Vec<BuiltinSkill>> {
    let catalog: BuiltinCatalogJson = serde_json::from_str(BUILTIN_CATALOG_JSON)
        .context("failed to decode embedded builtin skill catalog")?;
    Ok(catalog
        .skills
        .into_iter()
        .map(|skill| BuiltinSkill {
            name: skill.name,
            description: skill.description,
            files: skill
                .files
                .into_iter()
                .map(|file| BuiltinSkillFile {
                    path: file.path,
                    content: file.content,
                })
                .collect(),
        })
        .collect())
}

#[derive(Debug, Deserialize)]
struct BuiltinCatalogJson {
    #[allow(dead_code)]
    schema_version: u32,
    skills: Vec<BuiltinSkillJson>,
}

#[derive(Debug, Deserialize)]
struct BuiltinSkillJson {
    name: String,
    description: String,
    files: Vec<BuiltinSkillFileJson>,
}

#[derive(Debug, Deserialize)]
struct BuiltinSkillFileJson {
    path: String,
    content: String,
}
