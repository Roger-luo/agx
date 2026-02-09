use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use super::util::{RFC_DIR, slugify};

pub(crate) fn locate_existing_rfc(selector: &str) -> Result<PathBuf> {
    let candidates = collect_rfc_candidates()?;
    if selector.chars().all(|ch| ch.is_ascii_digit()) {
        return select_rfc_by_id(selector, &candidates);
    }

    let direct_path = Path::new(selector);
    if direct_path.exists() {
        return Ok(direct_path.to_path_buf());
    }

    let in_rfc = Path::new(RFC_DIR).join(selector);
    if in_rfc.exists() {
        return Ok(in_rfc);
    }

    let in_rfc_md = Path::new(RFC_DIR).join(format!("{selector}.md"));
    if in_rfc_md.exists() {
        return Ok(in_rfc_md);
    }

    let slug = slugify(selector);
    if slug.is_empty() {
        bail!("unable to locate RFC for selector `{selector}`");
    }

    let suffix = format!("-{slug}.md");
    let matches = candidates
        .iter()
        .filter(|(name, _)| name.ends_with(&suffix) || name.contains(&slug))
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    choose_single_match(matches, selector)
}

fn collect_rfc_candidates() -> Result<Vec<(String, PathBuf)>> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(RFC_DIR).context("failed to read RFC directory")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let Some(file_name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
        else {
            continue;
        };
        if file_name == "0000-template.md" {
            continue;
        }

        candidates.push((file_name, path));
    }

    Ok(candidates)
}

fn select_rfc_by_id(selector: &str, candidates: &[(String, PathBuf)]) -> Result<PathBuf> {
    let id_match = format!("{:04}", selector.parse::<u32>()?);
    let matches = candidates
        .iter()
        .filter(|(name, _)| name.starts_with(&id_match))
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    choose_single_match(matches, selector)
}

fn choose_single_match(matches: Vec<PathBuf>, selector: &str) -> Result<PathBuf> {
    match matches.as_slice() {
        [] => bail!("unable to locate RFC for selector `{selector}`"),
        [single] => Ok(single.clone()),
        _ => {
            let list = matches
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            bail!(
                "selector `{selector}` matched multiple RFC files; use an exact path or RFC id: {list}"
            )
        }
    }
}
