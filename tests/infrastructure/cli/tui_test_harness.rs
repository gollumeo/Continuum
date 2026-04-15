use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const INCREMENT_FIX_PROMPT: &str =
    "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.";

pub fn unique_temp_path(label: &str, suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("continuum-{label}-{nanos}.{suffix}"))
}

pub fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("continuum-{label}-{nanos}"));

    fs::create_dir_all(&dir).expect("temp dir should be created");

    dir
}

fn set_executable(path: &Path) {
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path)
            .expect("script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("script should be executable");
    }
}

pub fn install_fake_codex(bin_dir: &Path) {
    let script_path = bin_dir.join("codex");
    let script = "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$CODEX_ARGS_LOG\"\nprintf '%s' \"$CODEX_STDOUT\"\nprintf '%s' \"$CODEX_STDERR\" 1>&2\nexit \"$CODEX_EXIT_CODE\"\n";

    fs::write(&script_path, script).expect("fake codex script should be written");
    set_executable(&script_path);
}

pub fn install_fake_codex_script(bin_dir: &Path, script: &str) {
    let script_path = bin_dir.join("codex");
    fs::write(&script_path, script).expect("fake codex script should be written");
    set_executable(&script_path);
}

pub fn install_fake_cargo_script(bin_dir: &Path, script: &str) {
    let script_path = bin_dir.join("cargo");
    fs::write(&script_path, script).expect("fake cargo script should be written");
    set_executable(&script_path);
}

pub fn init_temp_git_repo(label: &str) -> PathBuf {
    let repo_dir = unique_temp_dir(label);
    let init_status = Command::new("git")
        .arg("init")
        .current_dir(&repo_dir)
        .output()
        .expect("git init should launch");

    assert!(init_status.status.success());

    repo_dir
}

pub fn init_increment_contract_repo(label: &str) -> PathBuf {
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

pub fn capture_tui_transcript(
    label: &str,
    repo_root: &Path,
    input_steps: Vec<(Duration, Vec<u8>)>,
    bin_dir: &Path,
    args_log: &Path,
    codex_exit_code: &str,
) -> (ExitStatus, String) {
    let binary_path =
        std::env::var("CARGO_BIN_EXE_continuum").expect("continuum binary should be built");
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

pub struct RunningTuiCapture {
    pub child: Child,
    pub stdin: ChildStdin,
    pub transcript_path: PathBuf,
}

pub fn spawn_tui_capture(
    label: &str,
    repo_root: &Path,
    bin_dir: &Path,
    args_log: &Path,
    codex_exit_code: &str,
    cargo_args_log: &Path,
    cargo_done_log: &Path,
) -> RunningTuiCapture {
    spawn_tui_capture_with_size(
        label,
        repo_root,
        bin_dir,
        args_log,
        codex_exit_code,
        cargo_args_log,
        cargo_done_log,
        "80",
        "24",
    )
}

pub fn spawn_tui_capture_with_size(
    label: &str,
    repo_root: &Path,
    bin_dir: &Path,
    args_log: &Path,
    codex_exit_code: &str,
    cargo_args_log: &Path,
    cargo_done_log: &Path,
    columns: &str,
    lines: &str,
) -> RunningTuiCapture {
    let binary_path =
        std::env::var("CARGO_BIN_EXE_continuum").expect("continuum binary should be built");
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
        .env("COLUMNS", columns)
        .env("LINES", lines)
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

pub fn read_transcript(transcript_path: &Path) -> String {
    match fs::read(transcript_path) {
        Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        Err(_) => String::new(),
    }
}

pub fn wait_for_transcript_condition<F>(
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

#[allow(dead_code)]
pub fn submit_increment_mission(stdin: &mut ChildStdin) {
    stdin
        .write_all(INCREMENT_FIX_PROMPT.as_bytes())
        .expect("mission prompt should be written");
    stdin.flush().expect("mission prompt should flush");
    thread::sleep(Duration::from_millis(120));

    stdin
        .write_all(b"\r")
        .expect("mission submit should be written");
    stdin.flush().expect("mission submit should flush");
}
