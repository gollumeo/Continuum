use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    let script_path = bin_dir.join("codex");
    let script = "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$CODEX_ARGS_LOG\"\nprintf '%s' \"$CODEX_STDOUT\"\nprintf '%s' \"$CODEX_STDERR\" 1>&2\nexit \"$CODEX_EXIT_CODE\"\n";

    fs::write(&script_path, script).expect("fake codex script should be written");

    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&script_path)
            .expect("fake codex script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions)
            .expect("fake codex script should be executable");
    }
}

fn install_fake_codex_script(bin_dir: &Path, script: &str) {
    let script_path = bin_dir.join("codex");

    fs::write(&script_path, script).expect("fake codex script should be written");

    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&script_path)
            .expect("fake codex script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions)
            .expect("fake codex script should be executable");
    }
}

fn install_fake_cargo_script(bin_dir: &Path, script: &str) {
    let script_path = bin_dir.join("cargo");

    fs::write(&script_path, script).expect("fake cargo script should be written");

    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&script_path)
            .expect("fake cargo script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions)
            .expect("fake cargo script should be executable");
    }
}

fn init_temp_git_repo(label: &str) -> PathBuf {
    let repo_dir = unique_temp_dir(label);
    let init_status = Command::new("git")
        .arg("init")
        .current_dir(&repo_dir)
        .output()
        .expect("git init should launch");

    assert!(init_status.status.success());

    repo_dir
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
fn shows_exact_increment_mission_admission_before_full_session_completion_and_keeps_proof_output_out_of_tui_pty(
) {
    let repo_root = init_increment_contract_repo("tui-increment-mission-admitted-repo");
    let temp_dir = unique_temp_dir("tui-increment-mission-admitted-logs");
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
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\nprintf 'INCREMENT PROOF STDOUT\n'\nprintf 'INCREMENT PROOF STDERR\n' 1>&2\nsleep 1\ntouch \"$CARGO_DONE_LOG\"\nexit 0\n",
    );

    let mut capture = spawn_tui_capture(
        "tui-increment-mission-admitted",
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
        .expect("increment prompt should be written to tui shell");
    capture
        .stdin
        .flush()
        .expect("increment prompt should flush to tui shell");
    thread::sleep(Duration::from_millis(200));
    capture
        .stdin
        .write_all(b"\r")
        .expect("increment prompt submit should be written to tui shell");
    capture
        .stdin
        .flush()
        .expect("increment prompt submit should flush to tui shell");

    let transcript = wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(3),
        |transcript| transcript.contains("State: Mission admitted") && !cargo_done_log.exists(),
    );

    assert!(transcript.contains("State: Submitting mission"));
    assert!(transcript.contains("State: Mission admitted"));
    assert!(transcript.contains("[~]"));
    assert!(!transcript.contains("INCREMENT PROOF STDOUT"));
    assert!(!transcript.contains("INCREMENT PROOF STDERR"));

    wait_for_transcript_condition(&capture.transcript_path, Duration::from_secs(3), |_| {
        cargo_done_log.exists()
    });

    capture
        .stdin
        .write_all(&[0x1b])
        .expect("escape should be written to tui shell");
    capture
        .stdin
        .flush()
        .expect("escape should flush to tui shell");

    let status = capture
        .child
        .wait()
        .expect("tui shell should terminate cleanly");
    let final_transcript = read_transcript(&capture.transcript_path);

    let _ = fs::remove_file(&capture.transcript_path);

    assert!(status.success());
    assert!(!final_transcript.contains("INCREMENT PROOF STDOUT"));
    assert!(!final_transcript.contains("INCREMENT PROOF STDERR"));

    let codex_args = fs::read_to_string(&codex_args_log).expect("codex args should be logged");
    let cargo_args = fs::read_to_string(&cargo_args_log).expect("cargo args should be logged");

    assert!(codex_args.contains("exec"));
    assert!(codex_args.contains("Allowed file scope: src/lib.rs"));
    assert!(cargo_args
        .contains("test --test increment_contract increment_adds_one_to_input -- --exact"));
}

#[test]
fn shows_pre_build_refusal_feedback_for_underspecified_mission_without_builder_side_effects() {
    let repo_root = init_temp_git_repo("tui-mission-refused-repo");
    let temp_dir = unique_temp_dir("tui-mission-refused-logs");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    fs::write(repo_root.join("README.md"), "# Continuum\n").expect("README.md should be written");
    install_fake_codex(&bin_dir);

    let (status, transcript) = capture_tui_transcript(
        "tui-mission-refused",
        &repo_root,
        vec![
            (
                Duration::from_millis(200),
                b"Generate the README.md for this repository.".to_vec(),
            ),
            (Duration::from_millis(200), b"\r".to_vec()),
            (Duration::from_millis(500), vec![0x1b]),
        ],
        &bin_dir,
        &args_log,
        "0",
    );

    assert!(status.success());
    assert!(transcript.contains("State: Submitting mission"));
    assert!(transcript.contains("State: Mission refused"));
    assert!(transcript.contains("Refusal: add an explicit allowed file scope."));
    assert!(transcript.contains("[!]"));
    assert!(!transcript.contains("Session: initializing."));
    assert!(!args_log.exists());
}

#[test]
fn shows_command_mode_and_unsupported_slash_feedback_while_keeping_prompt_reusable() {
    let repo_root = init_temp_git_repo("tui-command-mode-repo");
    let temp_dir = unique_temp_dir("tui-command-mode-logs");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let (status, transcript) = capture_tui_transcript(
        "tui-command-mode",
        &repo_root,
        vec![
            (Duration::from_millis(200), b"/help".to_vec()),
            (Duration::from_millis(200), b"\r".to_vec()),
            (Duration::from_millis(200), b"te".to_vec()),
            (Duration::from_millis(200), vec![0x1b]),
        ],
        &bin_dir,
        &args_log,
        "0",
    );

    assert!(status.success());
    assert!(transcript.contains("State: Command mode"));
    assert!(transcript.contains("Command unsupported in Story 1.2."));
    assert!(transcript.contains("No sessions yet."));
    assert!(transcript.contains("> te"));
    assert!(!args_log.exists());
}
