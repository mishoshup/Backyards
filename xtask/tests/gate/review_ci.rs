use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::fake_model_server::{FakeModelServer, Script, content_event, finish_event};
use crate::fixture::Fixture;

static RUN_COUNTER: AtomicUsize = AtomicUsize::new(0);

const RULES_DIRECTORY: &str = ".agents/skills/rust-coding-standards/rules";
const NEW_FILE_DIFF: &str = "diff --git a/src/a.rs b/src/a.rs\nnew file mode 100644\n--- /dev/null\n+++ b/src/a.rs\n@@ -0,0 +1,2 @@\n+fn a() {}\n+fn b() {}\n";
const FINDING_ANSWER: &str = r#"{"violations":[{"ruleId":"PERF-01","path":"src/a.rs","line":1,"body":"PERF-01 borrow instead","confidence":"high"}]}"#;

fn prepare_repository() -> Fixture {
    let fixture = Fixture::create();
    fixture.write(
        &format!("{RULES_DIRECTORY}/perf-01-borrow.md"),
        "---\nruleId: PERF-01\nenforcedBy: review\n---\n# Borrow\nBody.\n",
    );
    for agent in ["code-inline-reviewer", "holistic-reviewer"] {
        fixture.write(
            &format!(".claude/agents/{agent}.md"),
            "# Reviewer\nCheck.\n",
        );
    }
    fixture.write("src/a.rs", "fn a() {}\nfn b() {}\n");
    commit_everything(fixture.path());
    fixture
}

fn commit_everything(directory: &Path) {
    for arguments in [
        vec!["add", "."],
        vec![
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "user.name=test",
            "commit",
            "-q",
            "-m",
            "initial",
        ],
    ] {
        let status = Command::new("git")
            .args(arguments)
            .current_dir(directory)
            .status()
            .unwrap();
        assert!(status.success());
    }
}

fn install_fake_gh(bin_directory: &Path, log: &Path, diff: &Path) -> PathBuf {
    std::fs::create_dir_all(bin_directory).unwrap();
    let program = bin_directory.join("gh");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n<<END>>\\n' \"$*\" >> '{log}'\ncase \"$*\" in\n*'select(.body'*) ;;\n*'--jq .id'*) printf 99;;\n*'.[].filename'*) printf 'src/a.rs';;\n*'.base.ref'*) printf main;;\n*'compare/'*) cat '{diff}';;\nesac\n",
        log = log.display(),
        diff = diff.display()
    );
    std::fs::write(&program, script).unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
    program
}

struct CiRun {
    output: Output,
    calls: Vec<String>,
}

fn prepare_tools() -> PathBuf {
    let tools = std::env::temp_dir().join(format!(
        "review-ci-{}-{}",
        std::process::id(),
        RUN_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&tools).unwrap();
    std::fs::write(tools.join("change.diff"), NEW_FILE_DIFF).unwrap();
    install_fake_gh(
        &tools.join("bin"),
        &tools.join("calls.log"),
        &tools.join("change.diff"),
    );
    tools
}

fn launch_review(fixture: &Fixture, tools: &Path, server: &FakeModelServer) -> Output {
    let path = format!(
        "{}:{}",
        tools.join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["review", "pull-request", "--pull-request", "7"])
        .args([
            "--repository",
            "owner/repository",
            "--event-name",
            "pull_request",
        ])
        .arg("--repository-root")
        .arg(fixture.path())
        .env("PATH", path)
        .env("DEEPSEEK_API_KEY", "key")
        .env("DEEPSEEK_BASE_URL", &server.url)
        .output()
        .unwrap()
}

fn run_ci(scripts: Vec<Script>) -> CiRun {
    let fixture = prepare_repository();
    let tools = ToolsDirectory(prepare_tools());
    let server = FakeModelServer::start(scripts);
    let output = launch_review(&fixture, &tools.0, &server);
    let calls = read_calls(&tools.0.join("calls.log"));
    CiRun { output, calls }
}

struct ToolsDirectory(PathBuf);

impl Drop for ToolsDirectory {
    fn drop(&mut self) {
        drop(std::fs::remove_dir_all(&self.0));
    }
}

fn read_calls(log: &Path) -> Vec<String> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .split("\n<<END>>\n")
        .filter(|call| !call.is_empty())
        .map(str::to_owned)
        .collect()
}

fn count_calls(run: &CiRun, fragment: &str) -> usize {
    run.calls
        .iter()
        .filter(|call| call.contains(fragment))
        .count()
}

fn answer(text: &str) -> Script {
    Script::Respond(format!("{}{}", content_event(text), finish_event("stop")))
}

#[test]
fn valid_findings_are_posted_and_the_status_succeeds() {
    let run = run_ci(vec![answer(FINDING_ANSWER), answer(FINDING_ANSWER)]);
    assert!(
        run.output.status.success(),
        "{}",
        String::from_utf8_lossy(&run.output.stderr)
    );
    assert_eq!(count_calls(&run, "pulls/7/comments"), 2);
    assert_eq!(count_calls(&run, "state=pending"), 1);
    assert_eq!(count_calls(&run, "state=success"), 1);
    assert_eq!(count_calls(&run, "state=failure"), 0);
}

#[test]
fn failed_model_call_posts_nothing_and_marks_the_status_failed() {
    let run = run_ci(vec![Script::Status(400), Script::Status(400)]);
    assert_eq!(run.output.status.code(), Some(1));
    assert_eq!(count_calls(&run, "pulls/7/comments"), 0);
    assert_eq!(count_calls(&run, "state=failure"), 1);
    assert_eq!(count_calls(&run, "state=success"), 0);
    assert!(run.calls.iter().any(
        |call| call.contains("did not complete") && call.contains("remain on the pull request")
    ));
}

#[test]
fn off_diff_findings_are_dropped_and_counted_not_posted() {
    let off_diff = r#"{"violations":[{"ruleId":"PERF-01","path":"src/a.rs","line":99,"body":"PERF-01 nope","confidence":"high"}]}"#;
    let run = run_ci(vec![answer(off_diff), answer(off_diff)]);
    assert!(run.output.status.success());
    assert_eq!(count_calls(&run, "pulls/7/comments"), 0);
    assert!(
        run.calls
            .iter()
            .any(|call| call.contains("1 dropped as invalid"))
    );
}
