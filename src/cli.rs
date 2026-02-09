//! CLI shape and argument parsing for agx.
//!
//! agx is a general CLI for agent workflow tooling. RFC metadata reference
//! fields (`prerequisite`, `supersedes`, `superseded_by`) accept either an RFC
//! id (for example `12`) or a title string.

use clap::{ArgAction, Args, Parser, Subcommand};
use std::str::FromStr;

#[derive(Debug, Parser)]
#[command(
    name = "agx",
    about = "Manage agent workflow tooling",
    long_about = "Manage agent workflow tooling.\n\n\
Use `rfc` to initialize RFC project assets and create/revise RFC markdown files.\n\
Use `skill` to initialize/create/validate local skills.",
    after_help = "Examples:\n\
  agx rfc init\n\
  agx rfc new --author Roger --title \"Add parser support\"\n\
  agx rfc revise 0001\n\
  agx skill init\n\
  agx skill new ask-user-question\n\
  agx skill validate\n\
  agx skill validate ask-user-question"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(
        name = "rfc",
        about = "Initialize, create, and revise RFC markdown files",
        long_about = "Initialize, create, and revise RFC markdown files.\n\n\
`rfc init` scaffolds project RFC folders and installs the `create-rfc` skill.\n\
`rfc new` creates a new RFC from the resolved template source.\n\
`rfc revise` updates an existing RFC in place."
    )]
    Rfc(RfcArgs),

    #[command(
        name = "skill",
        about = "Initialize, create, and validate skills",
        long_about = "Initialize, create, and validate skills.\n\n\
Use `skill init` to scaffold `.agents/skills`.\n\
Use `skill new` to create a new skill scaffold.\n\
Use `skill validate` to validate one or more skills."
    )]
    Skill(SkillArgs),
}

#[derive(Debug, Args)]
pub struct RfcArgs {
    #[command(subcommand)]
    pub command: RfcCommand,
}

#[derive(Debug, Subcommand)]
pub enum RfcCommand {
    #[command(
        name = "init",
        about = "Initialize RFC directory and install create-rfc skill",
        long_about = "Initialize RFC directory and install create-rfc skill.\n\n\
Creates `rfc` and installs `.agents/skills/create-rfc` when missing.",
        after_help = "Examples:\n\
  agx rfc init"
    )]
    Init,

    #[command(
        name = "new",
        about = "Create a new RFC markdown file with TOML metadata",
        long_about = "Create a new RFC markdown file with TOML metadata.\n\n\
Creates a new RFC file from `rfc/0000-template.md` when present, or falls back to the embedded template.",
        after_help = "Examples:\n\
  agx rfc new --author Roger --title \"Add parser support\"\n\
  agx rfc new --author Roger --title_parts parser support",
        override_usage = "agx rfc new [options] <title>"
    )]
    New(RfcEditArgs),

    #[command(
        name = "revise",
        about = "Revise an existing RFC markdown file in place",
        long_about = "Revise an existing RFC markdown file in place.\n\n\
Accepts the same options and input shape as `rfc new`, but the positional argument selects an existing RFC.",
        after_help = "Examples:\n\
  agx rfc revise 0001\n\
  agx rfc revise --title \"Updated RFC title\" 0001",
        override_usage = "agx rfc revise [options] <title>"
    )]
    Revise(RfcEditArgs),
}

#[derive(Debug, Args)]
pub struct SkillArgs {
    #[command(subcommand)]
    pub command: SkillCommand,
}

#[derive(Debug, Subcommand)]
pub enum SkillCommand {
    #[command(
        name = "init",
        about = "Initialize local skills directory",
        long_about = "Initialize local skills directory.\n\n\
Creates `.agents/skills` when missing.",
        after_help = "Examples:\n\
  agx skill init"
    )]
    Init,

    #[command(
        name = "new",
        about = "Create a new skill scaffold under .agents/skills",
        long_about = "Create a new skill scaffold under `.agents/skills`.\n\n\
Creates `.agents/skills/<name>` with `SKILL.md` and `agents/openai.yaml`.",
        after_help = "Examples:\n\
  agx skill new ask-user-question"
    )]
    New(SkillNewArgs),

    #[command(
        name = "validate",
        about = "Validate one skill or all skills under .agents/skills",
        long_about = "Validate one skill or all skills under `.agents/skills`.\n\n\
Defaults to all skills when no name is provided.",
        after_help = "Examples:\n\
  agx skill validate\n\
  agx skill validate ask-user-question"
    )]
    Validate(SkillValidateArgs),
}

#[derive(Debug, Args)]
pub struct SkillNewArgs {
    /// Skill name to scaffold under `.agents/skills`.
    #[arg(value_name = "name")]
    pub name: String,
}

#[derive(Debug, Args)]
pub struct SkillValidateArgs {
    /// Optional skill name under `.agents/skills`.
    #[arg(value_name = "name")]
    pub name: Option<String>,
}

/// CLI-provided RFC reference used by metadata fields.
///
/// Numeric inputs are treated as direct RFC ids, while non-numeric inputs are
/// resolved later as RFC titles against the project RFC directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RfcReference {
    /// Direct numeric RFC identifier.
    Id(u32),
    /// RFC title that must be resolved to an identifier.
    Title(String),
}

impl FromStr for RfcReference {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim();
        if normalized.is_empty() {
            return Err("RFC reference cannot be empty".to_owned());
        }

        if normalized.chars().all(|ch| ch.is_ascii_digit()) {
            let parsed = normalized
                .parse::<u32>()
                .map_err(|_| format!("invalid RFC id `{normalized}`"))?;
            return Ok(Self::Id(parsed));
        }

        Ok(Self::Title(normalized.to_owned()))
    }
}

#[derive(Debug, Args)]
pub struct RfcEditArgs {
    /// Add an author to metadata. Repeat to include multiple authors.
    #[arg(long = "author", value_name = "name", action = ArgAction::Append)]
    pub authors: Vec<String>,

    /// Add an agent identifier to metadata. Repeat to include multiple agents.
    #[arg(long = "agent", value_name = "name", action = ArgAction::Append)]
    pub agents: Vec<String>,

    /// Set the discussion reference (for example, a link or ticket id).
    #[arg(long = "discussion", value_name = "link or id")]
    pub discussion: Option<String>,

    /// Set the tracking issue reference (for example, a link or ticket id).
    #[arg(long = "tracking_issue", value_name = "link or id")]
    pub tracking_issue: Option<String>,

    /// List prerequisite RFC references (id or title). Repeat to add multiple.
    #[arg(
        long = "prerequisite",
        value_name = "rfc id or title",
        action = ArgAction::Append
    )]
    pub prerequisite: Vec<RfcReference>,

    /// List superseded RFC references (id or title). Repeat to add multiple.
    #[arg(
        long = "supersedes",
        value_name = "rfc id or title",
        action = ArgAction::Append
    )]
    pub supersedes: Vec<RfcReference>,

    /// List replacement RFC references (id or title). Repeat to add multiple.
    #[arg(
        long = "superseded_by",
        value_name = "rfc id or title",
        action = ArgAction::Append
    )]
    pub superseded_by: Vec<RfcReference>,

    /// Set the RFC title directly. Takes precedence over positional <title>.
    #[arg(long = "title", value_name = "string")]
    pub title: Option<String>,

    /// Build the RFC title by joining parts with underscores.
    #[arg(long = "title_parts", value_name = "string", num_args = 1..)]
    pub title_parts: Vec<String>,

    /// For `rfc new`: RFC title. For `rfc revise`: selector (path, id, or slug) for an existing RFC.
    #[arg(value_name = "title")]
    pub title_arg: Option<String>,
}

impl RfcEditArgs {
    /// Resolve title input precedence:
    /// `--title` > `--title_parts` > positional `<title>`.
    pub fn resolved_title(&self) -> Option<String> {
        if let Some(title) = &self.title {
            return Some(title.clone());
        }

        if !self.title_parts.is_empty() {
            return Some(self.title_parts.join("_"));
        }

        self.title_arg.clone()
    }
}
