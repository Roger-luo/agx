---
name: new-rfc
description: Draft and revise RFC documents for the agx CLI project using the local RFC template, metadata conventions, and `agx rfc` workflow.
---

# AGX RFC Writer

## Overview

Create RFC design documents for this repository in `rfc/` using `agx rfc new` and `agx rfc revise`, keeping metadata and revision history consistent with project conventions.

## Workflow

1. Collect scope and constraints.
   - Confirm motivation, non-goals, and expected user impact.
   - Read relevant code and docs first, especially `src/rfc/`, `src/skill/`, `README.md`, and `AGENTS.md`.
2. Create the RFC file first.
   - Run `agx rfc new [options] <title>`.
   - Prefer explicit metadata flags when available:
     - `--author`
     - `--agent`
     - `--discussion`
     - `--tracking_issue`
     - `--prerequisite`
     - `--supersedes`
     - `--superseded_by`
     - `--title` or `--title_parts`
   - Title precedence is `--title` > `--title_parts` > positional `<title>`.
   - Output path is `rfc/NNNN-<slug>.md`.
3. Revise existing RFCs through command flow.
   - Run `agx rfc revise [options] <selector>` where selector can be an RFC id, path, or slug-like selector.
   - Let the command update metadata and append a `[[revision]]` entry with change text `Revised`.
4. Fill all template sections with concrete content.
   - The RFC template source is `rfc/0000-template.md` when present, otherwise the embedded fallback template.
   - Replace all instructional placeholder text with project-specific details.
   - Tie design claims to concrete code locations and APIs.
5. Evaluate alternatives and risks.
   - Include at least two alternatives with explicit tradeoffs.
   - Cover backwards compatibility, migration impact, and failure modes.
6. Close with actionable outcomes.
   - End with unresolved questions and explicit implementation slices in dependency order.

## Project-specific requirements

- Keep terminology consistent with this codebase: `agx`, `rfc new`, `rfc revise`, RFC frontmatter metadata, and `[[revision]]`.
- Use exact file references for impacted implementation (for example `src/rfc/create.rs`, `src/rfc/revise.rs`, `src/cli.rs`).
- For metadata references (`prerequisite`, `supersedes`, `superseded_by`), provide ids or titles; command resolution handles conversion.
- Do not hand-edit generated metadata when command flags can express the change.
- This project currently has no dedicated RFC status field in the default template; do not introduce a custom status taxonomy unless requested.

## Required RFC sections

Keep these sections and make each one project-specific:

- `Summary`
- `Motivation`
- `Guide-level explanation`
- `Reference-level explanation`
- `Reference implementation`
- `Backwards compatibility`
- `Security implications`
- `How to teach this`
- `Drawbacks`
- `Rationale and alternatives`
- `Prior art`
- `Unresolved questions`
- `Future possibilities`

## Quality gate

- Ensure no placeholder guidance text remains in the final RFC body.
- Verify command examples are executable in this repo.
- Run relevant checks when implementation accompanies the RFC:
  - `cargo build --workspace`
  - `cargo test --workspace`
  - `cargo fmt --all`
- If any quality item is intentionally skipped, call it out explicitly in the RFC.
