+++
rfc = "{{ rfc_id }}"
title = "{{ title_toml }}"
{% if agents -%}
agents = [{% for agent in agents %}"{{ agent }}"{% if not loop.last %}, {% endif %}{% endfor %}]
{% endif -%}
authors = [{% for author in authors %}"{{ author }}"{% if not loop.last %}, {% endif %}{% endfor %}]
created = "{{ timestamp }}"
last_updated = "{{ timestamp }}"
{% if discussion -%}
discussion = "{{ discussion }}"
{% endif -%}
{% if tracking_issue -%}
tracking_issue = "{{ tracking_issue }}"
{% endif -%}
{% if prerequisite -%}
prerequisite = [{% for rfc in prerequisite %}{{ rfc }}{% if not loop.last %}, {% endif %}{% endfor %}]
{% endif -%}
{% if supersedes -%}
supersedes = [{% for rfc in supersedes %}{{ rfc }}{% if not loop.last %}, {% endif %}{% endfor %}]
{% endif -%}
{% if superseded_by -%}
superseded_by = [{% for rfc in superseded_by %}{{ rfc }}{% if not loop.last %}, {% endif %}{% endfor %}]
{% endif -%}
[[revision]]
date = "{{ revision_timestamp }}"
change = "{{ revision_change }}"
+++

# RFC {{ rfc_id }}: {{ title }}

## Summary

*Briefly explain the proposal and intended outcome.*

## Motivation

*Why is this needed now? What user or project problem does it solve?*

## Guide-level explanation

*Explain the proposal as a user-facing concept with examples.*

## Reference-level explanation

*Describe the detailed technical design, edge cases, and interactions.*

## Reference implementation

*Link the implementation PR(s) and tracking issue when available.*

## Backwards compatibility

*List compatibility risks and migration guidance.*

## Security implications

*Call out security impact or state why there is none.*

## How to teach this

*Describe how to teach this to existing and new contributors.*

## Drawbacks

*Why might we choose not to do this?*

## Rationale and alternatives

*Why this design over alternatives? Include rejected ideas.*

## Prior art

*Related designs in other languages, projects, or papers.*

## Unresolved questions

*Open questions to resolve before or during implementation.*

## Future possibilities

*Potential follow-up work that is out of scope for this RFC.*
