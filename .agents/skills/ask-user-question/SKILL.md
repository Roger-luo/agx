---
name: ask-user-question
description: Run a structured requirements interview to remove ambiguity before implementation. Use when users ask to be interviewed, ask for guided questions, mention ask-user-question or interview skills, or when key decisions are missing and proceeding would risk rework.
---

# Ask User Question

Use focused, option-based questions to gather missing requirements before planning or coding.

## Workflow

1. Explore context first.
   - Read the request and inspect available artifacts such as code, docs, tickets, and prior answers.
   - Identify what is already decided versus unknown.
2. Ask only high-impact decisions.
   - Prioritize choices that affect scope, architecture, data model, UX, or delivery risk.
   - Defer low-level implementation details until direction is clear.
3. Ask one question at a time by default.
   - Batch only when answers are independent or when latency is high.
4. Offer mutually exclusive options.
   - Provide 2-4 concrete options.
   - Put the recommended option first and explain tradeoffs for each option.
5. Wait for the answer before proceeding.
   - Do not start implementation until blocking decisions are resolved.
6. Iterate until requirements are actionable.
   - Summarize decisions and remaining unknowns after each answer.

## Question Design Rules

- Keep headers short (12 characters or fewer) when the interface supports headers.
- Use concrete wording tied to discovered context.
- Keep options actionable, not vague.
- Include one-sentence tradeoffs per option.
- Avoid leading questions that hide important downsides.
- Offer an `Other` path when free-form input is useful.

## Tool Usage

If a structured question tool is available (for example `AskUserQuestion` or `request_user_input`), use it.

Question payload should include:

- `header`: short label for the decision
- `question` or `text`: one focused prompt
- `options`: 2-4 choices with `label` and `description`
- Single select by default unless multi-select is truly required

If no tool is available, present the same structure in markdown and ask the user to reply with one option or free-form input.

## Example Triggers

- "Interview me about this feature."
- "Ask me questions before you implement."
- "Use the ask-user-question skill."
- "Requirements are unclear; ask me one question at a time."
