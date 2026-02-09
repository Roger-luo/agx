mod common;

use std::{fs, path::Path};

use common::{TestWorkspace, output_stderr, output_stdout};

fn write_template(path: &Path, marker: &str) {
    let template = r#"+++
rfc = "{{ rfc_id }}"
title = "{{ title_toml }}"
authors = [{% for author in authors %}"{{ author }}"{% if not loop.last %}, {% endif %}{% endfor %}]
created = "{{ timestamp }}"
last_updated = "{{ timestamp }}"
[[revision]]
date = "{{ revision_timestamp }}"
change = "{{ revision_change }}"
+++

# RFC {{ rfc_id }}: {{ title }}

__MARKER__
"#;
    fs::write(path, template.replace("__MARKER__", marker)).expect("failed to write template");
}

fn last_updated_timestamp(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        line.trim()
            .strip_prefix("last_updated = \"")
            .and_then(|value| value.strip_suffix('"'))
            .map(ToOwned::to_owned)
    })
}

fn latest_revision_timestamp(markdown: &str) -> Option<String> {
    let mut latest = None;
    for line in markdown.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed
            .strip_prefix("date = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            latest = Some(value.to_owned());
        }
    }
    latest
}

#[test]
fn create_mode_writes_expected_metadata_and_heading() {
    let workspace = TestWorkspace::new("create-mode");
    let output = workspace.run_rfc_new(&[
        "--author",
        "Roger",
        "--agent",
        "codex",
        "--discussion",
        "DISC-123",
        "--tracking_issue",
        "ISSUE-42",
        "--prerequisite",
        "0000",
        "--supersedes",
        "0000",
        "--superseded_by",
        "0002",
        "--title",
        "Example RFC",
        "ignored-positional",
    ]);

    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );
    assert_eq!(output_stdout(&output).trim(), "rfc/0001-example-rfc.md");

    let file = workspace.path().join("rfc/0001-example-rfc.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("rfc = \"0001\""));
    assert!(content.contains("title = \"Example RFC\""));
    assert!(content.contains("authors = [\"Roger\"]"));
    assert!(content.contains("agents = [\"codex\"]"));
    assert!(content.contains("discussion = \"DISC-123\""));
    assert!(content.contains("tracking_issue = \"ISSUE-42\""));
    assert!(content.contains("prerequisite = [0]"));
    assert!(content.contains("supersedes = [0]"));
    assert!(content.contains("superseded_by = [2]"));
    assert!(content.contains("[[revision]]"));
    assert!(content.contains("change = \"Initial draft\""));
    assert!(content.contains("# RFC 0001: Example RFC"));
    assert!(content.contains("## Guide-level explanation"));
    assert!(content.contains("## Reference-level explanation"));
    assert!(content.contains("## Backwards compatibility"));
    assert!(content.contains("## Security implications"));
    assert!(content.contains("## How to teach this"));
    assert!(content.contains("## Future possibilities"));
}

#[test]
fn create_mode_resolves_title_references_to_rfc_ids() {
    let workspace = TestWorkspace::new("title-references");
    let base = workspace.run_rfc_new(&["--author", "Roger", "--title", "Base RFC"]);
    assert!(
        base.status.success(),
        "base create failed:\n{}",
        output_stderr(&base)
    );

    let output = workspace.run_rfc_new(&[
        "--author",
        "Roger",
        "--prerequisite",
        "Base RFC",
        "--supersedes",
        "Base RFC",
        "--superseded_by",
        "Base RFC",
        "--title",
        "Dependent RFC",
    ]);
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );

    let file = workspace.path().join("rfc/0002-dependent-rfc.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("prerequisite = [1]"));
    assert!(content.contains("supersedes = [1]"));
    assert!(content.contains("superseded_by = [1]"));
}

#[test]
fn create_mode_rejects_duplicate_title() {
    let workspace = TestWorkspace::new("duplicate-title");

    let first = workspace.run_rfc_new(&["--author", "Roger", "--title", "Existing RFC"]);
    assert!(
        first.status.success(),
        "first create failed:\n{}",
        output_stderr(&first)
    );

    let duplicate = workspace.run_rfc_new(&["--author", "Roger", "--title", "Existing RFC"]);
    assert!(
        !duplicate.status.success(),
        "duplicate create unexpectedly succeeded"
    );

    let stderr = output_stderr(&duplicate);
    assert!(stderr.contains("already exists"));
    assert!(stderr.contains("Existing RFC"));
}

#[test]
fn create_mode_rejects_numeric_only_title() {
    let workspace = TestWorkspace::new("numeric-create-title");
    let output = workspace.run_rfc_new(&["--author", "Roger", "0003"]);

    assert!(!output.status.success(), "command unexpectedly succeeded");

    let stderr = output_stderr(&output);
    assert!(stderr.contains("numeric-only title"));
    assert!(!workspace.path().join("rfc/0001-0003.md").exists());
}

#[test]
fn create_mode_uses_title_parts() {
    let workspace = TestWorkspace::new("title-parts");
    let output = workspace.run_rfc_new(&["--author", "Roger", "--title_parts", "agent", "writer"]);

    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );
    assert_eq!(output_stdout(&output).trim(), "rfc/0001-agent-writer.md");

    let file = workspace.path().join("rfc/0001-agent-writer.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("title = \"agent_writer\""));
    assert!(content.contains("# RFC 0001: agent_writer"));
}

#[test]
fn create_mode_uses_git_author_when_author_flag_missing() {
    let workspace = TestWorkspace::new("git-author");
    workspace.run_git(&["init", "."]);
    workspace.run_git(&["config", "user.name", "Local Author"]);

    let output = workspace.run_rfc_new(&["RFC from git author"]);
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );
    assert_eq!(
        output_stdout(&output).trim(),
        "rfc/0001-rfc-from-git-author.md"
    );

    let file = workspace.path().join("rfc/0001-rfc-from-git-author.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("authors = [\"Local Author\"]"));
}

#[test]
fn revision_mode_appends_lists_overwrites_fields_and_adds_revision_entry() {
    let workspace = TestWorkspace::new("revision-mode");

    let create = workspace.run_rfc_new(&["--author", "Roger", "Original RFC"]);
    assert!(
        create.status.success(),
        "initial create failed:\n{}",
        output_stderr(&create)
    );

    let revise = workspace.run_rfc_revise(&[
        "--author",
        "Alice",
        "--author",
        "Roger",
        "--agent",
        "codex",
        "--discussion",
        "DISC-999",
        "--tracking_issue",
        "ISSUE-999",
        "--prerequisite",
        "0000",
        "--supersedes",
        "0000",
        "--superseded_by",
        "0002",
        "--title",
        "Original RFC Updated",
        "0001",
    ]);
    assert!(
        revise.status.success(),
        "revision failed:\n{}",
        output_stderr(&revise)
    );
    assert_eq!(output_stdout(&revise).trim(), "rfc/0001-original-rfc.md");

    let file = workspace.path().join("rfc/0001-original-rfc.md");
    let content = fs::read_to_string(file).expect("failed to read revised RFC");
    assert!(content.contains("title = \"Original RFC Updated\""));
    assert!(content.contains("authors = [\"Roger\", \"Alice\"]"));
    assert!(content.contains("agents = [\"codex\"]"));
    assert!(content.contains("discussion = \"DISC-999\""));
    assert!(content.contains("tracking_issue = \"ISSUE-999\""));
    assert!(content.contains("prerequisite = [0]"));
    assert!(content.contains("supersedes = [0]"));
    assert!(content.contains("superseded_by = [2]"));
    assert!(content.contains("# RFC 0001: Original RFC Updated"));
    assert!(content.contains("change = \"Initial draft\""));
    assert!(content.contains("change = \"Revised\""));
    assert_eq!(content.matches("[[revision]]").count(), 2);
    assert_eq!(
        latest_revision_timestamp(&content),
        last_updated_timestamp(&content)
    );
}

#[test]
fn revision_mode_accepts_numeric_selector_as_rfc_id() {
    let workspace = TestWorkspace::new("revision-id-selector");
    let create = workspace.run_rfc_new(&["--author", "Roger", "Original RFC"]);
    assert!(
        create.status.success(),
        "initial create failed:\n{}",
        output_stderr(&create)
    );

    let revise = workspace.run_rfc_revise(&["1"]);
    assert!(
        revise.status.success(),
        "revision failed:\n{}",
        output_stderr(&revise)
    );
    assert_eq!(output_stdout(&revise).trim(), "rfc/0001-original-rfc.md");

    let file = workspace.path().join("rfc/0001-original-rfc.md");
    let content = fs::read_to_string(file).expect("failed to read revised RFC");
    assert!(content.contains("change = \"Revised\""));
}

#[test]
fn create_mode_requires_some_title_input() {
    let workspace = TestWorkspace::new("missing-title");
    let output = workspace.run_rfc_new(&["--author", "Roger"]);

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(output_stderr(&output).contains("missing <title>"));
}

#[test]
fn rfc_new_help_output_includes_command_docs_and_examples() {
    let workspace = TestWorkspace::new("help-output");
    let output = workspace.run_rfc(&["new", "--help"]);

    assert!(
        output.status.success(),
        "help command failed:\n{}",
        output_stderr(&output)
    );

    let help = output_stdout(&output);
    assert!(help.contains("Create a new RFC markdown file with TOML metadata."));
    assert!(help.contains("Creates a new RFC file from `rfc/0000-template.md`"));
    assert!(help.contains("Add an author to metadata."));
    assert!(help.contains("List prerequisite RFC references (id or title)"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("agx rfc new --author Roger --title \"Add parser support\""));
}

#[test]
fn root_help_lists_rfc_init_and_skill_subcommands() {
    let workspace = TestWorkspace::new("root-help-output");
    let output = workspace.run_cli(&["--help"]);

    assert!(
        output.status.success(),
        "help command failed:\n{}",
        output_stderr(&output)
    );

    let help = output_stdout(&output);
    assert!(help.contains("Manage agent workflow tooling"));
    assert!(help.contains("\n  rfc "));
    assert!(help.contains("\n  skill "));
    assert!(!help.contains("\n  init "));
}

#[test]
fn create_mode_prefers_template_from_crate_root() {
    let workspace = TestWorkspace::new("crate-template");
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname = \"crate-template\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("failed to write crate manifest");
    write_template(
        &workspace.path().join("rfc/0000-template.md"),
        "crate-root-template",
    );

    let output = workspace.run_rfc_new(&["--author", "Roger", "Crate Template"]);
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );

    let file = workspace.path().join("rfc/0001-crate-template.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("crate-root-template"));
}

#[test]
fn create_mode_prefers_workspace_root_template_from_member_crate() {
    let workspace = TestWorkspace::new("workspace-template");
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\"]\nresolver = \"2\"\n",
    )
    .expect("failed to write workspace manifest");
    fs::create_dir_all(workspace.path().join("crates/member/rfc"))
        .expect("failed to create member rfc directory");
    fs::write(
        workspace.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("failed to write member manifest");
    write_template(
        &workspace.path().join("rfc/0000-template.md"),
        "workspace-root-template",
    );

    let output = workspace.run_rfc_new_in("crates/member", &["--author", "Roger", "Workspace RFC"]);
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );

    let file = workspace
        .path()
        .join("crates/member/rfc/0001-workspace-rfc.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("workspace-root-template"));
}

#[test]
fn member_crate_resolves_reference_titles_from_workspace_rfc_directory() {
    let workspace = TestWorkspace::new("workspace-reference-resolution");
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\"]\nresolver = \"2\"\n",
    )
    .expect("failed to write workspace manifest");
    fs::create_dir_all(workspace.path().join("crates/member/rfc"))
        .expect("failed to create member rfc directory");
    fs::write(
        workspace.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("failed to write member manifest");

    let base = workspace.run_rfc_new(&["--author", "Roger", "--title", "Workspace Base"]);
    assert!(
        base.status.success(),
        "base create failed:\n{}",
        output_stderr(&base)
    );

    let output = workspace.run_rfc_new_in(
        "crates/member",
        &[
            "--author",
            "Roger",
            "--prerequisite",
            "Workspace Base",
            "--title",
            "Member RFC",
        ],
    );
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );

    let file = workspace
        .path()
        .join("crates/member/rfc/0001-member-rfc.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("prerequisite = [1]"));
}

#[test]
fn create_mode_prefers_workspace_template_over_member_template() {
    let workspace = TestWorkspace::new("workspace-precedence");
    fs::write(
        workspace.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/member\"]\nresolver = \"2\"\n",
    )
    .expect("failed to write workspace manifest");
    fs::create_dir_all(workspace.path().join("crates/member/rfc"))
        .expect("failed to create member rfc directory");
    fs::write(
        workspace.path().join("crates/member/Cargo.toml"),
        "[package]\nname = \"member\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("failed to write member manifest");
    write_template(
        &workspace.path().join("rfc/0000-template.md"),
        "workspace-root-template",
    );
    write_template(
        &workspace.path().join("crates/member/rfc/0000-template.md"),
        "member-template",
    );

    let output =
        workspace.run_rfc_new_in("crates/member", &["--author", "Roger", "Workspace Wins"]);
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );

    let file = workspace
        .path()
        .join("crates/member/rfc/0001-workspace-wins.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("workspace-root-template"));
    assert!(!content.contains("member-template"));
}

#[test]
fn create_mode_falls_back_to_embedded_template_when_project_template_missing() {
    let workspace = TestWorkspace::new("embedded-template");
    fs::remove_file(workspace.path().join("rfc/0000-template.md"))
        .expect("failed to remove seeded template");

    let output = workspace.run_rfc_new(&["--author", "Roger", "Embedded Template"]);
    assert!(
        output.status.success(),
        "command failed:\n{}",
        output_stderr(&output)
    );

    let file = workspace.path().join("rfc/0001-embedded-template.md");
    let content = fs::read_to_string(file).expect("failed to read created RFC");
    assert!(content.contains("## Future possibilities"));
}

#[test]
fn rfc_init_subcommand_creates_required_directories_and_skill() {
    let workspace = TestWorkspace::new("init-subcommand");
    fs::remove_dir_all(workspace.path().join("rfc")).expect("failed to remove rfc directory");
    assert!(!workspace.path().join(".agents").exists());
    assert!(!workspace.path().join(".agents/skills").exists());

    let output = workspace.run_rfc_init();
    assert!(
        output.status.success(),
        "rfc init command failed:\n{}",
        output_stderr(&output)
    );
    assert!(workspace.path().join("rfc").is_dir());
    assert!(workspace.path().join(".agents/skills").is_dir());
    assert!(
        workspace
            .path()
            .join(".agents/skills/create-rfc/SKILL.md")
            .is_file()
    );
    assert!(
        workspace
            .path()
            .join(".agents/skills/create-rfc/agents/openai.yaml")
            .is_file()
    );

    let stdout = output_stdout(&output);
    assert!(stdout.contains("rfc"));
    assert!(stdout.contains(".agents/skills"));
    assert!(stdout.contains(".agents/skills/create-rfc"));
}

#[test]
fn skill_init_creates_skills_root() {
    let workspace = TestWorkspace::new("skill-init");
    assert!(!workspace.path().join(".agents").exists());

    let output = workspace.run_skill_init();
    assert!(
        output.status.success(),
        "skill init command failed:\n{}",
        output_stderr(&output)
    );

    assert!(workspace.path().join(".agents/skills").is_dir());
    assert!(
        !workspace
            .path()
            .join(".agents/skills/ask-user-question")
            .exists()
    );
}

#[test]
fn skill_new_scaffolds_named_skill() {
    let workspace = TestWorkspace::new("skill-new");

    let output = workspace.run_skill_new("ask-user-question");
    assert!(
        output.status.success(),
        "skill new command failed:\n{}",
        output_stderr(&output)
    );

    let skill_dir = workspace.path().join(".agents/skills/ask-user-question");
    assert!(skill_dir.is_dir());
    assert!(skill_dir.join("agents").is_dir());

    let skill_md = fs::read_to_string(skill_dir.join("SKILL.md")).expect("failed to read SKILL.md");
    assert!(skill_md.contains("name: ask-user-question"));
    assert!(skill_md.contains("description:"));

    let openai_yaml = fs::read_to_string(skill_dir.join("agents/openai.yaml"))
        .expect("failed to read openai.yaml");
    assert!(openai_yaml.contains("interface:"));
}

#[test]
fn skill_validate_succeeds_for_initialized_skill() {
    let workspace = TestWorkspace::new("skill-validate-ok");

    let new_skill = workspace.run_skill_new("ask-user-question");
    assert!(new_skill.status.success(), "{}", output_stderr(&new_skill));

    let validate = workspace.run_skill_validate(None);
    assert!(
        validate.status.success(),
        "skill validate command failed:\n{}",
        output_stderr(&validate)
    );

    let stdout = output_stdout(&validate);
    assert!(stdout.contains("ok .agents/skills/ask-user-question"));
    assert!(stdout.contains("validated 1 skill(s)"));
}

#[test]
fn skill_validate_rejects_invalid_skill() {
    let workspace = TestWorkspace::new("skill-validate-bad");
    let bad_skill = workspace.path().join(".agents/skills/bad-skill");
    fs::create_dir_all(&bad_skill).expect("failed to create bad skill directory");
    fs::write(
        bad_skill.join("SKILL.md"),
        "---\nname: bad-skill\n---\n\n# Bad Skill\n",
    )
    .expect("failed to write SKILL.md");

    let output = workspace.run_skill_validate(Some("bad-skill"));
    assert!(
        !output.status.success(),
        "skill validate unexpectedly succeeded"
    );

    let stderr = output_stderr(&output);
    assert!(stderr.contains("missing required `description`"));
}
