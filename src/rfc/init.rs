use std::{fs, path::Path};

use anyhow::{Context, Result};

const RFC_DIR: &str = "rfc";
const SKILLS_ROOT: &str = ".agents/skills";
const CREATE_RFC_SKILL: &str = "create-rfc";
const CREATE_RFC_SKILL_MD: &str = r#"---
name: create-rfc
description: Create or revise RFC markdown files in this project with consistent metadata and structure.
---

# Create RFC

Use this skill when you need to add or revise RFC files in the local `rfc` directory.

## Workflow

1. Run `agx rfc new` to create a new RFC proposal.
2. Run `agx rfc revise` to update an existing RFC.
3. Keep metadata fields and revision history consistent with project context.
"#;
const CREATE_RFC_OPENAI_YAML: &str = r#"interface:
  display_name: "Create RFC"
  short_description: "Create and revise RFC documents in this project"
  default_prompt: "Use $create-rfc to manage RFC lifecycle with agx rfc new and agx rfc revise."
"#;

/// Initialize RFC project directory and install `.agents/skills/create-rfc`.
pub(crate) fn run() -> Result<()> {
    fs::create_dir_all(RFC_DIR).with_context(|| format!("failed to create `{RFC_DIR}`"))?;
    println!("{RFC_DIR}");

    fs::create_dir_all(SKILLS_ROOT).with_context(|| format!("failed to create `{SKILLS_ROOT}`"))?;
    println!("{SKILLS_ROOT}");

    let skill_dir = Path::new(SKILLS_ROOT).join(CREATE_RFC_SKILL);
    let agents_dir = skill_dir.join("agents");
    fs::create_dir_all(&agents_dir)
        .with_context(|| format!("failed to create `{}`", agents_dir.display()))?;
    println!("{}", skill_dir.display());
    println!("{}", agents_dir.display());

    let skill_md_path = skill_dir.join("SKILL.md");
    write_if_missing(&skill_md_path, CREATE_RFC_SKILL_MD)?;

    let openai_path = agents_dir.join("openai.yaml");
    write_if_missing(&openai_path, CREATE_RFC_OPENAI_YAML)?;

    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        println!("{}", path.display());
        return Ok(());
    }

    fs::write(path, content).with_context(|| format!("failed to write `{}`", path.display()))?;
    println!("{}", path.display());
    Ok(())
}
