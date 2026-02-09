---
name: new-rfc
description: Draft, revise, and review Request for Comments (RFC) documents for <project-name>. Use when proposing or changing <project-scope>.
---

# <Project Name> RFC Writer

## Overview

Create RFC design documents that match project conventions, point to concrete code locations, and follow the repository RFC workflow.

## Workflow

1. Collect scope and constraints.
   - Confirm change goals, motivation, and non-goals.
   - Identify affected crates, modules, and APIs before drafting.
   - Read relevant design docs and code first.
2. Create the RFC file first.
   - Run `<rfc-new-command>` with required metadata flags.
   - If updating an existing RFC, run `<rfc-revise-or-update-command>` in revision mode and pass `--agent <agent-name>`.
   - Always pass `--agent <agent-name>` so the acting agent identity is recorded in RFC metadata.
   - Revision entries must include a purpose summary in `[[revision]].change` that is no more than one sentence and very brief.
   - Never leave a generic revision message such as `Revised`; replace it with the brief purpose summary.
   - Keep metadata updates driven by commands, not manual frontmatter edits.
3. Fill template placeholders.
   - Replace every bracketed placeholder with concrete content.
   - Remove template-only helper text from the final RFC.
   - Tie behavior changes to specific files, types, and modules.
4. Handle illustrative sections intentionally.
   - Rename or restructure illustrative headings as needed.
   - Keep only sections relevant to this RFC.
5. Evaluate alternatives and risks.
   - Describe at least two alternatives and clear tradeoffs.
   - Cover compatibility, migration, and failure modes.
   - Define validation work (tests, snapshots, benchmarks, roundtrip checks).
6. Close with actionable outcomes.
   - End with open questions and explicit decision points.
   - List implementation slices in dependency order.

## <Project Name>-specific requirements

- Keep terminology consistent with <terminology-source-files>.
- Match tone and structure from <reference-docs>.
- Prefer concrete file references over abstract statements.
- For syntax or format changes, define roundtrip expectations when relevant.
- For tests, name exact crates and test targets.
- Use allowed RFC status values only: <status-values>.

## Quality gate

- Run checklist: <checklist-path-or-process>.
- If any checklist item is not satisfied, call it out explicitly in the RFC.
