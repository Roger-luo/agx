---
name: new-rfc
description: Draft and revise RFC markdown files for agx using project template, metadata conventions, and review gates.
---

# agx RFC Writer

## Overview

Create RFC design documents that match agx conventions, cite concrete code locations, and follow the repository RFC workflow.

## Workflow

1. Collect scope and constraints.
   - Confirm goals, motivation, non-goals, compatibility impact, and migration needs.
   - Identify affected modules before drafting, including concrete paths under `src/rfc/`, `src/skill/`, and other touched areas.
   - Read `README.md`, `rfc/0000-template.md`, and `src/cli.rs` for terminology and command behavior.
2. Create or revise the RFC file first.
   - For new RFCs, run `agx rfc new --agent <agent-name> --author "AUTHOR_NAME" --title "RFC_TITLE"`.
   - For updates, run `agx rfc revise --agent <agent-name> RFC_SELECTOR`.
   - Use metadata flags as needed: `--discussion`, `--tracking_issue`, `--prerequisite`, `--supersedes`, `--superseded_by`, `--title`, `--title_parts`.
   - Keep metadata updates command-driven when a CLI flag exists.
   - After revising, replace the default `[[revision]].change` value (`Revised`) with a very brief purpose summary that is no more than one sentence.
3. Fill all template sections.
   - Keep every heading from `rfc/0000-template.md`.
   - Replace template helper text with concrete design content.
   - If a section is not applicable, keep the heading and state why.
4. Ground the design in code and docs.
   - Reference exact touched files (for example, `src/rfc/create.rs`, `src/rfc/revise.rs`, `src/skill/validate.rs`).
   - Include `README.md` references whenever CLI behavior or user workflow changes.
5. Evaluate alternatives and risks.
   - Include at least two alternatives with explicit tradeoffs.
   - Cover compatibility, migration, and failure modes.
6. Close with actionable outcomes.
   - List unresolved questions and decision points explicitly.
   - List implementation slices in dependency order.
   - Include the validation command plan.

## agx-specific requirements

- RFC files live under `rfc/` and use the `NNNN-slug.md` naming pattern.
- Template source is `rfc/0000-template.md`; if absent, tooling falls back to the embedded template.
- Allowed metadata fields are current agx fields: `rfc`, `title`, `agents`, `authors`, `created`, `last_updated`, `discussion`, `tracking_issue`, `prerequisite`, `supersedes`, `superseded_by`, `revision`.
- Do not introduce a `status` frontmatter field.
- Keep terminology consistent with `README.md`, `rfc/0000-template.md`, and `src/cli.rs`.
- Keep tone concise, concrete, and file-path specific.

## Quality gate

- Require explicit compatibility and migration guidance; if none is needed, state "No migration required" and explain why.
- Include unresolved questions in the RFC before review.
- Include validation commands: `cargo fmt --all`, `cargo build --workspace`, `cargo test --workspace`, and `cargo insta review` when snapshots change.
- RFC is review-ready when the above items are present and the RFC is opened for maintainer review in a pull request.
