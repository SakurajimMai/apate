use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

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
        let path = std::env::temp_dir().join(format!("apate-cli-test-{nanos}-{nonce}"));
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

fn output_json(mut command: Command) -> Value {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "status: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn failure_json(mut command: Command) -> Value {
    let output = command.output().unwrap();
    assert!(
        !output.status.success(),
        "status: {:?}\nstdout: {}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn masks_json_lists_builtin_masks_for_agents() {
    let mut command = apate();
    command.args(["masks", "--json"]);
    let json = output_json(command);

    let masks = json["masks"].as_array().unwrap();
    assert!(masks.iter().any(|mask| mask["kind"] == "mp4"));
    assert!(masks.iter().any(|mask| mask["extension"] == ".exe"));
}

#[test]
fn dry_run_disguise_reports_actions_without_modifying_file() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin");
    let original = b"abcdef";
    fs::write(&file, original).unwrap();

    let mut command = apate();
    command.args([
        "disguise",
        "--input",
        file.to_str().unwrap(),
        "--kind",
        "jpg",
        "--dry-run",
        "--json",
    ]);
    let json = output_json(command);

    assert_eq!(json["dry_run"], true);
    assert_eq!(json["results"][0]["action"], "disguise");
    assert_eq!(fs::read(&file).unwrap(), original);
}

#[test]
fn dry_run_disguise_validates_custom_mask_file() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin");
    let mask = dir.path().join("empty.mask");
    let original = b"abcdef";
    fs::write(&file, original).unwrap();
    fs::write(&mask, []).unwrap();

    let mut command = apate();
    command.args([
        "disguise",
        "--input",
        file.to_str().unwrap(),
        "--mask-file",
        mask.to_str().unwrap(),
        "--dry-run",
        "--json",
    ]);
    let json = failure_json(command);

    assert_eq!(json["ok"], false);
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["results"][0]["code"], "empty_mask");
    assert_eq!(fs::read(&file).unwrap(), original);
}

#[test]
fn disguise_refuses_to_overwrite_default_rename_target_before_writing_source() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin");
    let target = dir.path().join("payload.bin.jpg");
    let original = b"abcdef";
    let existing = b"existing target";
    fs::write(&file, original).unwrap();
    fs::write(&target, existing).unwrap();

    let mut command = apate();
    command.args([
        "disguise",
        "--input",
        file.to_str().unwrap(),
        "--kind",
        "jpg",
        "--json",
    ]);
    let json = failure_json(command);

    assert_eq!(json["ok"], false);
    assert_eq!(json["results"][0]["code"], "output_exists");
    assert_eq!(json["results"][0]["output_path"], display_path(&target));
    assert_eq!(fs::read(&file).unwrap(), original);
    assert_eq!(fs::read(&target).unwrap(), existing);
}

#[test]
fn reveal_refuses_to_overwrite_default_rename_target_before_writing_source() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin.jpg");
    let target = dir.path().join("payload.bin");
    let original = b"abcdef0123456789";
    let existing = b"existing target";
    fs::write(&file, original).unwrap();

    let mut disguise = apate();
    disguise.args([
        "disguise",
        "--input",
        file.to_str().unwrap(),
        "--kind",
        "jpg",
        "--no-rename",
        "--json",
    ]);
    let disguised = output_json(disguise);
    assert_eq!(disguised["ok"], true);
    let disguised_source = fs::read(&file).unwrap();

    fs::write(&target, existing).unwrap();

    let mut reveal = apate();
    reveal.args(["reveal", "--input", file.to_str().unwrap(), "--json"]);
    let json = failure_json(reveal);

    assert_eq!(json["ok"], false);
    assert_eq!(json["results"][0]["code"], "output_exists");
    assert_eq!(json["results"][0]["output_path"], display_path(&target));
    assert_eq!(fs::read(&file).unwrap(), disguised_source);
    assert_eq!(fs::read(&target).unwrap(), existing);
}

#[test]
fn disguise_then_inspect_then_reveal_restores_original_bytes() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin");
    let original = b"abcdef0123456789";
    fs::write(&file, original).unwrap();

    let mut disguise = apate();
    disguise.args([
        "disguise",
        "--input",
        file.to_str().unwrap(),
        "--kind",
        "mp4",
        "--no-rename",
        "--json",
    ]);
    let disguised = output_json(disguise);
    assert_eq!(disguised["ok"], true);

    let mut inspect = apate();
    inspect.args(["inspect", file.to_str().unwrap(), "--json"]);
    let inspected = output_json(inspect);
    assert_eq!(inspected["disguised"], true);

    let mut reveal = apate();
    reveal.args([
        "reveal",
        "--input",
        file.to_str().unwrap(),
        "--no-rename",
        "--json",
    ]);
    let revealed = output_json(reveal);
    assert_eq!(revealed["ok"], true);
    assert_eq!(fs::read(&file).unwrap(), original);
}

fn display_path(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}
