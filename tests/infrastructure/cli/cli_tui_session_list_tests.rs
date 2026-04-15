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
