# AGENTS.md

## Principles

- less standalone function is better
- every module only expects a few names to be imported, do not create giant sets of new names
- if we have a lot of implementations (over 200 lines), it is better to split them into multiple files.
- use `mod.rs` over `<name>.rs` for modules that contain multiple files.
- if implementing a big new feature with fundamental changes, write an RFC first and keep interviewing the user until they are satisfied with the design.

## Build and Test

```bash
cargo build --workspace          # Build all crates
cargo test --workspace           # Run all tests
cargo test -p kirin-chumsky      # Test a single crate
cargo test -p kirin-chumsky-derive test_parse_add  # Run a single test
cargo fmt --all                  # Format code
cargo insta review               # Review snapshot test changes
```

Rust edition 2024. No `rust-toolchain.toml`; uses the default toolchain.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/): `<type>(<scope>): <description>`

Examples: `feat(chumsky): add region parser`, `fix(derive): handle empty enum variants`

Avoid large paragraphs in commit messages, keep them concise and focused on the changes made.
