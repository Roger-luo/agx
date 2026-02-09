use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::cli::{SkillListArgs, SkillListFormat};

use super::{
    builtin,
    catalog::{self, SkillDiscoveryEntry},
    init::SKILLS_ROOT,
};

pub(crate) fn run(args: SkillListArgs) -> Result<()> {
    let builtin_skills = builtin::load_skills()?;
    let workspace_skills = catalog::discover_workspace_skills(Path::new(SKILLS_ROOT))?;
    let entries = catalog::discover_skills(args.origin, &builtin_skills, &workspace_skills);

    match args.format {
        SkillListFormat::Text => print_text(&entries),
        SkillListFormat::Json => print_json(&entries)?,
    }
    Ok(())
}

fn print_text(entries: &[SkillDiscoveryEntry]) {
    println!("name\tpreferred_origin\tbuiltin_available\tworkspace_path\tdescription");
    for entry in entries {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            entry.name,
            origin_to_text(&entry.preferred_origin),
            entry.builtin_available,
            entry.workspace_path.as_deref().unwrap_or("-"),
            entry.description
        );
    }
}

fn print_json(entries: &[SkillDiscoveryEntry]) -> Result<()> {
    let payload = SkillListResponseJson {
        schema_version: 1,
        skills: entries.to_vec(),
    };
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn origin_to_text(origin: &catalog::PreferredOrigin) -> &'static str {
    match origin {
        catalog::PreferredOrigin::Builtin => "builtin",
        catalog::PreferredOrigin::Workspace => "workspace",
    }
}

#[derive(Debug, Serialize)]
struct SkillListResponseJson {
    schema_version: u32,
    skills: Vec<SkillDiscoveryEntry>,
}
