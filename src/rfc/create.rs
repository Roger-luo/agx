use std::{fs::OpenOptions, io::Write, path::Path};

use anyhow::{Context, Result, anyhow, bail};
use tera::{Context as TeraContext, Tera};

use crate::cli::RfcEditArgs;

use super::reference::{ensure_unique_rfc_title, resolve_metadata_references};
use super::template::load_template;
use super::util::{
    INITIAL_REVISION_CHANGE, RFC_DIR, dedupe, next_rfc_id, resolve_default_author, slugify,
    timestamp_now, toml_escape,
};

/// Create a new RFC file using CLI inputs and the resolved template source.
pub(crate) fn create_rfc(cli: &RfcEditArgs) -> Result<()> {
    let title = cli.resolved_title().ok_or_else(|| {
        anyhow!("missing <title>: pass positional <title>, --title, or --title_parts")
    })?;
    if is_numeric_selector(&title) {
        bail!(
            "create mode does not accept numeric-only title `{}`; numeric values are treated as RFC ids by `rfc revise`",
            title.trim()
        );
    }
    ensure_unique_rfc_title(&title)?;

    let mut authors = dedupe(&cli.authors);
    if authors.is_empty() {
        authors.push(resolve_default_author()?);
    }

    let agents = dedupe(&cli.agents);
    let references = resolve_metadata_references(cli)?;

    let rfc_id = next_rfc_id(Path::new(RFC_DIR))?;
    let output_path = Path::new(RFC_DIR).join(format!("{rfc_id}-{}.md", slugify(&title)));
    if output_path.exists() {
        bail!("output RFC already exists: {}", output_path.display());
    }

    let timestamp = timestamp_now();
    let revision_timestamp = timestamp.clone();

    let mut context = TeraContext::new();
    context.insert("rfc_id", &rfc_id);
    context.insert("title", &title);
    context.insert("title_toml", &toml_escape(&title));
    context.insert(
        "agents",
        &agents
            .iter()
            .map(|entry| toml_escape(entry))
            .collect::<Vec<_>>(),
    );
    context.insert(
        "authors",
        &authors
            .iter()
            .map(|entry| toml_escape(entry))
            .collect::<Vec<_>>(),
    );
    context.insert("timestamp", &timestamp);
    context.insert(
        "discussion",
        &cli.discussion.as_ref().map(|v| toml_escape(v)),
    );
    context.insert(
        "tracking_issue",
        &cli.tracking_issue.as_ref().map(|v| toml_escape(v)),
    );
    context.insert("prerequisite", &references.prerequisite);
    context.insert("supersedes", &references.supersedes);
    context.insert("superseded_by", &references.superseded_by);
    context.insert("revision_timestamp", &revision_timestamp);
    context.insert("revision_change", &toml_escape(INITIAL_REVISION_CHANGE));

    let template = load_template()?;
    let rendered =
        Tera::one_off(&template, &context, false).context("failed to render template")?;

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&output_path)
        .with_context(|| format!("failed to create RFC at {}", output_path.display()))?;
    file.write_all(rendered.as_bytes())
        .with_context(|| format!("failed to write RFC file {}", output_path.display()))?;

    println!("{}", output_path.display());
    Ok(())
}

fn is_numeric_selector(value: &str) -> bool {
    let normalized = value.trim();
    !normalized.is_empty() && normalized.chars().all(|ch| ch.is_ascii_digit())
}
