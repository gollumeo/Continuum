use std::fs;
use std::io::Write;
use std::thread;
use std::time::Duration;

#[path = "tui_test_harness.rs"]
mod tui_test_harness;

use tui_test_harness::{
    capture_tui_transcript, init_increment_contract_repo, init_temp_git_repo,
    install_fake_cargo_script, install_fake_codex, install_fake_codex_script, read_transcript,
    spawn_tui_capture, unique_temp_dir, wait_for_transcript_condition, INCREMENT_FIX_PROMPT,
};

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
        .write_all(INCREMENT_FIX_PROMPT.as_bytes())
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
