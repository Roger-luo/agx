use anyhow::{Result, bail};

use super::builtin::BuiltinSkill;

pub(crate) fn select_builtin_skills(
    skills: &[BuiltinSkill],
    name: Option<&str>,
    all: bool,
) -> Result<Vec<BuiltinSkill>> {
    if all && name.is_some() {
        bail!("cannot pass both a skill name and `--all`");
    }
    if !all && name.is_none() {
        bail!("provide a skill name or pass `--all`");
    }

    if all {
        return Ok(skills.to_vec());
    }

    let name = name.expect("name is checked to exist");
    if let Some(skill) = skills.iter().find(|skill| skill.name == name) {
        return Ok(vec![skill.clone()]);
    }

    let known = skills
        .iter()
        .map(|skill| skill.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    bail!("unknown builtin skill `{name}`; known skills: {known}")
}
