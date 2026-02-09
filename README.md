# agx

`agx` is a Rust CLI for project-local RFC and skill workflows.

## Current Functionality

### RFC commands

- `agx rfc init`
  - Creates `rfc/`.
  - Writes `rfc/0000-template.md` from the embedded binary template when missing.
  - Requires `.agents/skills/` to already exist.
  - If missing, errors and suggests: `agx skill dump --all`.

- `agx rfc new [options] <title>`
  - Creates a new RFC markdown file in `rfc/` as `NNNN-<slug>.md`.
  - Uses `rfc/0000-template.md` when present; otherwise uses the embedded default template.
  - Supports metadata options:
    - `--author`
    - `--agent`
    - `--discussion`
    - `--tracking_issue`
    - `--prerequisite`
    - `--supersedes`
    - `--superseded_by`
    - `--title`
    - `--title_parts`
  - Title resolution order: `--title` > `--title_parts` > positional `<title>`.
  - Numeric-only titles are rejected in `rfc new`.
  - Prints the created RFC path on success.

- `agx rfc revise [options] <title>`
  - Revises an existing RFC in place.
  - Accepts the same metadata/title options as `rfc new`.
  - Positional `<title>` is the RFC selector and can be:
    - an integer RFC id (for example `1`),
    - a path,
    - a slug-like selector.
  - Appends a revision entry with change text `"Revised"`.
  - Updates `last_updated` and the `# RFC NNNN: ...` heading.
  - Prints the revised RFC path on success.

### Skill commands

- `agx skill init`
  - Creates `.agents/skills/`.
  - By default, also dumps built-in skills into `.agents/skills/` (equivalent scope to `agx skill dump --all`).
  - `--no-dump` opt-out creates only `.agents/skills/`.
  - Prints a hint to use the code agent for creating new RFC skills.
  - Prints a recommended copy-paste prompt for `$new-rfc-skill-creation-skill` so users can ask their coding agent to create `new-rfc` and collect feedback.
  - Copies the recommended prompt to the system clipboard when clipboard access is available.
  - Uses colored message categories in terminal output (paths/logs/hints/quoted prompt) when ANSI colors are supported.

- `agx skill new <skill-name>`
  - Creates `.agents/skills/<skill-name>/`.
  - Scaffolds:
    - `SKILL.md`
    - `agents/openai.yaml`
  - Skill names must be lowercase letters, digits, and `-`, with no leading/trailing/consecutive `-`.

- `agx skill validate [skill-name]`
  - With no name: validates all skills under `.agents/skills/`.
  - With a name: validates `.agents/skills/<skill-name>/`.
  - Validation checks include:
    - `SKILL.md` exists and has YAML frontmatter.
    - frontmatter contains only `name` and `description`.
    - folder name matches frontmatter `name`.
    - `description` is present and non-empty.
    - if `agents/openai.yaml` exists, it contains `interface:`.

- `agx skill list [--origin builtin|workspace|all] [--format text|json]`
  - Discovers built-in and/or workspace skills.
  - JSON output includes `schema_version` and stable machine-readable fields.
  - When both built-in and workspace entries exist for the same name, preferred origin is `workspace`.

- `agx skill dump (<name> | --all) [--to <path>] [--force]`
  - Human-oriented materialization of built-in skills.
  - Default target is `.agents/skills` under the current Cargo project root.
  - Refuses overwrites unless `--force` is provided.

- `agx skill install (<name> | --all) [--origin builtin] [--to <path>] [--force] [--format text|json]`
  - Automation-oriented materialization of built-in skills.
  - Default target is `.agents/skills`.
  - Refuses overwrites unless `--force` is provided.
  - JSON output includes installed skill names and paths.

- `agx skill export --origin builtin --output <archive.tar.gz>`
  - Exports built-in skills to a `.tar.gz` archive.
  - Preserves `.agents/skills/<name>/...` layout in the archive.

## Bundled skills

Built-in skills are generated at build time from:

- `.agents/skills/builtin-manifest.toml`
- `.agents/skills/<name>/...` directories selected by the manifest

The resulting built-in catalog is embedded in the `agx` binary and used consistently by:

- `agx skill list --origin builtin`
- `agx skill dump`
- `agx skill install --origin builtin`
- `agx skill export --origin builtin`

## Examples

```bash
agx rfc init
agx rfc new --author Roger --title "Add parser support"
agx rfc revise --title "Add parser support (updated)" 0001

agx skill init
agx skill new ask-user-question
agx skill validate
agx skill validate ask-user-question
agx skill list --origin all --format json
agx skill dump --all
agx skill install ask-user-question --format json --to /tmp/agent-skills
agx skill export --origin builtin --output dist/agx-skills-v0.1.0.tar.gz
```

## Build and Test

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
```
