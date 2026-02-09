# agx

`agx` is a Rust CLI for project-local RFC and skill workflows.

## Current Functionality

### RFC commands

- `agx rfc init`
  - Creates `rfc/`.
  - Creates `.agents/skills/` if missing.
  - Installs `.agents/skills/create-rfc/` with:
    - `SKILL.md`
    - `agents/openai.yaml`

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

## Examples

```bash
agx rfc init
agx rfc new --author Roger --title "Add parser support"
agx rfc revise --title "Add parser support (updated)" 0001

agx skill init
agx skill new ask-user-question
agx skill validate
agx skill validate ask-user-question
```

## Build and Test

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
```
