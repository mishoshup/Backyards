use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use xtask::finding::{Finding, Severity};
use xtask::repository::Repository;

static FIXTURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub const CLEAN_MANIFEST: &str = "[package]\nname = \"x\"\n\n[lints]\nworkspace = true\n";

pub struct Fixture {
    root: PathBuf,
}

impl Fixture {
    pub fn create() -> Self {
        let number = FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let root =
            std::env::temp_dir().join(format!("xtask-fixture-{}-{number}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let status = Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .unwrap();
        assert!(status.success());
        Self { root }
    }

    pub fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    pub fn path(&self) -> &std::path::Path {
        &self.root
    }

    pub fn repository(&self) -> Repository {
        Repository::rooted_at(self.root.clone())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

pub fn error_messages(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .map(|finding| finding.message.clone())
        .collect()
}

pub fn any_message_contains(findings: &[Finding], needle: &str) -> bool {
    findings
        .iter()
        .any(|finding| finding.message.contains(needle))
}
