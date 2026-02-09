use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    pub fn new(name: &str) -> Self {
        let mut root = std::env::temp_dir();
        let seq = WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        root.push(format!(
            "agx-tests-{name}-{}-{nanos}-{seq}",
            std::process::id()
        ));

        fs::create_dir_all(root.join("rfc")).expect("failed to create rfc directory");
        fs::write(
            root.join("rfc/0000-template.md"),
            include_str!("../../rfc/0000-template.md"),
        )
        .expect("failed to seed template file");

        Self { root }
    }

    pub fn path(&self) -> &Path {
        &self.root
    }

    pub fn run_cli(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_agx"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .expect("failed to execute agx")
    }

    pub fn run_cli_in(&self, relative_dir: &str, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_agx"))
            .current_dir(self.root.join(relative_dir))
            .args(args)
            .output()
            .expect("failed to execute agx")
    }

    pub fn run_rfc(&self, args: &[&str]) -> Output {
        let mut command_args = Vec::with_capacity(args.len() + 1);
        command_args.push("rfc");
        command_args.extend_from_slice(args);
        self.run_cli(&command_args)
    }

    pub fn run_rfc_init(&self) -> Output {
        self.run_rfc(&["init"])
    }

    pub fn run_rfc_new(&self, args: &[&str]) -> Output {
        let mut command_args = Vec::with_capacity(args.len() + 1);
        command_args.push("new");
        command_args.extend_from_slice(args);
        self.run_rfc(&command_args)
    }

    pub fn run_rfc_revise(&self, args: &[&str]) -> Output {
        let mut command_args = Vec::with_capacity(args.len() + 1);
        command_args.push("revise");
        command_args.extend_from_slice(args);
        self.run_rfc(&command_args)
    }

    pub fn run_rfc_in(&self, relative_dir: &str, args: &[&str]) -> Output {
        let mut command_args = Vec::with_capacity(args.len() + 1);
        command_args.push("rfc");
        command_args.extend_from_slice(args);
        self.run_cli_in(relative_dir, &command_args)
    }

    pub fn run_rfc_new_in(&self, relative_dir: &str, args: &[&str]) -> Output {
        let mut command_args = Vec::with_capacity(args.len() + 1);
        command_args.push("new");
        command_args.extend_from_slice(args);
        self.run_rfc_in(relative_dir, &command_args)
    }

    pub fn run_skill(&self, args: &[&str]) -> Output {
        let mut command_args = Vec::with_capacity(args.len() + 1);
        command_args.push("skill");
        command_args.extend_from_slice(args);
        self.run_cli(&command_args)
    }

    pub fn run_skill_init(&self) -> Output {
        self.run_skill(&["init"])
    }

    pub fn run_skill_new(&self, name: &str) -> Output {
        self.run_skill(&["new", name])
    }

    pub fn run_skill_validate(&self, name: Option<&str>) -> Output {
        match name {
            Some(skill_name) => self.run_skill(&["validate", skill_name]),
            None => self.run_skill(&["validate"]),
        }
    }

    pub fn run_git(&self, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .status()
            .expect("failed to execute git");
        assert!(
            status.success(),
            "git command failed: git {}",
            args.join(" ")
        );
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn output_stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn output_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}
