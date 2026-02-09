use std::{fs, path::Path};

use crate::output;
use anyhow::{Context, Result, bail};

const RFC_DIR: &str = "rfc";
const SKILLS_ROOT: &str = ".agents/skills";
const TEMPLATE_PATH: &str = "rfc/0000-template.md";

use super::template::embedded_template;

/// Initialize RFC project directory.
///
/// This command requires an existing `.agents/skills` directory so skill
/// materialization remains explicit (`agx skill dump --all`).
pub(crate) fn run() -> Result<()> {
    let skills_root = Path::new(SKILLS_ROOT);
    if !skills_root.exists() {
        bail!(
            "`{SKILLS_ROOT}` does not exist; run `agx skill dump --all` to materialize built-in skills in this project"
        );
    }
    if !skills_root.is_dir() {
        bail!(
            "`{SKILLS_ROOT}` exists but is not a directory; fix this path and rerun `agx skill dump --all`"
        );
    }

    fs::create_dir_all(RFC_DIR).with_context(|| format!("failed to create `{RFC_DIR}`"))?;
    write_template_if_missing()?;
    output::print_path(RFC_DIR);
    Ok(())
}

fn write_template_if_missing() -> Result<()> {
    let template_path = Path::new(TEMPLATE_PATH);
    if template_path.exists() {
        return Ok(());
    }

    fs::write(template_path, embedded_template())
        .with_context(|| format!("failed to write `{}`", template_path.display()))?;
    Ok(())
}
