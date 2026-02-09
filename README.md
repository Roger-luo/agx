# agx

`agx` is a Rust CLI for project-local RFC and skill workflows.

## Install

Latest release:

```bash
curl -fsSL https://github.com/Roger-luo/agx/releases/latest/download/install.sh | sh
```

Specific version:

```bash
curl -fsSL https://github.com/Roger-luo/agx/releases/download/v0.1.0/install.sh | sh
```

Optional installer variables:

- `AGX_INSTALL_DIR` (default: `~/.local/bin`)
- `AGX_REPO` (default: `Roger-luo/agx`)
- `AGX_BIN_NAME` (default: `agx`)
- `AGX_VERSION` (default: `latest`)

## Case Study: Adopt RFC workflow in one repo

Scenario: your repo has no RFC process yet, and you want one proposal created and revised.

```bash
# 1) Initialize built-in skills and RFC folder/template.
agx skill init
agx rfc init

# 2) Create a proposal with metadata.
agx rfc new \
  --author "Roger Luo" \
  --title "Add CI release pipeline" \
  --discussion "https://github.com/Roger-luo/agx/discussions/1"

# 3) Revise the same RFC after review feedback.
agx rfc revise 0001 --title "Add CI/CD release pipeline"
```

Result:

- `rfc/0000-template.md` exists.
- a new RFC file (for example `rfc/0001-add-ci-release-pipeline.md`) is created.
- the RFC is revised in place with updated heading and metadata history.

Use `agx --help`, `agx rfc --help`, and `agx skill --help` for full command options.

## Build and Test

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all
```

## Release Artifacts

Tag releases publish signed archives for:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Each target includes:

- `agx-<target>.tar.gz`
- `agx-<target>.tar.gz.sha256`
- `agx-<target>.tar.gz.sig`
