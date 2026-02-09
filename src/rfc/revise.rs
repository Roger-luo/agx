use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value, value};

use crate::cli::RfcEditArgs;
use crate::output;

use super::{
    lookup::locate_existing_rfc,
    reference::resolve_metadata_references,
    util::{REVISED_REVISION_CHANGE, dedupe, timestamp_now},
};

/// Update an existing RFC frontmatter/body and append a revision entry.
pub(crate) fn revise_rfc(cli: &RfcEditArgs) -> Result<()> {
    let selector = cli.title_arg.as_deref().ok_or_else(|| {
        anyhow!("rfc revise requires positional <title> to locate an existing RFC")
    })?;
    let path = locate_existing_rfc(selector)?;
    let original = fs::read_to_string(&path)
        .with_context(|| format!("failed to read RFC file {}", path.display()))?;
    let (frontmatter, body) = split_frontmatter(&original)?;
    let mut metadata = frontmatter
        .parse::<DocumentMut>()
        .context("failed to parse RFC TOML frontmatter")?;

    for author in dedupe(&cli.authors) {
        append_unique_array_value(&mut metadata, "authors", &author)?;
    }
    for agent in dedupe(&cli.agents) {
        append_unique_array_value(&mut metadata, "agents", &agent)?;
    }
    let references = resolve_metadata_references(cli)?;

    if let Some(discussion) = &cli.discussion {
        metadata["discussion"] = value(discussion.as_str());
    }
    if let Some(tracking_issue) = &cli.tracking_issue {
        metadata["tracking_issue"] = value(tracking_issue.as_str());
    }
    if !references.prerequisite.is_empty() {
        set_integer_array_value(&mut metadata, "prerequisite", &references.prerequisite);
    }
    if !references.supersedes.is_empty() {
        set_integer_array_value(&mut metadata, "supersedes", &references.supersedes);
    }
    if !references.superseded_by.is_empty() {
        set_integer_array_value(&mut metadata, "superseded_by", &references.superseded_by);
    }

    let title_override = revision_title_override(cli);
    if let Some(new_title) = &title_override {
        metadata["title"] = value(new_title.as_str());
    }

    let updated_timestamp = timestamp_now();
    metadata["last_updated"] = value(updated_timestamp.clone());
    append_revision_entry(
        &mut metadata,
        updated_timestamp,
        REVISED_REVISION_CHANGE.to_owned(),
    )?;

    let rfc_id = metadata
        .get("rfc")
        .and_then(|item| item.as_str())
        .ok_or_else(|| anyhow!("metadata is missing required `rfc` field"))?;
    let title = title_override
        .or_else(|| {
            metadata
                .get("title")
                .and_then(|item| item.as_str())
                .map(ToOwned::to_owned)
        })
        .ok_or_else(|| anyhow!("metadata is missing required `title` field"))?;

    let updated_body = rewrite_rfc_heading(&body, rfc_id, &title);
    let mut updated = String::new();
    updated.push_str("+++\n");
    let mut serialized_frontmatter = metadata.to_string();
    if !serialized_frontmatter.ends_with('\n') {
        serialized_frontmatter.push('\n');
    }
    updated.push_str(&serialized_frontmatter);
    updated.push_str("+++\n\n");
    updated.push_str(updated_body.trim_start_matches('\n'));
    if !updated.ends_with('\n') {
        updated.push('\n');
    }

    fs::write(&path, updated).with_context(|| format!("failed to update {}", path.display()))?;
    output::print_path(path.display());
    Ok(())
}

fn revision_title_override(cli: &RfcEditArgs) -> Option<String> {
    if let Some(title) = &cli.title {
        return Some(title.clone());
    }

    if !cli.title_parts.is_empty() {
        return Some(cli.title_parts.join("_"));
    }

    None
}

fn split_frontmatter(markdown: &str) -> Result<(String, String)> {
    let normalized = markdown.replace("\r\n", "\n");
    if !normalized.starts_with("+++\n") {
        bail!("RFC file does not start with TOML frontmatter marker `+++`");
    }

    let rest = &normalized[4..];
    if let Some(end) = rest.find("\n+++\n") {
        let frontmatter = rest[..end].to_owned();
        let body = rest[end + 5..].to_owned();
        return Ok((frontmatter, body));
    }
    if let Some(end) = rest.find("\n+++") {
        let frontmatter = rest[..end].to_owned();
        let mut body = rest[end + 4..].to_owned();
        if body.starts_with('\n') {
            body = body[1..].to_owned();
        }
        return Ok((frontmatter, body));
    }

    bail!("missing closing TOML frontmatter marker `+++`");
}

fn rewrite_rfc_heading(body: &str, rfc_id: &str, title: &str) -> String {
    let heading = format!("# RFC {rfc_id}: {title}");
    let mut replaced = false;
    let mut output = String::new();

    for line in body.lines() {
        if !replaced && line.starts_with("# RFC ") {
            output.push_str(&heading);
            replaced = true;
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }

    if replaced {
        return output;
    }

    let mut prefixed = String::new();
    prefixed.push_str(&heading);
    prefixed.push_str("\n\n");
    prefixed.push_str(body.trim_start_matches('\n'));
    if !prefixed.ends_with('\n') {
        prefixed.push('\n');
    }
    prefixed
}

fn append_unique_array_value(doc: &mut DocumentMut, key: &str, value_to_add: &str) -> Result<()> {
    if !doc.as_table().contains_key(key) {
        let mut values = Array::new();
        values.push(value_to_add);
        doc[key] = Item::Value(Value::Array(values));
        return Ok(());
    }

    let Some(array) = doc[key].as_array_mut() else {
        bail!("metadata field `{key}` exists but is not an array");
    };

    let already_present = array
        .iter()
        .filter_map(|entry| entry.as_str())
        .any(|entry| entry == value_to_add);
    if !already_present {
        array.push(value_to_add);
    }

    Ok(())
}

fn set_integer_array_value(doc: &mut DocumentMut, key: &str, values: &[u32]) {
    let mut array = Array::new();
    for entry in values {
        array.push(i64::from(*entry));
    }
    doc[key] = Item::Value(Value::Array(array));
}

fn append_revision_entry(doc: &mut DocumentMut, date: String, change: String) -> Result<()> {
    if !doc.as_table().contains_key("revision") {
        doc["revision"] = Item::ArrayOfTables(ArrayOfTables::new());
    }

    let Some(revisions) = doc["revision"].as_array_of_tables_mut() else {
        bail!("metadata field `revision` exists but is not an array of tables");
    };

    let mut entry = Table::new();
    entry["date"] = value(date);
    entry["change"] = value(change);
    revisions.push(entry);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{rewrite_rfc_heading, split_frontmatter};

    #[test]
    fn split_frontmatter_parses_metadata_and_body() {
        let markdown = "+++\nrfc = \"0001\"\n+++\n\n# RFC 0001: Title\n";
        let (frontmatter, body) = split_frontmatter(markdown).expect("frontmatter should parse");
        assert_eq!(frontmatter.trim(), "rfc = \"0001\"");
        assert_eq!(body.trim(), "# RFC 0001: Title");
    }

    #[test]
    fn split_frontmatter_rejects_missing_markers() {
        let error = split_frontmatter("# RFC 0001: Title").expect_err("expected error");
        assert!(error.to_string().contains("frontmatter marker"));
    }

    #[test]
    fn rewrite_rfc_heading_replaces_existing_heading() {
        let body = "# RFC 0001: Old\n\n## Summary\n";
        let updated = rewrite_rfc_heading(body, "0001", "New");
        assert!(updated.starts_with("# RFC 0001: New"));
        assert_eq!(updated.matches("# RFC ").count(), 1);
    }

    #[test]
    fn rewrite_rfc_heading_prepends_when_absent() {
        let body = "## Summary\nDetails\n";
        let updated = rewrite_rfc_heading(body, "0002", "Prepended");
        assert!(updated.starts_with("# RFC 0002: Prepended\n\n## Summary"));
    }
}
