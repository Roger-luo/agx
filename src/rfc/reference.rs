//! Metadata reference resolution.
//!
//! `prerequisite`, `supersedes`, and `superseded_by` accept mixed inputs:
//! direct RFC ids or RFC titles. This module normalizes them into integer id
//! lists for metadata output.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use toml_edit::{DocumentMut, Item};

use crate::cli::{RfcEditArgs, RfcReference};

use super::{
    template::resolve_project_rfc_dir,
    util::{dedupe, slugify},
};

/// Integer-only metadata references ready for template rendering or TOML edit.
pub(crate) struct ResolvedMetadataReferences {
    pub(crate) prerequisite: Vec<u32>,
    pub(crate) supersedes: Vec<u32>,
    pub(crate) superseded_by: Vec<u32>,
}

/// Resolve all metadata references on the CLI into RFC ids.
///
/// Title references are resolved against RFC files under the project RFC
/// directory (workspace root first, then crate root).
pub(crate) fn resolve_metadata_references(cli: &RfcEditArgs) -> Result<ResolvedMetadataReferences> {
    let needs_title_lookup = [&cli.prerequisite, &cli.supersedes, &cli.superseded_by]
        .into_iter()
        .flatten()
        .any(|reference| matches!(reference, RfcReference::Title(_)));
    let title_index = if needs_title_lookup {
        Some(RfcTitleIndex::load()?)
    } else {
        None
    };

    Ok(ResolvedMetadataReferences {
        prerequisite: resolve_reference_list(&cli.prerequisite, title_index.as_ref())?,
        supersedes: resolve_reference_list(&cli.supersedes, title_index.as_ref())?,
        superseded_by: resolve_reference_list(&cli.superseded_by, title_index.as_ref())?,
    })
}

/// Ensure no existing RFC title conflicts with the provided title.
///
/// Conflict checks are performed by case-insensitive title match and slug
/// match to prevent effectively-duplicate RFC entries.
pub(crate) fn ensure_unique_rfc_title(title: &str) -> Result<()> {
    let index = RfcTitleIndex::load()?;
    let matches = index.find_title_conflicts(title);
    if matches.is_empty() {
        return Ok(());
    }

    let normalized = title.trim();
    if matches.len() == 1 {
        let existing = matches[0];
        bail!(
            "RFC title `{normalized}` already exists in {} as {:04} ({})",
            index.rfc_dir.display(),
            existing.id,
            existing.title
        );
    }

    bail!(
        "RFC title `{normalized}` conflicts with multiple existing RFCs in {}: {}",
        index.rfc_dir.display(),
        format_match_list(&matches)
    )
}

fn resolve_reference_list(
    references: &[RfcReference],
    title_index: Option<&RfcTitleIndex>,
) -> Result<Vec<u32>> {
    let mut resolved = Vec::new();
    for reference in references {
        match reference {
            RfcReference::Id(id) => resolved.push(*id),
            RfcReference::Title(title) => {
                let index = title_index.ok_or_else(|| anyhow!("missing title index"))?;
                resolved.push(index.resolve_title(title)?);
            }
        }
    }
    Ok(dedupe(&resolved))
}

struct RfcTitleIndex {
    entries: Vec<RfcTitleEntry>,
    rfc_dir: PathBuf,
}

struct RfcTitleEntry {
    id: u32,
    title: String,
    title_folded: String,
    title_slug: String,
}

impl RfcTitleIndex {
    /// Build a searchable title index from RFC files in the resolved RFC dir.
    fn load() -> Result<Self> {
        let rfc_dir = resolve_project_rfc_dir()?;
        if !rfc_dir.is_dir() {
            bail!(
                "cannot resolve RFC title references: RFC directory does not exist at {}",
                rfc_dir.display()
            );
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&rfc_dir)
            .with_context(|| format!("failed to read RFC directory {}", rfc_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if file_name == "0000-template.md" {
                continue;
            }
            let prefix: String = file_name.chars().take(4).collect();
            if prefix.len() != 4 || !prefix.chars().all(|ch| ch.is_ascii_digit()) {
                continue;
            }

            let (id, title) = parse_rfc_id_and_title(&path)
                .with_context(|| format!("failed to index RFC file {}", path.display()))?;
            entries.push(RfcTitleEntry {
                id,
                title_folded: title.trim().to_ascii_lowercase(),
                title_slug: slugify(&title),
                title,
            });
        }

        Ok(Self { entries, rfc_dir })
    }

    /// Resolve a title-like string to a single RFC id.
    ///
    /// Matching order:
    /// 1. Exact title
    /// 2. Case-insensitive title
    /// 3. Slugified title
    fn resolve_title(&self, input: &str) -> Result<u32> {
        let normalized = input.trim();
        if normalized.is_empty() {
            bail!("RFC title reference cannot be empty");
        }

        let exact_matches = self
            .entries
            .iter()
            .filter(|entry| entry.title == normalized)
            .collect::<Vec<_>>();
        if exact_matches.len() == 1 {
            return Ok(exact_matches[0].id);
        }
        if exact_matches.len() > 1 {
            bail!(
                "RFC title reference `{normalized}` matched multiple RFCs by exact title: {}",
                format_match_list(&exact_matches)
            );
        }

        let folded = normalized.to_ascii_lowercase();
        let folded_matches = self
            .entries
            .iter()
            .filter(|entry| entry.title_folded == folded)
            .collect::<Vec<_>>();
        if folded_matches.len() == 1 {
            return Ok(folded_matches[0].id);
        }
        if folded_matches.len() > 1 {
            bail!(
                "RFC title reference `{normalized}` matched multiple RFCs by case-insensitive title: {}",
                format_match_list(&folded_matches)
            );
        }

        let slug = slugify(normalized);
        let slug_matches = self
            .entries
            .iter()
            .filter(|entry| entry.title_slug == slug)
            .collect::<Vec<_>>();
        if slug_matches.len() == 1 {
            return Ok(slug_matches[0].id);
        }
        if slug_matches.len() > 1 {
            bail!(
                "RFC title reference `{normalized}` matched multiple RFCs by slug: {}",
                format_match_list(&slug_matches)
            );
        }

        bail!(
            "unable to resolve RFC title reference `{normalized}` in {}",
            self.rfc_dir.display()
        )
    }

    fn find_title_conflicts<'a>(&'a self, input: &str) -> Vec<&'a RfcTitleEntry> {
        let normalized = input.trim();
        if normalized.is_empty() {
            return Vec::new();
        }

        let folded = normalized.to_ascii_lowercase();
        let slug = slugify(normalized);

        self.entries
            .iter()
            .filter(|entry| entry.title_folded == folded || entry.title_slug == slug)
            .collect()
    }
}

fn format_match_list(matches: &[&RfcTitleEntry]) -> String {
    matches
        .iter()
        .map(|entry| format!("{:04} ({})", entry.id, entry.title))
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_rfc_id_and_title(path: &Path) -> Result<(u32, String)> {
    let markdown = fs::read_to_string(path)
        .with_context(|| format!("failed to read RFC file {}", path.display()))?;
    let frontmatter = extract_frontmatter(&markdown)?;
    let metadata = frontmatter
        .parse::<DocumentMut>()
        .context("failed to parse RFC frontmatter as TOML")?;

    let title = metadata
        .get("title")
        .and_then(|item| item.as_str())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("metadata is missing required `title` field"))?;
    let rfc_id = parse_rfc_id_item(
        metadata
            .get("rfc")
            .ok_or_else(|| anyhow!("metadata is missing required `rfc` field"))?,
    )?;

    Ok((rfc_id, title))
}

fn parse_rfc_id_item(item: &Item) -> Result<u32> {
    if let Some(value) = item.as_str() {
        return value
            .parse::<u32>()
            .with_context(|| format!("invalid RFC id `{value}`"));
    }
    if let Some(value) = item.as_integer() {
        let parsed = u32::try_from(value).context("RFC id must be a non-negative integer")?;
        return Ok(parsed);
    }

    bail!("RFC id field must be a string or integer")
}

fn extract_frontmatter(markdown: &str) -> Result<String> {
    let normalized = markdown.replace("\r\n", "\n");
    if !normalized.starts_with("+++\n") {
        bail!("RFC file does not start with TOML frontmatter marker `+++`");
    }

    let rest = &normalized[4..];
    if let Some(end) = rest.find("\n+++\n") {
        return Ok(rest[..end].to_owned());
    }
    if let Some(end) = rest.find("\n+++") {
        return Ok(rest[..end].to_owned());
    }

    bail!("missing closing TOML frontmatter marker `+++`");
}
