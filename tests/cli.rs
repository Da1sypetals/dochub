use assert_cmd::Command;
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

const ROOT: &str = "/tmp/.dochub";
const HOME_DIR: &str = "/tmp";

struct TestSandbox;

impl TestSandbox {
    fn new() -> Self {
        cleanup_root();
        fs::create_dir_all(ROOT).unwrap();
        Self
    }
}

impl Drop for TestSandbox {
    fn drop(&mut self) {
        cleanup_root();
    }
}

fn cleanup_root() {
    match fs::remove_dir_all(ROOT) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => panic!("failed to clean {ROOT}: {err}"),
    }
}

fn cmd() -> Command {
    let mut command = Command::cargo_bin("dochub").unwrap();
    command.env("HOME", HOME_DIR);
    command
}

fn run(args: &[&str]) -> Output {
    cmd().args(args).output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn mkdir(path: impl AsRef<Path>) {
    fs::create_dir_all(path).unwrap();
}

fn write_file(path: impl AsRef<Path>, contents: &[u8]) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn write_large_file(path: impl AsRef<Path>, size: u64) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let file = fs::File::create(path).unwrap();
    file.set_len(size).unwrap();
}

fn config_path() -> PathBuf {
    PathBuf::from(ROOT).join("hub.toml")
}

fn read_config() -> String {
    fs::read_to_string(config_path()).unwrap()
}

fn canonical_string(path: impl AsRef<Path>) -> String {
    fs::canonicalize(path).unwrap().display().to_string()
}

#[test]
#[serial]
fn add_succeeds_and_persists() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/sources/a");

    let output = run(&["add", "myhub", "/tmp/.dochub/sources/a"]);
    assert_success(&output);

    let config = read_config();
    assert!(config.contains("myhub"));
    assert!(config.contains(&canonical_string("/tmp/.dochub/sources/a")));
}

#[test]
#[serial]
fn add_rejects_duplicate_name() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/sources/a");

    assert_success(&run(&["add", "dup", "/tmp/.dochub/sources/a"]));
    let output = run(&["add", "dup", "/tmp/.dochub/sources/a"]);
    assert_failure(&output);
    assert!(stderr(&output).contains("already exists"));
}

#[test]
#[serial]
fn add_rejects_non_directory() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/notadir", b"file");

    let output = run(&["add", "bad", "/tmp/.dochub/notadir"]);
    assert_failure(&output);
    assert!(stderr(&output).contains("not a directory"));
}

#[test]
#[serial]
fn ls_lists_all_and_single_and_missing() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/s1/a.txt", b"abc");
    mkdir("/tmp/.dochub/s2");
    assert_success(&run(&["add", "a", "/tmp/.dochub/s1"]));
    assert_success(&run(&["add", "b", "/tmp/.dochub/s2"]));

    let all_output = run(&["ls"]);
    assert_success(&all_output);
    let all_stdout = stdout(&all_output);
    let all_lines: Vec<_> = all_stdout.lines().collect();
    assert_eq!(all_lines.len(), 4);
    assert!(all_lines[0].contains("NAME"));
    assert!(all_lines[0].contains("PATH"));
    assert!(all_lines[0].contains("SIZE"));
    assert!(all_lines[1].contains("---"));
    assert!(all_lines[2].contains(&canonical_string("/tmp/.dochub/s1")));
    assert!(all_lines[2].ends_with("3 B"));
    assert!(all_lines[3].contains(&canonical_string("/tmp/.dochub/s2")));
    assert!(all_lines[3].ends_with("0 B"));
    assert_eq!(all_lines[2].find("  "), all_lines[3].find("  "));

    let single_output = run(&["ls", "a"]);
    assert_success(&single_output);
    let single_stdout = stdout(&single_output);
    let single_lines: Vec<_> = single_stdout.lines().collect();
    assert_eq!(single_lines.len(), 3);
    assert!(single_lines[0].contains("NAME"));
    assert!(single_lines[0].contains("PATH"));
    assert!(single_lines[0].contains("SIZE"));
    assert!(single_lines[2].contains(&canonical_string("/tmp/.dochub/s1")));
    assert!(single_lines[2].ends_with("3 B"));

    let missing_output = run(&["list", "missing"]);
    assert_failure(&missing_output);
    assert!(stderr(&missing_output).contains("not found"));
}

#[test]
#[serial]
fn ls_shows_missing_when_path_no_longer_exists() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/gone");
    assert_success(&run(&["add", "gone", "/tmp/.dochub/gone"]));
    fs::remove_dir_all("/tmp/.dochub/gone").unwrap();

    let output = run(&["ls", "gone"]);
    assert_success(&output);
    let output_text = stdout(&output);
    let lines: Vec<_> = output_text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[2].ends_with("missing"));
}

#[test]
#[serial]
fn ls_formats_large_sizes_human_readably() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/large");
    write_large_file("/tmp/.dochub/large/blob.bin", 3 * 1024 * 1024 + 512 * 1024);
    assert_success(&run(&["add", "large", "/tmp/.dochub/large"]));

    let output = run(&["ls", "large"]);
    assert_success(&output);
    let output_text = stdout(&output);
    let lines: Vec<_> = output_text.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[2].ends_with("3.5 MB"));
}

#[test]
#[serial]
fn prune_removes_stale_and_reports() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/gone");
    mkdir("/tmp/.dochub/keep");
    let stale_path = canonical_string("/tmp/.dochub/gone");
    assert_success(&run(&["add", "stale", "/tmp/.dochub/gone"]));
    assert_success(&run(&["add", "keep", "/tmp/.dochub/keep"]));
    fs::remove_dir_all("/tmp/.dochub/gone").unwrap();

    let output = run(&["prune"]);
    assert_success(&output);
    assert!(stdout(&output).contains(&format!("stale\t{stale_path}")));

    let config = read_config();
    assert!(!config.contains("stale"));
    assert!(config.contains("keep"));
}

#[test]
#[serial]
fn sanity_default_limit_and_custom_sane_size() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/big");
    write_large_file("/tmp/.dochub/big/huge.bin", 16 * 1024 * 1024 + 1);
    assert_success(&run(&["add", "big", "/tmp/.dochub/big"]));

    let default_output = run(&["sanity"]);
    assert_success(&default_output);
    assert!(stdout(&default_output).contains("big"));
    assert!(stdout(&default_output).contains("limit=16MB"));

    write_file(
        config_path(),
        br#"sane-size = 1
[hub]
small = "/tmp/.dochub/small"
"#,
    );
    mkdir("/tmp/.dochub/small");
    write_large_file("/tmp/.dochub/small/too-big.bin", 1024 * 1024 + 1);

    let custom_output = run(&["sanity"]);
    assert_success(&custom_output);
    assert!(stdout(&custom_output).contains("small"));
    assert!(stdout(&custom_output).contains("limit=1MB"));
}

#[test]
#[serial]
fn cp_creates_dest_name_content_layout_and_reports_destination() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/hub1/a.txt", b"a");
    write_file("/tmp/.dochub/src/hub1/b/c.txt", b"c");
    assert_success(&run(&["add", "hub1", "/tmp/.dochub/src/hub1"]));

    let output = run(&["cp", "hub1", "/tmp/.dochub/out"]);
    assert_success(&output);
    assert_eq!(
        fs::read("/tmp/.dochub/out/hub1/content/a.txt").unwrap(),
        b"a"
    );
    assert_eq!(
        fs::read("/tmp/.dochub/out/hub1/content/b/c.txt").unwrap(),
        b"c"
    );
    assert!(stdout(&output).contains("/tmp/.dochub/out/hub1/content"));
}

#[test]
#[serial]
fn cp_dest_trailing_slash_equivalent() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/hub1/file.txt", b"hello");
    assert_success(&run(&["add", "hub1", "/tmp/.dochub/src/hub1"]));

    let output_one = run(&["cp", "hub1", "/tmp/.dochub/out2"]);
    assert_success(&output_one);

    fs::remove_dir_all("/tmp/.dochub/out2").unwrap();

    let output_two = run(&["cp", "hub1", "/tmp/.dochub/out2/"]);
    assert_success(&output_two);
    assert_eq!(stdout(&output_one), stdout(&output_two));
    assert_eq!(
        fs::read("/tmp/.dochub/out2/hub1/content/file.txt").unwrap(),
        b"hello"
    );
}

#[test]
#[serial]
fn cp_skips_dot_git_and_respects_hub_ignore() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/hub1/.git/HEAD", b"ref: refs/heads/main\n");
    write_file("/tmp/.dochub/src/hub1/subdir/.git/HEAD", b"nested");
    write_file("/tmp/.dochub/src/hub1/keep.txt", b"keep");
    write_file("/tmp/.dochub/src/hub1/skipme/x.txt", b"skip");
    write_file("/tmp/.dochub/src/hub1/foo.tmp", b"tmp");
    write_file(
        config_path(),
        br#"ignore = ["skipme", "*.tmp"]
[hub]
hub1 = "/tmp/.dochub/src/hub1"
"#,
    );

    let output = run(&["cp", "hub1", "/tmp/.dochub/out"]);
    assert_success(&output);
    assert!(Path::new("/tmp/.dochub/out/hub1/content/keep.txt").exists());
    assert!(!Path::new("/tmp/.dochub/out/hub1/content/.git").exists());
    assert!(!Path::new("/tmp/.dochub/out/hub1/content/subdir/.git").exists());
    assert!(!Path::new("/tmp/.dochub/out/hub1/content/skipme").exists());
    assert!(!Path::new("/tmp/.dochub/out/hub1/content/foo.tmp").exists());
}

#[test]
#[serial]
fn cp_non_tty_shows_closest_hint_when_typo_is_near_threshold() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/hub1/a.txt", b"a");
    assert_success(&run(&["add", "hub1", "/tmp/.dochub/src/hub1"]));

    let output = run(&["cp", "hub1x", "/tmp/.dochub/out"]);
    assert_failure(&output);
    let err = stderr(&output);
    assert!(
        err.contains("Closest hub name: `hub1`"),
        "expected fuzzy hint on stderr, got:\n{err}"
    );
    assert!(
        err.contains("Hub entry `hub1x` not found."),
        "expected not-found error on stderr, got:\n{err}"
    );
}

#[test]
#[serial]
fn skill_cp_non_tty_shows_closest_hint_when_typo_is_near_threshold() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/sk/x.txt", b"x");
    write_file(
        config_path(),
        br#"skill-dir = [".claude/skill/", ".cursor/skill"]
[hub]
sk = "/tmp/.dochub/src/sk"
"#,
    );

    let output = run(&["skill", "cp", "skx", "/tmp/.dochub/project"]);
    assert_failure(&output);
    let err = stderr(&output);
    assert!(
        err.contains("Closest hub name: `sk`"),
        "expected fuzzy hint on stderr, got:\n{err}"
    );
    assert!(
        err.contains("Hub entry `skx` not found."),
        "expected not-found error on stderr, got:\n{err}"
    );
}

#[test]
#[serial]
fn skill_cp_applies_each_skill_dir_and_reports_all_destinations() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/sk/x.txt", b"x");
    write_file(
        config_path(),
        br#"skill-dir = [".claude/skill/", ".cursor/skill"]
[hub]
sk = "/tmp/.dochub/src/sk"
"#,
    );

    let output = run(&["skill", "cp", "sk", "/tmp/.dochub/project"]);
    assert_success(&output);

    assert!(Path::new("/tmp/.dochub/project/.claude/skill/sk/content/x.txt").exists());
    assert!(Path::new("/tmp/.dochub/project/.cursor/skill/sk/content/x.txt").exists());
    let out = stdout(&output);
    assert!(out.contains("/tmp/.dochub/project/.claude/skill/sk/content"));
    assert!(out.contains("/tmp/.dochub/project/.cursor/skill/sk/content"));

    mkdir("/tmp/.dochub/cwd");
    let mut default_dest = cmd();
    let output = default_dest
        .current_dir("/tmp/.dochub/cwd")
        .args(["skill", "cp", "sk"])
        .output()
        .unwrap();
    assert_success(&output);
    assert!(Path::new("/tmp/.dochub/cwd/.claude/skill/sk/content/x.txt").exists());
    assert!(Path::new("/tmp/.dochub/cwd/.cursor/skill/sk/content/x.txt").exists());
}

#[test]
#[serial]
fn skill_cp_skill_dir_trailing_slash_equivalent() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/sk/x.txt", b"x");
    write_file(
        config_path(),
        br#"skill-dir = [".claude/skill/"]
[hub]
sk = "/tmp/.dochub/src/sk"
"#,
    );

    let first = run(&["skill", "cp", "sk", "/tmp/.dochub/project-a"]);
    assert_success(&first);

    write_file(
        config_path(),
        br#"skill-dir = [".claude/skill"]
[hub]
sk = "/tmp/.dochub/src/sk"
"#,
    );

    let second = run(&["skill", "cp", "sk", "/tmp/.dochub/project-b"]);
    assert_success(&second);

    assert!(Path::new("/tmp/.dochub/project-a/.claude/skill/sk/content/x.txt").exists());
    assert!(Path::new("/tmp/.dochub/project-b/.claude/skill/sk/content/x.txt").exists());
}

#[test]
#[serial]
fn skill_cp_errors_when_skill_dir_missing_or_empty() {
    let _sandbox = TestSandbox::new();
    write_file(
        config_path(),
        br#"[hub]
sk = "/tmp/.dochub/src/sk"
"#,
    );
    mkdir("/tmp/.dochub/src/sk");

    let missing_output = run(&["skill", "cp", "sk", "."]);
    assert_failure(&missing_output);
    assert!(stderr(&missing_output).contains("skill-dir"));

    write_file(
        config_path(),
        br#"skill-dir = []
[hub]
sk = "/tmp/.dochub/src/sk"
"#,
    );

    let empty_output = run(&["skill", "cp", "sk", "."]);
    assert_failure(&empty_output);
    assert!(stderr(&empty_output).contains("skill-dir"));
}

#[test]
#[serial]
fn rm_prompt_yes_removes_entry_no_delete_files() {
    let _sandbox = TestSandbox::new();
    write_file("/tmp/.dochub/src/rmtest/file.txt", b"keep");
    assert_success(&run(&["add", "rmtest", "/tmp/.dochub/src/rmtest"]));

    let mut command = cmd();
    let output = command
        .args(["rm", "rmtest"])
        .write_stdin("y\n")
        .output()
        .unwrap();
    assert_success(&output);

    assert!(!read_config().contains("rmtest"));
    assert!(Path::new("/tmp/.dochub/src/rmtest/file.txt").exists());
}

#[test]
#[serial]
fn rm_prompt_no_leaves_entry() {
    let _sandbox = TestSandbox::new();
    mkdir("/tmp/.dochub/src/rmtest");
    assert_success(&run(&["add", "rmtest", "/tmp/.dochub/src/rmtest"]));

    let mut command = cmd();
    let output = command
        .args(["rm", "rmtest"])
        .write_stdin("n\n")
        .output()
        .unwrap();
    assert_success(&output);

    assert!(read_config().contains("rmtest"));
}
