use ff::input::{process_file_content, process_stdin_content, read_direct_items, read_input};
use std::fs;
use tempfile::TempDir;

/// Helper function to create a temporary file with content
fn create_temp_file(content: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_items.txt");
    fs::write(&file_path, content).unwrap();
    (temp_dir, file_path.to_string_lossy().to_string())
}

#[tokio::test]
async fn test_file_input() {
    let (temp_dir, file_path) = create_temp_file("file_item1\nfile_item2\nfile_item3");

    let read_result = read_input(&file_path).await;
    assert!(read_result.is_ok());
    assert_eq!(
        read_result.unwrap(),
        vec!["file_item1", "file_item2", "file_item3"]
    );

    temp_dir.close().unwrap();
}

#[tokio::test]
async fn test_nonexistent_file() {
    let read_result = read_input("nonexistent_file.txt").await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), vec!["nonexistent_file.txt"]);
}

#[tokio::test]
async fn test_empty_file() {
    let (temp_dir, file_path) = create_temp_file("");

    let read_result = read_input(&file_path).await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), Vec::<String>::new());

    temp_dir.close().unwrap();
}

#[tokio::test]
async fn test_file_with_whitespace_only() {
    let (temp_dir, file_path) = create_temp_file("   \n  \n  \n");

    let read_result = read_input(&file_path).await;
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), vec!["   ", "  ", "  "]);

    temp_dir.close().unwrap();
}

#[test]
fn test_read_direct_items_integration() {
    let items = vec![
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    let result = read_direct_items(items.clone());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), items);
}

#[test]
fn test_process_stdin_content_integration() {
    let content = "line1\nline2\nline3";
    let result = process_stdin_content(content);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

#[test]
fn test_process_file_content_integration() {
    let content = "line1\nline2\nline3";
    let result = process_file_content(content);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["line1", "line2", "line3"]);
}

// === End-to-end tests of the compiled binary ===
//
// These run without a TTY: `--filter` must never require one.

use std::io::Write;
use std::process::{Command, Stdio};

fn ff(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ff"))
        .args(args)
        .output()
        .expect("failed to run ff binary")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn filter_mode_prints_matches_without_a_tty() {
    let output = ff(&["--filter", "app", "apple", "banana"]);

    assert!(output.status.success(), "exit status: {:?}", output.status);
    assert_eq!(stdout(&output), "apple\n");
}

#[test]
fn filter_mode_exits_one_when_nothing_matches() {
    let output = ff(&["--filter", "zzz", "apple", "banana"]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
}

#[test]
fn filter_mode_reads_piped_stdin() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ff"))
        .args(["--filter", "ban"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn ff");

    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(b"apple\nbanana\ncherry\n")
        .expect("write stdin");

    let output = child.wait_with_output().expect("wait for ff");
    assert!(output.status.success());
    assert_eq!(stdout(&output), "banana\n");
}

#[test]
fn filter_mode_prints_file_line_numbers() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("items.txt");
    fs::write(&path, "alpha\nbeta\ngamma\n").unwrap();

    let output = ff(&["--line-number", "--filter", "beta", &path.to_string_lossy()]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), format!("{}:2\n", path.to_string_lossy()));
}

#[test]
fn filter_mode_handles_more_than_1000_items() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("many.txt");
    let contents: String = (0..2000).map(|i| format!("item{i}\n")).collect();
    fs::write(&path, contents).unwrap();

    let output = ff(&["--filter", "item1999", &path.to_string_lossy()]);

    assert!(
        output.status.success(),
        "filter must not hang above 1000 items"
    );
    assert_eq!(stdout(&output), "item1999\n");
}

#[test]
fn flags_may_precede_the_input() {
    // Regression: `ff --height 10 file` used to fail with "Invalid input source".
    let output = ff(&["-m", "--filter", "app", "apple"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "apple\n");
}

#[test]
fn double_dash_allows_dash_prefixed_items() {
    let output = ff(&["--filter", "app", "--", "--apple"]);

    assert!(output.status.success());
    assert_eq!(stdout(&output), "--apple\n");
}

#[test]
fn unknown_flags_are_rejected() {
    let output = ff(&["--bogus", "apple"]);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown flag: --bogus"), "stderr: {stderr}");
}
