+++
rfc = "0001"
title = "Bundle Agent Skills with Releases and Expose Skill Discovery"
agents = ["codex"]
authors = ["Roger"]
created = "2026-02-09T20:40:04Z"
last_updated = "2026-02-09T21:00:52Z"
[[revision]]
date = "2026-02-09T20:40:04Z"
change = "Initial draft"

[[revision]]
date = "2026-02-09T20:46:12Z"
change = "Revised"

[[revision]]
date = "2026-02-09T20:49:35Z"
change = "Revised"

[[revision]]
date = "2026-02-09T21:00:52Z"
change = "Revised"
+++

# RFC 0001: Bundle Agent Skills with Releases and Expose Skill Discovery

## Summary

Adopt dual skill distribution for `agx`: compile and embed curated generic skills from `.agents/skills` into the binary at build time, and publish a companion skills tarball with each release. This lets users and other agents work from either the standalone binary path or explicit downloaded skill files.

This RFC introduces:

1. A build-generated built-in skill catalog sourced from `.agents/skills`, initially including `ask-user-question` and `new-rfc-skill-creation-skill`.
2. New machine-readable skill discovery via `agx skill list --format json`.
3. A human-first `agx skill dump` workflow that writes bundled skills into `.agents/use` in the current project.
4. Install/export commands so other CLI agents can discover and materialize skills by calling `agx` commands.

## Motivation

Today, `agx` can scaffold and validate skills (`src/skill/init.rs`, `src/skill/validate.rs`), but it does not package a curated set of reusable skills with releases.

Problems:

1. A downloaded binary is not enough to obtain existing shared skills unless users copy `.agents/skills` manually.
2. Other CLI agents cannot reliably discover skills through a stable machine-readable interface.
3. Release workflows cannot generate a deterministic "binary + skills" bundle from the same source of truth.
4. Adding a new generic skill currently risks manual code updates if skill payloads are hardcoded in Rust source.

## Guide-level explanation

Users get a predictable built-in skill set with every `agx` release.

Example flow for humans:

```bash
# show embedded and workspace skills
agx skill list --origin all

# dump all bundled skills to the standard human-use path under current project
agx skill dump --all
# default target: .agents/use

# export bundled skills into an archive next to release artifacts
agx skill export --origin builtin --output dist/agx-skills-v0.2.0.tar.gz
```

Example flow for another CLI agent:

```bash
# discover skills in JSON
agx skill list --origin all --format json

# materialize one skill into a temp directory it controls
agx skill install ask-user-question --origin builtin --to /tmp/agent-skills --format json
```

The external agent reads JSON output, chooses a skill, installs it, and then consumes the resulting directory path.

Contributor flow for incremental packaging:

```bash
# add or update a generic skill in the canonical source directory
agx skill validate ask-user-question

# add its name to the packaging manifest
$EDITOR .agents/skills/builtin-manifest.toml

# build embeds selected skills automatically
cargo build --workspace
```

## Reference-level explanation

### Scope and files

Current command routing and skill behavior are defined in:

1. `src/cli.rs`
2. `src/main.rs`
3. `src/skill/init.rs`
4. `src/skill/validate.rs`
5. `src/skill/mod.rs`

This RFC adds a built-in catalog plus four additive skill subcommands:

1. `skill list`
2. `skill dump`
3. `skill install`
4. `skill export`

### Built-in catalog design

Use `.agents/skills/<name>/...` as the source of truth for generic packaged skills.

Add a packaging manifest to select which repository skills are considered built-in:

1. `.agents/skills/builtin-manifest.toml`
2. Example shape: `skills = ["ask-user-question", "new-rfc-skill-creation-skill"]`

Add a build-time catalog generation pipeline:

1. `build.rs` reads `.agents/skills/builtin-manifest.toml`.
2. `build.rs` loads each selected `.agents/skills/<name>` directory.
3. `build.rs` validates each selected skill with the same structural rules used by `skill validate`.
4. `build.rs` writes a generated catalog artifact in `OUT_DIR` (for example `builtin_skills.json`).
5. `src/skill/builtin/mod.rs` loads that generated artifact with `include_str!` and exposes iteration APIs used by `list`, `install`, and `export`.

This removes skill-content copy/paste from Rust code. Contributors add or update skill files in `.agents/skills` and register names in one manifest.

### Terminology

To avoid ambiguity, this RFC separates compile-time and runtime terms:

1. Build source: `.agents/skills` plus `.agents/skills/builtin-manifest.toml`, consumed by `build.rs`.
2. Runtime origin: where a command reads skills from at runtime (`builtin`, `workspace`, or `all`).

### CLI contract

#### `agx skill list`

Options:

1. `--origin builtin|workspace|all` (default `all`)
2. `--format text|json` (default `text`)

JSON output includes a schema version and deterministic fields:

1. `name`
2. `description`
3. `builtin_available`
4. `workspace_path` (nullable)
5. `preferred_origin` (`workspace` when both exist, else `builtin`)

This is the primary discovery endpoint for other CLI agents.

#### `agx skill dump`

Human-oriented materialization command.

Behavior:

1. Dumps one skill by name or all bundled skills with `--all`.
2. Reads from runtime origin `builtin` only (no `--origin` flag).
3. Writes to `.agents/use` by default under the current project root.
4. Supports `--to <path>` to override the output path.
5. Refuses to overwrite existing files unless `--force` is passed.

#### `agx skill install`

Automation-oriented materialization command for other tools.

Behavior:

1. Installs one skill by name or all skills with `--all`.
2. Supports `--origin builtin` in this RFC scope.
3. Writes to `.agents/skills` by default, overridable by `--to <path>`.
4. Refuses to overwrite existing files unless `--force` is passed.
5. Supports `--format json` to return installed paths for automation.

Conflict policy:

1. If workspace skill exists and `--force` is not set, return a conflict error.
2. With `--force`, overwrite only files under the selected skill directory.

#### `agx skill export`

Behavior:

1. Exports built-in skills to a `.tar.gz` archive.
2. Preserves the `.agents/skills/<name>/...` layout.
3. Intended for release pipelines and offline transfer.

### Release packaging

This RFC chooses a dual-distribution model:

1. During `cargo build`, `build.rs` packages selected skills from `.agents/skills` into a generated built-in catalog embedded in the binary.
2. The binary exposes that catalog through `skill list`, `skill dump`, `skill install`, and `skill export`.
3. Release automation publishes `agx-skills-<version>.tar.gz` produced by `agx skill export --origin builtin`.
4. The skills tarball is shipped alongside each platform binary artifact.

The key guarantee is deterministic parity: `skill export --origin builtin`, `skill dump`, and `skill install --origin builtin` must use the same embedded catalog source.

### Edge cases and failure modes

1. Missing built-in files at compile time fail fast during build.
2. Manifest entries that point to missing skill directories fail the build.
3. Invalid built-in frontmatter fails build-time validation and blocks release.
4. Name collisions between built-in and workspace skills resolve to workspace for discovery; install requires explicit overwrite.
5. `skill dump` and `skill install` return non-zero status on partial writes and print failed paths.
6. `skill dump` fails when run outside a project root unless `--to` is explicitly provided.

### Implementation slices (dependency order)

1. Add `.agents/skills/builtin-manifest.toml` and `build.rs` catalog generation.
2. Add `src/skill/builtin/mod.rs` catalog API over generated `OUT_DIR` data.
3. Add CLI argument structs and subcommands in `src/cli.rs`.
4. Add command handlers and wiring in `src/main.rs` and `src/skill/mod.rs`.
5. Implement `list` with JSON/text output and `origin` semantics.
6. Implement `dump` with `.agents/use` default target behavior.
7. Implement `install` with conflict policy and optional JSON output.
8. Implement `export` archive writer and release workflow integration.
9. Update `README.md` with separate human (`dump`) and agent (`install`) usage examples.

### Validation plan

1. `cargo fmt --all`
2. `cargo build --workspace`
3. `cargo test --workspace`
4. `cargo insta review` when snapshots change
5. `agx skill validate` against installed exported fixtures in integration tests

## Reference implementation

Tracking issue: TBD.

Planned implementation areas:

1. `src/cli.rs`
2. `src/main.rs`
3. `src/skill/mod.rs`
4. `src/skill/builtin/mod.rs` (new)
5. `src/skill/list.rs` (new)
6. `src/skill/dump.rs` (new)
7. `src/skill/install.rs` (new)
8. `src/skill/export.rs` (new)
9. `build.rs` (new)
10. `.agents/skills/builtin-manifest.toml` (new)
11. `README.md`

## Backwards compatibility

No migration required.

Existing commands (`skill init`, `skill new`, `skill validate`, and all `rfc` commands) remain unchanged. New commands are additive.

Compatibility risks:

1. JSON output is a new contract; changing field names later would break agent integrations.
2. If built-in skill names overlap local custom skills, install may fail without `--force` by design.

## Security implications

Security impact is low but non-zero.

1. File writes are constrained to explicit target roots and validated skill names.
2. Install/export never execute skill content; they only read/write text files.
3. `--force` is explicit to reduce accidental overwrite.
4. Optional future hardening: publish checksums for exported archives and verify them in CI.

## How to teach this

Teach three concepts:

1. Built-in skills are curated reusable assets shipped with `agx`.
2. `agx skill list --format json` is the supported machine discovery API.
3. `agx skill dump` is the standard human workflow and writes to `.agents/use` by default.
4. `agx skill install` remains the automation-oriented materialization command.

Documentation updates:

1. Add a "Bundled skills" section to `README.md`.
2. Add examples for human and agent usage.
3. Add release notes template text describing binary and skill archive artifacts.

## Drawbacks

1. Binary size increases because skill payloads are embedded.
2. We now maintain curated generic skills as release artifacts with compatibility expectations.
3. Build complexity increases due to manifest-driven catalog generation.
4. More CLI surface area means more long-term contract maintenance.

## Rationale and alternatives

Chosen design: embed a curated built-in catalog, publish a companion skills tarball, and expose additive list/install/export commands.

Alternative 1: Manual hardcoded skill payloads in Rust source.

1. Pros: no build-script generation logic.
2. Cons: high maintenance, easy to drift from `.agents/skills`, and does not scale for incremental contributor skill additions.

Alternative 2: Keep skills only in repository files.

1. Pros: zero new CLI/API surface.
2. Cons: users downloading binaries do not get skills, and agents cannot discover skills reliably.

Alternative 3: Publish only a sidecar archive, no embedded catalog.

1. Pros: smaller binary.
2. Cons: `agx` cannot install built-ins by itself when archive is missing; weaker UX and automation story.

Alternative 4: Remote skill registry/service.

1. Pros: dynamic updates.
2. Cons: network dependency, auth complexity, and out-of-scope operational burden for current project stage.

## Prior art

1. CLI tools that expose machine-readable inventories (`--json`) for automation workflows.
2. Release layouts that place reusable assets under `share/` alongside binaries.
3. Plugin ecosystems where discovery and installation are separate commands (`list` then `install`).

## Unresolved questions

1. Should `new-rfc` also be bundled now, or remain project-specific?
2. Do we guarantee JSON schema compatibility by semantic version, or by explicit `schema_version` field only?
3. Should `skill install` support `--origin archive:<path>` in the initial implementation?

## Future possibilities

1. Signature verification for skill archives.
2. Compatibility metadata per skill (for example minimum `agx` version).
3. Additional machine endpoints (`skill show`, `skill doctor`) for richer agent integration.
