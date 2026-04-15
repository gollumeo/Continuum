use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unique_temp_file(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("continuum-{label}-{nanos}.typescript"))
}

fn capture_bootstrap_transcript(
    label: &str,
    input_steps: Vec<(Duration, Vec<u8>)>,
    constrained_terminal: bool,
) -> (ExitStatus, String) {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let transcript_path = unique_temp_file(label);
    let mut command = Command::new("script");

    command
        .current_dir(repo_root)
        .arg("-q")
        .arg("-e")
        .arg("-f")
        .arg("-c")
        .arg(&binary_path)
        .arg(&transcript_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if constrained_terminal {
        command.env("COLUMNS", "24");
        command.env("LINES", "6");
    }

    let mut child = command
        .spawn()
        .expect("bootstrap shell should launch in a pty");
    let mut stdin = child.stdin.take().expect("script stdin should be piped");

    let writer = thread::spawn(move || {
        for (delay, bytes) in input_steps {
            thread::sleep(delay);
            stdin
                .write_all(&bytes)
                .expect("input should be written to bootstrap shell");
            stdin.flush().expect("bootstrap input should flush");
        }
    });

    let status = child
        .wait()
        .expect("bootstrap shell should terminate cleanly");
    writer.join().expect("bootstrap input writer should finish");

    let transcript = String::from_utf8_lossy(
        &fs::read(&transcript_path).expect("bootstrap transcript should be readable"),
    )
    .into_owned();

    let _ = fs::remove_file(&transcript_path);

    (status, transcript)
}

#[test]
fn opens_bootstrap_tui_shell_in_application_held_terminal_surface() {
    let (status, transcript) = capture_bootstrap_transcript(
        "bootstrap-shell-normal",
        vec![
            (Duration::from_millis(200), b"te".to_vec()),
            (Duration::from_millis(200), vec![0x1b]),
        ],
        false,
    );

    assert!(status.success());
    assert!(transcript.contains("\u{1b}[?1049h"));
    assert!(transcript.contains("\u{1b}[?1049l"));
    assert!(transcript.contains("Continuum TUI"));
    assert!(transcript.contains("State: Idle"));
    assert!(transcript.contains("Next: Type a prompt. Esc exits."));
    assert!(transcript.contains("Sessions"));
    assert!(transcript.contains("No sessions yet."));
    assert!(transcript.contains("Prompt [focused]"));
    assert!(transcript.contains("> te"));
    assert!(transcript.contains("\u{1b}[?25h"));
    assert!(transcript.contains("\u{1b}[7;5H"));
}

#[test]
fn simplifies_bootstrap_tui_shell_in_constrained_terminal_with_app_managed_prompt() {
    let (status, transcript) = capture_bootstrap_transcript(
        "bootstrap-shell-constrained",
        vec![
            (Duration::from_millis(200), b"x".to_vec()),
            (Duration::from_millis(200), vec![0x1b]),
        ],
        true,
    );

    assert!(status.success());
    assert!(transcript.contains("\u{1b}[?1049h"));
    assert!(transcript.contains("\u{1b}[?1049l"));
    assert!(transcript.contains("Continuum TUI"));
    assert!(transcript.contains("Idle | Esc exits"));
    assert!(transcript.contains("Supervision: none"));
    assert!(transcript.contains("> x"));
    assert!(transcript.contains("\u{1b}[?25h"));
    assert!(transcript.contains("\u{1b}[4;4H"));
    assert!(!transcript.contains("No session running."));
}
