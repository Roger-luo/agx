use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use crate::cli::{SkillInstallArgs, SkillInstallFormat};
use crate::output;

use super::{builtin, init::SKILLS_ROOT, materialize, select};

pub(crate) fn run(args: SkillInstallArgs) -> Result<()> {
    let _origin = args.origin;
    let skills = builtin::load_skills()?;
    let selected = select::select_builtin_skills(&skills, args.name.as_deref(), args.all)?;
    let target_root = args.to.unwrap_or_else(|| PathBuf::from(SKILLS_ROOT));
    let installed = materialize::materialize_skills(&selected, &target_root, args.force)?;

    match args.format {
        SkillInstallFormat::Text => {
            for skill in installed {
                let line = format!("{}\t{}", skill.name, skill.path.display());
                output::print_log(line);
            }
        }
        SkillInstallFormat::Json => {
            let payload = SkillInstallResponseJson {
                schema_version: 1,
                installed: installed
                    .into_iter()
                    .map(|item| InstalledSkillJson {
                        name: item.name,
                        path: item.path.to_string_lossy().into_owned(),
                    })
                    .collect(),
            };
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct SkillInstallResponseJson {
    schema_version: u32,
    installed: Vec<InstalledSkillJson>,
}

#[derive(Debug, Serialize)]
struct InstalledSkillJson {
    name: String,
    path: String,
}
