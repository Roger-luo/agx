---
name: new-rfc-skill-creation-skill
description: Create a project-specific RFC-writing skill named `new-rfc` by interviewing for RFC conventions, applying a reusable template, and validating the generated skill.
---

# New Rfc Skill Creation Skill

## Overview

Create or refresh `.agents/skills/new-rfc/SKILL.md` and `.agents/skills/new-rfc/agents/openai.yaml` so the project has a concrete RFC-writing skill with project terminology and workflow.

## Workflow

1. Verify prerequisites.
   - Confirm `.agents/skills/ask-user-question/SKILL.md` exists.
   - If it does not exist, continue with inline questions and state that `$ask-user-question` could not be used.
2. Scaffold the target skill.
   - Run `agx skill new new-rfc`.
3. Interview until coverage is complete.
   - Use `$ask-user-question` behavior: ask one blocking question at a time, with concrete options and tradeoffs.
   - Keep asking and hinting until all checklist items are answered or explicitly marked not applicable.
   - If the user is unsure, offer a recommended default and ask for confirmation.
4. Specialize the template.
   - Start from `references/rfc-skill-template.md`.
   - Replace every `<placeholder>` with project-specific values.
   - Keep frontmatter keys limited to `name` and `description`.
   - Ensure frontmatter `name` is exactly `new-rfc`.
   - Ensure revision guidance in the generated skill requires revision mode plus `--agent <agent-name>`.
   - Ensure revision guidance requires replacing generic revision text with a very brief one-sentence purpose summary.
5. Write target files.
   - Write the specialized content to `.agents/skills/new-rfc/SKILL.md`.
   - Update `.agents/skills/new-rfc/agents/openai.yaml` with matching display name, short description, and a default prompt that references `$new-rfc`.
6. Validate and iterate.
   - Run `agx skill validate new-rfc`.
   - Fix any validation failures and rerun until it passes.
   - When this meta skill changes, also run `agx skill validate new-rfc-skill-creation-skill`.
7. Report output.
   - Summarize captured RFC specifics.
   - List any defaults that were assumed.
   - Provide the created or updated file paths.

## Project RFC Specifics Checklist

Cover every item before finalizing `new-rfc`:

- RFC lifecycle commands (`new`, `revise`, or project-specific equivalents), including required flags.
- Revision behavior: always revise with `--agent`, and always use a one-sentence revision purpose summary (not generic `Revised`).
- RFC file location, naming pattern, and template source path.
- Metadata fields and allowed status values.
- Required and optional RFC sections.
- Project terminology that must stay consistent with code and docs.
- Code areas, crates, or modules the RFC should reference.
- Quality gates (tests, lint, format, checklist, and review workflow).
- Compatibility and migration expectations.
- Output style expectations (tone, depth, and file reference requirements).

## Interview Hints

When details are missing, propose concrete starters such as:

- Command flow: `agx rfc new` and `agx rfc revise` vs `cargo xtask new-rfc`.
- Status set: `Draft/Review/Accepted/Implemented` vs a project-specific lifecycle.
- Validation: require a checklist section vs allow lightweight RFCs for trivial changes.

## Guardrails

- Do not leave unresolved `<placeholder>` tokens in `.agents/skills/new-rfc/SKILL.md`.
- Do not invent conventions when repository evidence exists; ask and cite files.
- Keep instructions concise and operational.
