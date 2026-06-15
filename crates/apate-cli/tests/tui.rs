use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("apate-tui-test-{nanos}-{nonce}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn apate() -> Command {
    Command::new(env!("CARGO_BIN_EXE_apate"))
}

#[test]
fn tui_subcommand_shows_menu() {
    let mut child = apate()
        .arg("tui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "0").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apate TUI 模式"));
    assert!(stdout.contains("输入数字后回车"));
}

#[test]
fn no_arguments_enters_tui_for_double_click_usage() {
    let mut child = apate()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "0").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("apate TUI 模式"));
    assert!(stdout.contains("输入数字后回车"));
}

#[test]
fn tui_can_inspect_a_file_from_stdin_menu() {
    let dir = TestDir::new();
    let file = dir.path().join("plain.bin");
    fs::write(&file, b"plain").unwrap();

    let mut child = apate()
        .arg("tui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "1").unwrap();
        writeln!(stdin, "{}", file.display()).unwrap();
        writeln!(stdin, "0").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("未识别为 apate 伪装文件"));
}

#[test]
fn tui_returns_to_menu_after_action_until_user_exits() {
    let dir = TestDir::new();
    let file = dir.path().join("plain.bin");
    fs::write(&file, b"plain").unwrap();

    let mut child = apate()
        .arg("tui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "1").unwrap();
        writeln!(stdin, "{}", file.display()).unwrap();
        writeln!(stdin, "0").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("apate TUI 模式").count() >= 2);
}

#[test]
fn tui_disguise_renames_like_cli() {
    let dir = TestDir::new();
    let source = dir.path().join("secret.zip");
    let disguised = dir.path().join("secret.jpg");
    fs::write(&source, b"plain payload").unwrap();

    let mut child = apate()
        .arg("tui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "3").unwrap();
        writeln!(stdin, "{}", source.display()).unwrap();
        writeln!(stdin, "jpg").unwrap();
        writeln!(stdin, "0").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("伪装完成"));
    assert!(!source.exists());
    assert!(disguised.exists());
}

#[test]
fn tui_can_exit_cleanly() {
    let mut child = apate()
        .arg("tui")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "0").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
