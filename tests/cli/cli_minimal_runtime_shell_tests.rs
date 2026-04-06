use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

    fs::write(
        &script_path,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CODEX_ARGS_LOG\"\npwd > \"$CODEX_PWD_LOG\"\nprintf '%s' \"$CODEX_STDOUT\"\nprintf '%s' \"$CODEX_STDERR\" 1>&2\nexit \"$CODEX_EXIT_CODE\"\n",
    )
    .expect("fake codex script should be written");

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

fn prefixed_path(bin_dir: &Path) -> String {
    let existing_path = std::env::var("PATH").unwrap_or_default();

    if existing_path.is_empty() {
        bin_dir.display().to_string()
    } else {
        format!("{}:{existing_path}", bin_dir.display())
    }
}

#[test]
fn runs_single_session_from_terminal_prompt_on_current_repo() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let temp_dir = unique_temp_dir("codex-success");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");
    let pwd_log = temp_dir.join("codex-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", &pwd_log)
        .env("CODEX_STDOUT", "builder stdout")
        .env("CODEX_STDERR", "builder stderr")
        .env("CODEX_EXIT_CODE", "0")
        .arg("Generate the README.md for this repository. Modify only README.md.")
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let codex_args = fs::read_to_string(&args_log).expect("codex args should be logged");
    let codex_pwd = fs::read_to_string(&pwd_log).expect("codex working dir should be logged");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert!(stdout.contains("builder_issue=completed"));
    assert!(stdout.contains("builder_scope_status=within_scope"));
    assert!(stdout.contains("builder_stdout=builder stdout"));
    assert!(stdout.contains("builder_stderr=builder stderr"));
    assert!(stdout.contains(repo_root));

    assert!(codex_pwd.contains(repo_root));
    assert!(codex_args.contains("exec"));
    assert!(codex_args.contains("-C"));
    assert!(codex_args.contains(repo_root));
    assert!(codex_args.contains("Role: Builder"));
    assert!(codex_args.contains("Allowed file scope: README.md"));
}

#[test]
fn runs_two_file_document_sync_scope_from_terminal_prompt_on_current_repo() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let temp_dir = unique_temp_dir("codex-two-file-success");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");
    let pwd_log = temp_dir.join("codex-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", &pwd_log)
        .env("CODEX_STDOUT", "builder stdout")
        .env("CODEX_STDERR", "builder stderr")
        .env("CODEX_EXIT_CODE", "0")
        .arg(
            "Synchronize README.md and project-directives/index.md. Modify only README.md and project-directives/index.md.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let codex_args = fs::read_to_string(&args_log).expect("codex args should be logged");
    let codex_pwd = fs::read_to_string(&pwd_log).expect("codex working dir should be logged");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert!(stdout.contains("builder_issue=completed"));
    assert!(stdout.contains("builder_scope_status=within_scope"));
    assert!(stdout.contains(
        "builder_allowed_file_scope=README.md,project-directives/index.md"
    ));
    assert!(stdout.contains("builder_stdout=builder stdout"));
    assert!(stdout.contains("builder_stderr=builder stderr"));
    assert!(stdout.contains(repo_root));

    assert!(codex_pwd.contains(repo_root));
    assert!(codex_args.contains("exec"));
    assert!(codex_args.contains("-C"));
    assert!(codex_args.contains(repo_root));
    assert!(codex_args.contains("Role: Builder"));
    assert!(codex_args.contains(
        "Allowed file scope: README.md, project-directives/index.md"
    ));
}

#[test]
fn fails_when_prompt_argument_is_missing() {
    let binary_path =
        std::env::var("CARGO_BIN_EXE_continuum").expect("continuum binary should be built");
    let repo_root = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("error=expected exactly one non-empty prompt argument"));
}

#[test]
fn fails_with_exploitable_builder_issue_when_codex_process_fails() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let temp_dir = unique_temp_dir("codex-failure");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", temp_dir.join("codex-pwd.log"))
        .env("CODEX_STDOUT", "codex failed stdout")
        .env("CODEX_STDERR", "codex failed stderr")
        .env("CODEX_EXIT_CODE", "23")
        .arg("Generate the README.md for this repository. Modify only README.md.")
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let codex_args = fs::read_to_string(&args_log).expect("codex args should be logged");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert!(stderr.contains("builder_issue=process_failed"));
    assert!(stderr.contains("builder_scope_status=within_scope"));
    assert!(stderr.contains("builder_stdout=codex failed stdout"));
    assert!(stderr.contains("builder_stderr=codex failed stderr"));
    assert!(stderr.contains(repo_root));
    assert!(codex_args.contains("Role: Builder"));
    assert!(codex_args.contains("Allowed file scope: README.md"));
}

#[test]
fn fails_closed_when_prompt_has_no_explicit_allowed_scope() {
    let binary_path =
        std::env::var("CARGO_BIN_EXE_continuum").expect("continuum binary should be built");
    let repo_root = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .arg("Do something useful for this repository.")
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert!(stderr.contains("builder_issue=precondition_failed"));
    assert!(stderr.contains("builder_scope_status=not_checked"));
    assert!(stderr.contains("builder_allowed_file_scope="));
    assert!(stderr.contains("builder_changed_files="));
    assert!(stderr.contains(
        "builder_stderr=builder requires an explicit allowed file scope; only README.md or README.md plus project-directives/index.md are admitted in this minimal adapter"
    ));
}

#[test]
fn fails_closed_when_two_file_sync_prompt_only_allows_readme() {
    let binary_path =
        std::env::var("CARGO_BIN_EXE_continuum").expect("continuum binary should be built");
    let repo_root = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .arg("Synchronize README.md and project-directives/index.md. Modify only README.md.")
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert!(stderr.contains("builder_issue=precondition_failed"));
    assert!(stderr.contains("builder_scope_status=not_checked"));
    assert!(stderr.contains("builder_allowed_file_scope="));
    assert!(stderr.contains("builder_changed_files="));
    assert!(stderr.contains(
        "builder_stderr=builder requires an explicit allowed file scope; only README.md or README.md plus project-directives/index.md are admitted in this minimal adapter"
    ));
}

#[test]
fn fails_closed_when_two_file_sync_prompt_only_allows_project_directives_index() {
    let binary_path =
        std::env::var("CARGO_BIN_EXE_continuum").expect("continuum binary should be built");
    let repo_root = env!("CARGO_MANIFEST_DIR");

    let output = Command::new(binary_path)
        .current_dir(repo_root)
        .arg(
            "Synchronize README.md and project-directives/index.md. Modify only project-directives/index.md.",
        )
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert!(stderr.contains("builder_issue=precondition_failed"));
    assert!(stderr.contains("builder_scope_status=not_checked"));
    assert!(stderr.contains("builder_allowed_file_scope="));
    assert!(stderr.contains("builder_changed_files="));
    assert!(stderr.contains(
        "builder_stderr=builder requires an explicit allowed file scope; only README.md or README.md plus project-directives/index.md are admitted in this minimal adapter"
    ));
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
