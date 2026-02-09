//! Template and project-root discovery.
//!
//! Root resolution precedence:
//! 1. Workspace root (ancestor `Cargo.toml` with `[workspace]`)
//! 2. Crate root (nearest ancestor `Cargo.toml`)
//! 3. Current directory fallback

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use toml_edit::DocumentMut;

use super::util::{RFC_DIR, TEMPLATE_PATH};

const DEFAULT_TEMPLATE: &str = include_str!("../../rfc/0000-template.md");

/// Cargo project roots discovered from the current working directory.
#[derive(Debug, Clone)]
pub(crate) struct ProjectRoots {
    pub(crate) workspace_root: Option<PathBuf>,
    pub(crate) crate_root: Option<PathBuf>,
}

/// Load template text from the project template path when available, otherwise
/// fall back to the embedded default template shipped with the binary.
pub(crate) fn load_template() -> Result<String> {
    let Some(template_path) = resolve_project_template_path()? else {
        return Ok(DEFAULT_TEMPLATE.to_owned());
    };

    fs::read_to_string(&template_path).with_context(|| {
        format!(
            "failed to read template file at {}",
            template_path.display()
        )
    })
}

/// Return the embedded RFC template shipped in the binary.
pub(crate) fn embedded_template() -> &'static str {
    DEFAULT_TEMPLATE
}

/// Resolve the RFC directory used for title-based metadata reference lookup.
pub(crate) fn resolve_project_rfc_dir() -> Result<PathBuf> {
    let roots = discover_project_roots()?;
    if let Some(root) = roots.workspace_root.as_ref() {
        return Ok(root.join(RFC_DIR));
    }
    if let Some(root) = roots.crate_root.as_ref() {
        return Ok(root.join(RFC_DIR));
    }

    let cwd = env::current_dir().context("failed to resolve current directory")?;
    Ok(cwd.join(RFC_DIR))
}

fn resolve_project_template_path() -> Result<Option<PathBuf>> {
    let roots = discover_project_roots()?;

    if let Some(root) = roots.workspace_root.as_ref() {
        let candidate = root.join(TEMPLATE_PATH);
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    if let Some(root) = roots.crate_root.as_ref() {
        if roots.workspace_root.as_ref() == Some(root) {
            return Ok(None);
        }

        let candidate = root.join(TEMPLATE_PATH);
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    Ok(None)
}

/// Discover crate/workspace roots by traversing ancestors from the current
/// working directory.
pub(crate) fn discover_project_roots() -> Result<ProjectRoots> {
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

#[cfg(test)]
mod tests {
    use super::manifest_declares_workspace;
    use std::{fs, time::SystemTime};

    #[test]
    fn workspace_manifest_is_detected() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agx-template-test-{}-workspace",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).expect("failed to create temp dir");

        let manifest = temp_dir.join("Cargo.toml");
        fs::write(
            &manifest,
            "[workspace]\nmembers = [\"crates/member\"]\nresolver = \"2\"\n",
        )
        .expect("failed to write manifest");

        let has_workspace = manifest_declares_workspace(&manifest).expect("failed to parse");
        assert!(has_workspace);

        fs::remove_dir_all(temp_dir).expect("failed to clean temp dir");
    }
}
