use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[path = "tui_test_harness.rs"]
mod tui_test_harness;

use tui_test_harness::{
    capture_tui_transcript, init_increment_contract_repo, init_temp_git_repo,
    install_fake_cargo_script, install_fake_codex, install_fake_codex_script, read_transcript,
    spawn_tui_capture, spawn_tui_capture_with_size, submit_increment_mission, unique_temp_dir,
    wait_for_transcript_condition, INCREMENT_FIX_PROMPT,
};

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

fn strip_csi_sequences(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }

        if ch != '\r' {
            output.push(ch);
        }
    }

    output
}

fn latest_visible_session_rows(transcript: &str) -> Vec<String> {
    let Some(frame_start) = transcript.rfind("Continuum TUI") else {
        return Vec::new();
    };

    transcript[frame_start..]
        .lines()
        .map(strip_csi_sequences)
        .take_while(|line| !line.starts_with("Prompt [focused]"))
        .filter(|line| line.starts_with("  [") || line.starts_with("> ["))
        .collect()
}

fn latest_visible_selected_position(transcript: &str) -> Option<usize> {
    latest_visible_session_rows(transcript)
        .iter()
        .position(|line| line.starts_with("> "))
}

fn latest_visible_selected_count(transcript: &str) -> usize {
    latest_visible_session_rows(transcript)
        .iter()
        .filter(|line| line.starts_with("> "))
        .count()
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
        .write_all(INCREMENT_FIX_PROMPT.as_bytes())
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
        .write_all(INCREMENT_FIX_PROMPT.as_bytes())
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

#[test]
fn shows_stopped_session_row_after_mission_refusal() {
    let repo_root = init_temp_git_repo("tui-session-list-stopped-repo");
    let temp_dir = unique_temp_dir("tui-session-list-stopped-logs");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    fs::write(repo_root.join("README.md"), "# Continuum\n").expect("README.md should be written");
    install_fake_codex(&bin_dir);

    let (status, transcript) = capture_tui_transcript(
        "tui-session-list-stopped",
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
    assert!(transcript.contains("[!]"));
}

#[test]
fn shows_selection_prefix_on_session_row_after_mission_completes() {
    let repo_root = init_increment_contract_repo("tui-session-list-select-repo");
    let temp_dir = unique_temp_dir("tui-session-list-select-logs");
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
        "tui-session-list-select",
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
        .write_all(INCREMENT_FIX_PROMPT.as_bytes())
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

    // Send Down arrow — selection stays clamped at idx 0, row remains selected
    capture
        .stdin
        .write_all(&[0x1b, 0x5b, 0x42])
        .expect("down arrow should be written");
    capture.stdin.flush().expect("down arrow should flush");
    thread::sleep(Duration::from_millis(100));

    capture
        .stdin
        .write_all(&[0x1b])
        .expect("escape should be written");
    capture.stdin.flush().expect("escape should flush");

    let status = capture.child.wait().expect("tui shell should terminate");
    let final_transcript = read_transcript(&capture.transcript_path);
    let _ = fs::remove_file(&capture.transcript_path);

    assert!(status.success());
    assert!(final_transcript.contains("> [+]"));
}

#[test]
fn keeps_selection_visible_and_moving_within_overflow_window_on_up_down_navigation() {
    let repo_root = init_increment_contract_repo("tui-session-list-overflow-focus-repo");
    let temp_dir = unique_temp_dir("tui-session-list-overflow-focus-logs");
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
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\nexit 0\n",
    );

    let mut capture = spawn_tui_capture_with_size(
        "tui-session-list-overflow-focus",
        &repo_root,
        &bin_dir,
        &codex_args_log,
        "0",
        &cargo_args_log,
        &cargo_done_log,
        "80",
        "9",
    );

    thread::sleep(Duration::from_millis(200));
    for _ in 0..5 {
        submit_increment_mission(&mut capture.stdin);
        thread::sleep(Duration::from_millis(250));
    }

    wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(5),
        |transcript| {
            latest_visible_session_rows(transcript).len() == 3
                && latest_visible_selected_position(transcript) == Some(2)
                && latest_visible_selected_count(transcript) == 1
        },
    );

    capture
        .stdin
        .write_all(&[0x1b, 0x5b, 0x41])
        .expect("up arrow should be written");
    capture.stdin.flush().expect("up arrow should flush");
    wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(2),
        |transcript| {
            latest_visible_selected_position(transcript) == Some(1)
                && latest_visible_selected_count(transcript) == 1
        },
    );

    capture
        .stdin
        .write_all(&[0x1b, 0x5b, 0x41])
        .expect("up arrow should be written");
    capture.stdin.flush().expect("up arrow should flush");
    wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(2),
        |transcript| {
            latest_visible_selected_position(transcript) == Some(0)
                && latest_visible_selected_count(transcript) == 1
        },
    );

    capture
        .stdin
        .write_all(&[0x1b, 0x5b, 0x41])
        .expect("up arrow should be written");
    capture.stdin.flush().expect("up arrow should flush");
    wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(2),
        |transcript| {
            latest_visible_selected_position(transcript) == Some(0)
                && latest_visible_selected_count(transcript) == 1
        },
    );

    capture
        .stdin
        .write_all(&[0x1b, 0x5b, 0x42])
        .expect("down arrow should be written");
    capture.stdin.flush().expect("down arrow should flush");
    wait_for_transcript_condition(
        &capture.transcript_path,
        Duration::from_secs(2),
        |transcript| {
            latest_visible_selected_position(transcript) == Some(1)
                && latest_visible_selected_count(transcript) == 1
        },
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
    assert_eq!(latest_visible_selected_count(&final_transcript), 1);
    assert_eq!(latest_visible_session_rows(&final_transcript).len(), 3);
}
