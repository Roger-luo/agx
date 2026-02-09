use std::{
    fs::{self, File},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use flate2::{Compression, write::GzEncoder};
use tar::Builder;

use crate::cli::SkillExportArgs;
use crate::output;

use super::builtin;

pub(crate) fn run(args: SkillExportArgs) -> Result<()> {
    let _origin = args.origin;
    let skills = builtin::load_skills()?;
    if skills.is_empty() {
        bail!("no built-in skills are available to export");
    }

    if let Some(parent) = args.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create `{}`", parent.display()))?;
        }
    }

    let archive_file = File::create(&args.output)
        .with_context(|| format!("failed to create `{}`", args.output.display()))?;
    let encoder = GzEncoder::new(archive_file, Compression::default());
    let mut builder = Builder::new(encoder);

    for skill in &skills {
        for file in &skill.files {
            let archive_path = resolve_archive_path(&skill.name, &file.path)?;
            append_archive_file(&mut builder, &archive_path, file.content.as_bytes())?;
        }
    }

    let encoder = builder
        .into_inner()
        .context("failed to finalize skills tar archive")?;
    encoder
        .finish()
        .context("failed to finalize skills gzip stream")?;

    output::print_path(args.output.display());
    Ok(())
}

fn resolve_archive_path(skill_name: &str, relative_path: &str) -> Result<PathBuf> {
    let relative = Path::new(relative_path);
    if relative.is_absolute() {
        bail!("skill file path `{relative_path}` must be relative");
    }
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            bail!("skill file path `{relative_path}` must not contain traversal components");
        }
    }

    Ok(Path::new(".agents/skills").join(skill_name).join(relative))
}

fn append_archive_file(
    builder: &mut Builder<GzEncoder<File>>,
    path: &Path,
    bytes: &[u8],
) -> Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, bytes)
        .with_context(|| format!("failed to append `{}` to archive", path.display()))?;
    Ok(())
}
