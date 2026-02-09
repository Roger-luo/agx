use anyhow::Result;

use crate::cli::SkillDumpArgs;
use crate::output;

use super::{builtin, materialize, paths, select};

pub(crate) fn run(args: SkillDumpArgs) -> Result<()> {
    let skills = builtin::load_skills()?;
    let selected = select::select_builtin_skills(&skills, args.name.as_deref(), args.all)?;
    let target_root = paths::resolve_dump_target(args.to.as_ref())?;
    let materialized = materialize::materialize_skills(&selected, &target_root, args.force)?;

    for skill in materialized {
        output::print_path(skill.path.display());
    }
    Ok(())
}
