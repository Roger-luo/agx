mod common;

use std::{fs, io::Read, path::Path};

use common::{TestWorkspace, output_stderr, output_stdout};
use flate2::read::GzDecoder;
use serde_json::Value;
use tar::Archive;

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

fn write_package_manifest(root: &Path) {
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"skill-tests\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("failed to write Cargo.toml");
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
fn rfc_init_requires_skills_root_and_hints_skill_dump() {
    let workspace = TestWorkspace::new("init-subcommand-requires-skills");
    fs::remove_dir_all(workspace.path().join("rfc")).expect("failed to remove rfc directory");
    assert!(!workspace.path().join(".agents").exists());
    assert!(!workspace.path().join(".agents/skills").exists());

    let output = workspace.run_rfc_init();
    assert!(!output.status.success(), "rfc init unexpectedly succeeded");

    let stderr = output_stderr(&output);
    assert!(stderr.contains(".agents/skills"));
    assert!(stderr.contains("agx skill dump --all"));
    assert!(!workspace.path().join("rfc").exists());
}

#[test]
fn rfc_init_succeeds_when_skills_root_exists() {
    let workspace = TestWorkspace::new("init-subcommand-success");
    fs::remove_dir_all(workspace.path().join("rfc")).expect("failed to remove rfc directory");
    fs::create_dir_all(workspace.path().join(".agents/skills"))
        .expect("failed to create skills root");

    let output = workspace.run_rfc_init();
    assert!(
        output.status.success(),
        "rfc init command failed:\n{}",
        output_stderr(&output)
    );

    assert!(workspace.path().join("rfc").is_dir());
    assert!(
        workspace.path().join("rfc/0000-template.md").is_file(),
        "rfc init should materialize the embedded template"
    );
    let template = fs::read_to_string(workspace.path().join("rfc/0000-template.md"))
        .expect("failed to read materialized template");
    assert!(template.contains("## Future possibilities"));
    assert!(workspace.path().join(".agents/skills").is_dir());
    assert!(
        !workspace
            .path()
            .join(".agents/skills/create-rfc/SKILL.md")
            .exists()
    );
    assert_eq!(output_stdout(&output).trim(), "rfc");
}

#[test]
fn rfc_init_does_not_overwrite_existing_template() {
    let workspace = TestWorkspace::new("init-subcommand-no-overwrite-template");
    fs::create_dir_all(workspace.path().join(".agents/skills"))
        .expect("failed to create skills root");
    fs::write(
        workspace.path().join("rfc/0000-template.md"),
        "+++\ncustom = true\n+++\n\n# custom template\n",
    )
    .expect("failed to write custom template");

    let output = workspace.run_rfc_init();
    assert!(
        output.status.success(),
        "rfc init command failed:\n{}",
        output_stderr(&output)
    );

    let template = fs::read_to_string(workspace.path().join("rfc/0000-template.md"))
        .expect("failed to read template");
    assert!(template.contains("custom template"));
    assert!(!template.contains("## Future possibilities"));
}

#[test]
fn skill_init_creates_skills_root_and_seeds_builtins() {
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
        workspace
            .path()
            .join(".agents/skills/ask-user-question/SKILL.md")
            .is_file()
    );
    assert!(
        workspace
            .path()
            .join(".agents/skills/new-rfc-skill-creation-skill/SKILL.md")
            .is_file()
    );
    assert!(
        workspace
            .path()
            .join(".agents/skills/new-rfc-skill-creation-skill/references/rfc-skill-template.md")
            .is_file()
    );

    let stdout = output_stdout(&output);
    assert!(stdout.contains("use the code agent"));
    assert!(stdout.contains("RFC skills"));
    assert!(stdout.contains("recommended prompt"));
    assert!(stdout.contains("> Use $new-rfc-skill-creation-skill"));
    assert!(stdout.contains("named `new-rfc`"));
    assert!(stdout.contains("feedback"));
    assert!(stdout.contains("copied recommended prompt to clipboard"));
}

#[test]
fn skill_init_no_dump_creates_only_skills_root() {
    let workspace = TestWorkspace::new("skill-init-no-dump");
    assert!(!workspace.path().join(".agents").exists());

    let output = workspace.run_skill(&["init", "--no-dump"]);
    assert!(
        output.status.success(),
        "skill init --no-dump command failed:\n{}",
        output_stderr(&output)
    );

    assert!(workspace.path().join(".agents/skills").is_dir());
    assert!(
        !workspace
            .path()
            .join(".agents/skills/ask-user-question/SKILL.md")
            .exists()
    );
    assert!(
        !workspace
            .path()
            .join(".agents/skills/new-rfc-skill-creation-skill/SKILL.md")
            .exists()
    );

    let stdout = output_stdout(&output);
    assert!(stdout.contains("use the code agent"));
    assert!(stdout.contains("RFC skills"));
    assert!(stdout.contains("recommended prompt"));
    assert!(stdout.contains("> Use $new-rfc-skill-creation-skill"));
    assert!(stdout.contains("named `new-rfc`"));
    assert!(stdout.contains("feedback"));
    assert!(stdout.contains("copied recommended prompt to clipboard"));
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

#[test]
fn skill_list_builtin_json_includes_schema_and_expected_entries() {
    let workspace = TestWorkspace::new("skill-list-builtin-json");
    let output = workspace.run_skill_list(&["--origin", "builtin", "--format", "json"]);

    assert!(
        output.status.success(),
        "skill list command failed:\n{}",
        output_stderr(&output)
    );

    let payload: Value =
        serde_json::from_str(&output_stdout(&output)).expect("failed to parse JSON output");
    assert_eq!(payload["schema_version"].as_u64(), Some(1));

    let skills = payload["skills"]
        .as_array()
        .expect("skills must be an array");
    assert!(skills.iter().any(|entry| {
        entry["name"] == "ask-user-question"
            && entry["builtin_available"] == true
            && entry["workspace_path"].is_null()
            && entry["preferred_origin"] == "builtin"
    }));
    assert!(skills.iter().any(|entry| {
        entry["name"] == "new-rfc-skill-creation-skill" && entry["builtin_available"] == true
    }));
}

#[test]
fn skill_list_all_prefers_workspace_when_name_collides() {
    let workspace = TestWorkspace::new("skill-list-collision");
    let new_skill = workspace.run_skill_new("ask-user-question");
    assert!(new_skill.status.success(), "{}", output_stderr(&new_skill));

    let output = workspace.run_skill_list(&["--origin", "all", "--format", "json"]);
    assert!(
        output.status.success(),
        "skill list command failed:\n{}",
        output_stderr(&output)
    );

    let payload: Value =
        serde_json::from_str(&output_stdout(&output)).expect("failed to parse JSON output");
    let entry = payload["skills"]
        .as_array()
        .expect("skills must be an array")
        .iter()
        .find(|entry| entry["name"] == "ask-user-question")
        .expect("missing ask-user-question entry");

    assert_eq!(entry["preferred_origin"], "workspace");
    assert_eq!(entry["builtin_available"], true);
    assert!(
        entry["workspace_path"]
            .as_str()
            .expect("workspace path should be a string")
            .contains(".agents/skills/ask-user-question")
    );
}

#[test]
fn skill_dump_all_writes_to_default_agents_skills_path() {
    let workspace = TestWorkspace::new("skill-dump-default");
    write_package_manifest(workspace.path());

    let output = workspace.run_skill_dump(&["--all"]);
    assert!(
        output.status.success(),
        "skill dump command failed:\n{}",
        output_stderr(&output)
    );

    assert!(
        workspace
            .path()
            .join(".agents/skills/ask-user-question/SKILL.md")
            .is_file()
    );
    assert!(
        workspace
            .path()
            .join(".agents/skills/new-rfc-skill-creation-skill/references/rfc-skill-template.md")
            .is_file()
    );
}

#[test]
fn skill_dump_requires_to_when_not_in_project_root() {
    let workspace = TestWorkspace::new("skill-dump-no-project-root");
    let output = workspace.run_skill_dump(&["--all"]);

    assert!(
        !output.status.success(),
        "skill dump unexpectedly succeeded"
    );
    assert!(output_stderr(&output).contains("could not determine a project root"));
}

#[test]
fn skill_install_json_outputs_installed_paths() {
    let workspace = TestWorkspace::new("skill-install-json");
    let output = workspace.run_skill_install(&[
        "ask-user-question",
        "--origin",
        "builtin",
        "--to",
        "installed-skills",
        "--format",
        "json",
    ]);

    assert!(
        output.status.success(),
        "skill install command failed:\n{}",
        output_stderr(&output)
    );

    let payload: Value =
        serde_json::from_str(&output_stdout(&output)).expect("failed to parse JSON output");
    assert_eq!(payload["schema_version"].as_u64(), Some(1));
    assert_eq!(payload["installed"][0]["name"], "ask-user-question");
    assert!(
        payload["installed"][0]["path"]
            .as_str()
            .expect("path should be a string")
            .contains("installed-skills/ask-user-question")
    );
    assert!(
        workspace
            .path()
            .join("installed-skills/ask-user-question/SKILL.md")
            .is_file()
    );
}

#[test]
fn skill_install_refuses_conflict_without_force() {
    let workspace = TestWorkspace::new("skill-install-conflict");

    let new_skill = workspace.run_skill_new("ask-user-question");
    assert!(new_skill.status.success(), "{}", output_stderr(&new_skill));

    let output = workspace.run_skill_install(&["ask-user-question"]);
    assert!(
        !output.status.success(),
        "skill install unexpectedly succeeded"
    );
    assert!(output_stderr(&output).contains("use --force to overwrite"));

    let forced = workspace.run_skill_install(&["ask-user-question", "--force"]);
    assert!(
        forced.status.success(),
        "skill install with --force failed:\n{}",
        output_stderr(&forced)
    );
}

#[test]
fn skill_export_writes_tarball_with_expected_layout() {
    let workspace = TestWorkspace::new("skill-export");
    let output = workspace.run_skill_export(&[
        "--origin",
        "builtin",
        "--output",
        "dist/agx-skills-v0.1.0.tar.gz",
    ]);

    assert!(
        output.status.success(),
        "skill export command failed:\n{}",
        output_stderr(&output)
    );

    let archive_path = workspace.path().join("dist/agx-skills-v0.1.0.tar.gz");
    assert!(archive_path.is_file());

    let archive_file = fs::File::open(&archive_path).expect("failed to open exported archive");
    let decoder = GzDecoder::new(archive_file);
    let mut archive = Archive::new(decoder);
    let mut found_skill_md = false;
    let mut found_reference = false;

    for entry in archive.entries().expect("failed to read archive entries") {
        let mut entry = entry.expect("failed to read archive entry");
        let path = entry
            .path()
            .expect("entry path should be valid")
            .to_string_lossy()
            .into_owned();

        if path == ".agents/skills/ask-user-question/SKILL.md" {
            found_skill_md = true;
            let mut content = String::new();
            entry
                .read_to_string(&mut content)
                .expect("failed to read skill markdown from archive");
            assert!(content.contains("name: ask-user-question"));
        }
        if path == ".agents/skills/new-rfc-skill-creation-skill/references/rfc-skill-template.md" {
            found_reference = true;
        }
    }

    assert!(
        found_skill_md,
        "expected ask-user-question SKILL.md in archive"
    );
    assert!(
        found_reference,
        "expected bundled reference file in archive layout"
    );
}
