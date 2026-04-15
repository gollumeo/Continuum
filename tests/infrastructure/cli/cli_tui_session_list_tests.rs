use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unique_temp_path(label: &str, suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("continuum-{label}-{nanos}.{suffix}"))
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("continuum-{label}-{nanos}"));

    fs::create_dir_all(&dir).expect("temp dir should be created");

    dir
}

fn install_fake_codex(bin_dir: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let script_path = bin_dir.join("codex");
    let script = "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$CODEX_ARGS_LOG\"\nprintf '%s' \"$CODEX_STDOUT\"\nprintf '%s' \"$CODEX_STDERR\" 1>&2\nexit \"$CODEX_EXIT_CODE\"\n";

    fs::write(&script_path, script).expect("fake codex script should be written");

    let mut permissions = fs::metadata(&script_path)
        .expect("fake codex script metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).expect("fake codex script should be executable");
}

fn init_temp_git_repo(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("continuum-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("temp dir should be created");

    let init_status = Command::new("git")
        .arg("init")
        .current_dir(&dir)
        .output()
        .expect("git init should launch");

    assert!(init_status.status.success());

    dir
}

fn prefixed_path(bin_dir: &Path) -> String {
    let existing_path = std::env::var("PATH").unwrap_or_default();

    if existing_path.is_empty() {
        bin_dir.display().to_string()
    } else {
        format!("{}:{existing_path}", bin_dir.display())
    }
}

fn capture_tui_transcript(
    label: &str,
    repo_root: &Path,
    input_steps: Vec<(Duration, Vec<u8>)>,
    bin_dir: &Path,
    args_log: &Path,
    codex_exit_code: &str,
) -> (ExitStatus, String) {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let transcript_path = unique_temp_path(label, "typescript");
    let mut command = Command::new("script");

    command
        .current_dir(repo_root)
        .arg("-q")
        .arg("-e")
        .arg("-f")
        .arg("-c")
        .arg(&binary_path)
        .arg(&transcript_path)
        .env("PATH", prefixed_path(bin_dir))
        .env("CODEX_ARGS_LOG", args_log)
        .env("CODEX_STDOUT", "builder stdout")
        .env("CODEX_STDERR", "builder stderr")
        .env("CODEX_EXIT_CODE", codex_exit_code)
        .env("COLUMNS", "80")
        .env("LINES", "24")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = command.spawn().expect("tui shell should launch in a pty");
    let mut stdin = child.stdin.take().expect("script stdin should be piped");

    let writer = thread::spawn(move || {
        for (delay, bytes) in input_steps {
            thread::sleep(delay);
            stdin
                .write_all(&bytes)
                .expect("input should be written to tui shell");
            stdin.flush().expect("tui shell input should flush");
        }
    });

    let status = child.wait().expect("tui shell should terminate cleanly");
    writer.join().expect("tui shell input writer should finish");

    let transcript = String::from_utf8_lossy(
        &fs::read(&transcript_path).expect("transcript should be readable"),
    )
    .into_owned();

    let _ = fs::remove_file(&transcript_path);

    (status, transcript)
}

#[test]
fn shows_empty_state_in_sessions_area_before_any_session_submitted() {
    let repo_root = init_temp_git_repo("tui-session-list-empty-repo");
    let temp_dir = unique_temp_dir("tui-session-list-empty-logs");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let (status, transcript) = capture_tui_transcript(
        "tui-session-list-empty",
        &repo_root,
        vec![
            (Duration::from_millis(200), b"te".to_vec()),
            (Duration::from_millis(200), vec![0x1b]),
        ],
        &bin_dir,
        &args_log,
        "0",
    );

    assert!(status.success());
    assert!(transcript.contains("Sessions"));
    assert!(transcript.contains("No sessions yet."));
    assert!(!transcript.contains("No session running."));
}

use std::process::{Child, ChildStdin};

fn install_fake_codex_script(bin_dir: &Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;

    let script_path = bin_dir.join("codex");
    fs::write(&script_path, script).expect("fake codex script should be written");

    let mut permissions = fs::metadata(&script_path)
        .expect("fake codex script metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).expect("fake codex script should be executable");
}

fn install_fake_cargo_script(bin_dir: &Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;

    let script_path = bin_dir.join("cargo");
    fs::write(&script_path, script).expect("fake cargo script should be written");

    let mut permissions = fs::metadata(&script_path)
        .expect("fake cargo script metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).expect("fake cargo script should be executable");
}

fn init_increment_contract_repo(label: &str) -> PathBuf {
    let repo_dir = init_temp_git_repo(label);

    fs::write(
        repo_dir.join("Cargo.toml"),
        "[package]\nname = \"increment-contract\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .expect("Cargo.toml should be written");
    fs::create_dir_all(repo_dir.join("src")).expect("src dir should be created");
    fs::create_dir_all(repo_dir.join("tests")).expect("tests dir should be created");
    fs::write(
        repo_dir.join("src/lib.rs"),
        "pub fn increment(input: i32) -> i32 {\n    input\n}\n",
    )
    .expect("src/lib.rs should be written");
    fs::write(
        repo_dir.join("tests/increment_contract.rs"),
        "use increment_contract::increment;\n\n#[test]\nfn increment_adds_one_to_input() {\n    assert_eq!(increment(1), 2);\n}\n",
    )
    .expect("increment contract test should be written");

    repo_dir
}

struct RunningTuiCapture {
    child: Child,
    stdin: ChildStdin,
    transcript_path: PathBuf,
}

fn spawn_tui_capture(
    label: &str,
    repo_root: &Path,
    bin_dir: &Path,
    args_log: &Path,
    codex_exit_code: &str,
    cargo_args_log: &Path,
    cargo_done_log: &Path,
) -> RunningTuiCapture {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let transcript_path = unique_temp_path(label, "typescript");
    let mut command = Command::new("script");

    command
        .current_dir(repo_root)
        .arg("-q")
        .arg("-e")
        .arg("-f")
        .arg("-c")
        .arg(&binary_path)
        .arg(&transcript_path)
        .env("PATH", prefixed_path(bin_dir))
        .env("CODEX_ARGS_LOG", args_log)
        .env("CODEX_STDOUT", "builder stdout")
        .env("CODEX_STDERR", "builder stderr")
        .env("CODEX_EXIT_CODE", codex_exit_code)
        .env("CARGO_ARGS_LOG", cargo_args_log)
        .env("CARGO_DONE_LOG", cargo_done_log)
        .env("COLUMNS", "80")
        .env("LINES", "24")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = command.spawn().expect("tui shell should launch in a pty");
    let stdin = child.stdin.take().expect("script stdin should be piped");

    RunningTuiCapture {
        child,
        stdin,
        transcript_path,
    }
}

fn read_transcript(transcript_path: &Path) -> String {
    match fs::read(transcript_path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => String::new(),
    }
}

fn wait_for_transcript_condition<F>(
    transcript_path: &Path,
    timeout: Duration,
    mut condition: F,
) -> String
where
    F: FnMut(&str) -> bool,
{
    use std::time::Instant;

    let deadline = Instant::now() + timeout;

    loop {
        let transcript = read_transcript(transcript_path);

        if condition(&transcript) {
            return transcript;
        }

        if Instant::now() >= deadline {
            panic!("timed out waiting for transcript condition");
        }

        thread::sleep(Duration::from_millis(25));
    }
}

#[test]
fn shows_active_session_row_during_runtime_execution_before_cargo_completes() {
    let repo_root = init_increment_contract_repo("tui-session-list-active-repo");
    let temp_dir = unique_temp_dir("tui-session-list-active-logs");
    let bin_dir = temp_dir.join("bin");
    let codex_args_log = temp_dir.join("codex-args.log");
    let cargo_args_log = temp_dir.join("cargo-args.log");
    let cargo_done_log = temp_dir.join("cargo-done.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$CODEX_ARGS_LOG\"\nprintf 'pub fn increment(input: i32) -> i32 {\n    input + 1\n}\n' > src/lib.rs\nexit \"$CODEX_EXIT_CODE\"\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\nsleep 1\ntouch \"$CARGO_DONE_LOG\"\nexit 0\n",
    );

    let mut capture = spawn_tui_capture(
        "tui-session-list-active",
        &repo_root,
        &bin_dir,
        &codex_args_log,
        "0",
        &cargo_args_log,
        &cargo_done_log,
    );

    thread::sleep(Duration::from_millis(200));
    capture
        .stdin
        .write_all(b"Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.")
        .expect("mission prompt should be written");
    capture.stdin.flush().expect("mission prompt should flush");
    thread::sleep(Duration::from_millis(200));
    capture
        .stdin
        .write_all(b"\r")
        .expect("mission submit should be written");
    capture.stdin.flush().expect("mission submit should flush");

    let transcript = wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(5),
        |transcript| transcript.contains("[~]") && !cargo_done_log.exists(),
    );

    assert!(transcript.contains("[~]"));

    wait_for_transcript_condition(&capture.transcript_path, Duration::from_secs(5), |_| {
        cargo_done_log.exists()
    });

    capture
        .stdin
        .write_all(&[0x1b])
        .expect("escape should be written");
    capture.stdin.flush().expect("escape should flush");

    let _ = capture.child.wait().expect("tui shell should terminate");
    let _ = fs::remove_file(&capture.transcript_path);
}

#[test]
fn shows_completed_session_row_after_successful_mission_execution() {
    let repo_root = init_increment_contract_repo("tui-session-list-completed-repo");
    let temp_dir = unique_temp_dir("tui-session-list-completed-logs");
    let bin_dir = temp_dir.join("bin");
    let codex_args_log = temp_dir.join("codex-args.log");
    let cargo_args_log = temp_dir.join("cargo-args.log");
    let cargo_done_log = temp_dir.join("cargo-done.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$CODEX_ARGS_LOG\"\nprintf 'pub fn increment(input: i32) -> i32 {\n    input + 1\n}\n' > src/lib.rs\nexit \"$CODEX_EXIT_CODE\"\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\ntouch \"$CARGO_DONE_LOG\"\nexit 0\n",
    );

    let mut capture = spawn_tui_capture(
        "tui-session-list-completed",
        &repo_root,
        &bin_dir,
        &codex_args_log,
        "0",
        &cargo_args_log,
        &cargo_done_log,
    );

    thread::sleep(Duration::from_millis(200));
    capture
        .stdin
        .write_all(b"Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.")
        .expect("mission prompt should be written");
    capture.stdin.flush().expect("mission prompt should flush");
    thread::sleep(Duration::from_millis(200));
    capture
        .stdin
        .write_all(b"\r")
        .expect("mission submit should be written");
    capture.stdin.flush().expect("mission submit should flush");

    wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(10),
        |transcript| transcript.contains("[+]"),
    );

    capture
        .stdin
        .write_all(&[0x1b])
        .expect("escape should be written");
    capture.stdin.flush().expect("escape should flush");

    let status = capture.child.wait().expect("tui shell should terminate");
    let final_transcript = read_transcript(&capture.transcript_path);
    let _ = fs::remove_file(&capture.transcript_path);

    assert!(status.success());
    assert!(final_transcript.contains("[+]"));
}
