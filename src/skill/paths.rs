use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use toml_edit::DocumentMut;

pub(crate) const SKILL_DUMP_ROOT: &str = ".agents/skills";

pub(crate) fn resolve_dump_target(to: Option<&PathBuf>) -> Result<PathBuf> {
    if let Some(path) = to {
        return Ok(path.clone());
    }

    let roots = discover_project_roots()?;
    if let Some(root) = roots.workspace_root {
        return Ok(root.join(SKILL_DUMP_ROOT));
    }
    if let Some(root) = roots.crate_root {
        return Ok(root.join(SKILL_DUMP_ROOT));
    }

    bail!(
        "`skill dump` could not determine a project root from the current directory; use `--to <path>`"
    )
}

#[derive(Debug, Clone)]
struct ProjectRoots {
    workspace_root: Option<PathBuf>,
    crate_root: Option<PathBuf>,
}

fn discover_project_roots() -> Result<ProjectRoots> {
    let cwd = env::current_dir().context("failed to resolve current directory")?;
    let mut crate_root = None;
    let mut workspace_root = None;

    for dir in cwd.ancestors() {
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }

        if crate_root.is_none() {
            crate_root = Some(dir.to_path_buf());
        }
        if workspace_root.is_none() && manifest_declares_workspace(&manifest)? {
            workspace_root = Some(dir.to_path_buf());
            break;
        }
    }

    Ok(ProjectRoots {
        workspace_root,
        crate_root,
    })
}

fn manifest_declares_workspace(path: &Path) -> Result<bool> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read cargo manifest {}", path.display()))?;
    let Ok(manifest) = source.parse::<DocumentMut>() else {
        return Ok(false);
    };
    Ok(manifest.as_table().contains_key("workspace"))
}
