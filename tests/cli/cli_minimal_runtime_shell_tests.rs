use std::process::Command;

#[test]
fn runs_single_session_from_terminal_prompt_on_current_repo() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .arg("Generate README.md for this repository")
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert!(stdout.contains(repo_root));
}

#[test]
fn keeps_shell_concretes_out_of_runtime_ports_and_avoids_handoff_bridge() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let actors_rs = std::fs::read_to_string(format!("{repo_root}/src/application/actors.rs"))
        .expect("actors.rs should be readable");
    let main_rs = std::fs::read_to_string(format!("{repo_root}/src/main.rs"))
        .expect("main.rs should be readable");

    assert!(!actors_rs.contains("LocalScholar"));
    assert!(!actors_rs.contains("LocalPlanner"));
    assert!(!actors_rs.contains("LocalBuilder"));
    assert!(!actors_rs.contains("LocalCritic"));
    assert!(!main_rs.contains("HandoffDecision"));
    assert!(!main_rs.contains("ScopePlanner"));
}
