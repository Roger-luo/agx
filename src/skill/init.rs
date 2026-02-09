use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use arboard::Clipboard;

use crate::cli::{SkillInitArgs, SkillNewArgs};
use crate::output;

use super::{builtin, metadata::validate_skill_name};

pub(crate) const SKILLS_ROOT: &str = ".agents/skills";
const RECOMMENDED_PROMPT: &str = "Use $new-rfc-skill-creation-skill to create a project skill named `new-rfc` (new RFC). Ask for my feedback and keep iterating until I confirm the skill is correct.";
const DISABLE_CLIPBOARD_ENV: &str = "AGX_DISABLE_CLIPBOARD";

/// Initialize `.agents/skills`.
pub(crate) fn run(args: SkillInitArgs) -> Result<()> {
    fs::create_dir_all(SKILLS_ROOT).with_context(|| format!("failed to create `{SKILLS_ROOT}`"))?;
    output::print_path(SKILLS_ROOT);
    if !args.no_dump {
        seed_builtin_skills()?;
    }
    output::print_hint(
        "use the code agent to initialize and create new RFC skills in this project",
    );
    output::print_hint("recommended prompt (copy and paste):");
    output::print_quote(RECOMMENDED_PROMPT);
    match copy_to_clipboard(RECOMMENDED_PROMPT) {
        Ok(()) => output::print_log("copied recommended prompt to clipboard"),
        Err(error) => output::print_warning(format!(
            "failed to copy recommended prompt to clipboard: {error:#}"
        )),
    }
    Ok(())
}

/// Create a new skill scaffold under `.agents/skills`.
pub(crate) fn run_new(args: SkillNewArgs) -> Result<()> {
    fs::create_dir_all(SKILLS_ROOT).with_context(|| format!("failed to create `{SKILLS_ROOT}`"))?;
    output::print_path(SKILLS_ROOT);
    scaffold_skill(&args.name)
}

fn scaffold_skill(name: &str) -> Result<()> {
    validate_skill_name(name)?;

    let skill_dir = Path::new(SKILLS_ROOT).join(name);
    let agents_dir = skill_dir.join("agents");
    fs::create_dir_all(&agents_dir)
        .with_context(|| format!("failed to create `{}`", agents_dir.display()))?;
    output::print_path(skill_dir.display());
    output::print_path(agents_dir.display());

    let skill_file = skill_dir.join("SKILL.md");
    write_if_missing(
        &skill_file,
        &format!(
            "---\nname: {name}\ndescription: Describe what this skill does and when to use it.\n---\n\n# {title}\n",
            title = title_case(name)
        ),
    )?;

    let openai_yaml = agents_dir.join("openai.yaml");
    write_if_missing(
        &openai_yaml,
        &format!(
            "interface:\n  display_name: \"{title}\"\n  short_description: \"Describe this skill briefly\"\n  default_prompt: \"Use ${name} to help with this task.\"\n",
            title = title_case(name)
        ),
    )?;

    Ok(())
}

fn seed_builtin_skills() -> Result<()> {
    let builtins = builtin::load_skills()?;
    for skill in builtins {
        let skill_dir = Path::new(SKILLS_ROOT).join(&skill.name);
        fs::create_dir_all(&skill_dir)
            .with_context(|| format!("failed to create `{}`", skill_dir.display()))?;
        output::print_path(skill_dir.display());

        for file in skill.files {
            let relative = Path::new(&file.path);
            if relative.is_absolute() {
                bail!("built-in skill file path `{}` must be relative", file.path);
            }
            for component in relative.components() {
                if !matches!(component, std::path::Component::Normal(_)) {
                    bail!(
                        "built-in skill file path `{}` must not contain traversal components",
                        file.path
                    );
                }
            }

            let destination = skill_dir.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create `{}`", parent.display()))?;
                output::print_path(parent.display());
            }
            write_if_missing(&destination, &file.content)?;
        }
    }

    Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<()> {
    if path.exists() {
        output::print_path(path.display());
        return Ok(());
    }

    fs::write(path, content).with_context(|| format!("failed to write `{}`", path.display()))?;
    output::print_path(path.display());
    Ok(())
}

fn title_case(name: &str) -> String {
    name.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    if std::env::var_os(DISABLE_CLIPBOARD_ENV).is_some() {
        return Ok(());
    }

    let mut clipboard = Clipboard::new().context("failed to access system clipboard")?;
    clipboard
        .set_text(text.to_owned())
        .context("failed to set clipboard text")?;
    Ok(())
}
