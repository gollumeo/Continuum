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
    install_fake_codex_script(
        bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CODEX_ARGS_LOG\"\npwd > \"$CODEX_PWD_LOG\"\nprintf '%s' \"$CODEX_STDOUT\"\nprintf '%s' \"$CODEX_STDERR\" 1>&2\nexit \"$CODEX_EXIT_CODE\"\n",
    );
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

fn git_status_porcelain(repo_dir: &Path) -> String {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .current_dir(repo_dir)
        .output()
        .expect("git status should launch");

    assert!(output.status.success());

    String::from_utf8(output.stdout).expect("git status stdout should be utf-8")
}

fn git_diff(repo_dir: &Path, path: &str) -> String {
    let output = Command::new("git")
        .args(["diff", "--", path])
        .current_dir(repo_dir)
        .output()
        .expect("git diff should launch");

    assert!(output.status.success());

    String::from_utf8(output.stdout).expect("git diff stdout should be utf-8")
}

fn commit_all(repo_dir: &Path) {
    let add_status = Command::new("git")
        .args(["add", "."])
        .current_dir(repo_dir)
        .output()
        .expect("git add should launch");

    assert!(add_status.status.success());

    let commit_status = Command::new("git")
        .args([
            "-c",
            "user.name=Continuum Test",
            "-c",
            "user.email=continuum@example.com",
            "commit",
            "-m",
            "initial",
        ])
        .current_dir(repo_dir)
        .output()
        .expect("git commit should launch");

    assert!(commit_status.status.success());
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

    commit_all(&repo_dir);

    repo_dir
}

fn init_increment_contract_repo_with_zero_confirmation(label: &str) -> PathBuf {
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
        "use increment_contract::increment;\n\n#[test]\nfn increment_adds_one_to_input() {\n    assert_eq!(increment(1), 2);\n}\n\n#[test]\nfn increment_adds_one_to_zero() {\n    assert_eq!(increment(0), 1);\n}\n",
    )
    .expect("increment contract tests should be written");

    commit_all(&repo_dir);

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
        .current_dir(&repo_root)
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
fn runs_increment_contract_fix_and_zero_confirmation_from_repo_root_with_a_then_b_success() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo_with_zero_confirmation(
        "increment-contract-a-then-b-success",
    );
    let temp_dir = unique_temp_dir("increment-contract-a-then-b-success-logs");
    let bin_dir = temp_dir.join("bin");
    let codex_args = temp_dir.join("codex-args.log");
    let codex_pwd = temp_dir.join("codex-pwd.log");
    let cargo_args = temp_dir.join("cargo-args.log");
    let cargo_pwd = temp_dir.join("cargo-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$@\" > \"$CODEX_ARGS_LOG\"\npwd > \"$CODEX_PWD_LOG\"\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\npwd >> \"$CARGO_PWD_LOG\"\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &codex_args)
        .env("CODEX_PWD_LOG", &codex_pwd)
        .env("CARGO_ARGS_LOG", &cargo_args)
        .env("CARGO_PWD_LOG", &cargo_pwd)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let codex_args = fs::read_to_string(&codex_args).expect("codex args should be logged");
    let codex_pwd = fs::read_to_string(&codex_pwd).expect("codex pwd should be logged");
    let cargo_args = fs::read_to_string(&cargo_args).expect("cargo args should be logged");
    let cargo_pwd = fs::read_to_string(&cargo_pwd).expect("cargo pwd should be logged");
    let status = git_status_porcelain(&repo_root);
    let diff = git_diff(&repo_root, "src/lib.rs");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert!(stdout.contains("builder_issue=completed"));
    assert!(stdout.contains("builder_scope_status=within_scope"));
    assert!(stdout.contains("builder_allowed_file_scope=src/lib.rs"));
    assert!(stdout.contains("builder_changed_files=src/lib.rs"));
    assert!(codex_args.contains("exec"));
    assert!(codex_args.contains("-C"));
    assert!(codex_args.contains(&repo_root.display().to_string()));
    assert!(codex_args.contains("Allowed file scope: src/lib.rs"));
    assert_eq!(
        cargo_args,
        "test --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_zero -- --exact\n"
    );
    assert_eq!(
        cargo_pwd,
        format!("{}\n{}\n", repo_root.display(), repo_root.display())
    );
    assert!(codex_pwd.contains(&repo_root.display().to_string()));
    assert!(status.contains(" M src/lib.rs"));
    assert!(!status.contains("Cargo.toml"));
    assert!(!status.contains("tests/increment_contract.rs"));
    assert!(diff.contains("+    input + 1"));
}

#[test]
fn retries_increment_contract_fix_and_zero_confirmation_when_a_fails_before_b_runs() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo_with_zero_confirmation(
        "increment-contract-a-fails-then-retry",
    );
    let temp_dir = unique_temp_dir("increment-contract-a-fails-then-retry-logs");
    let bin_dir = temp_dir.join("bin");
    let cargo_args = temp_dir.join("cargo-args.log");
    let cargo_pwd = temp_dir.join("cargo-pwd.log");
    let cargo_call_count = temp_dir.join("cargo-call-count.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\npwd >> \"$CARGO_PWD_LOG\"\ncount=0\nif [ -f \"$CARGO_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CARGO_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CARGO_CALL_COUNT_LOG\"\nif [ \"$count\" -eq 1 ]; then\n  exit 1\nfi\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CARGO_ARGS_LOG", &cargo_args)
        .env("CARGO_PWD_LOG", &cargo_pwd)
        .env("CARGO_CALL_COUNT_LOG", &cargo_call_count)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let cargo_args = fs::read_to_string(&cargo_args).expect("cargo args should be logged");
    let cargo_pwd = fs::read_to_string(&cargo_pwd).expect("cargo pwd should be logged");
    let cargo_call_count =
        fs::read_to_string(&cargo_call_count).expect("cargo call count should be logged");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert_eq!(cargo_call_count, "3");
    assert_eq!(
        cargo_args,
        "test --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_zero -- --exact\n"
    );
    assert_eq!(
        cargo_pwd,
        format!("{}\n{}\n{}\n", repo_root.display(), repo_root.display(), repo_root.display())
    );
}

#[test]
fn retries_increment_contract_fix_and_zero_confirmation_when_b_fails_after_a_succeeds() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo_with_zero_confirmation(
        "increment-contract-b-fails-then-retry",
    );
    let temp_dir = unique_temp_dir("increment-contract-b-fails-then-retry-logs");
    let bin_dir = temp_dir.join("bin");
    let cargo_args = temp_dir.join("cargo-args.log");
    let cargo_pwd = temp_dir.join("cargo-pwd.log");
    let cargo_call_count = temp_dir.join("cargo-call-count.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\npwd >> \"$CARGO_PWD_LOG\"\ncount=0\nif [ -f \"$CARGO_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CARGO_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CARGO_CALL_COUNT_LOG\"\nif [ \"$count\" -eq 2 ]; then\n  exit 1\nfi\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CARGO_ARGS_LOG", &cargo_args)
        .env("CARGO_PWD_LOG", &cargo_pwd)
        .env("CARGO_CALL_COUNT_LOG", &cargo_call_count)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let cargo_args = fs::read_to_string(&cargo_args).expect("cargo args should be logged");
    let cargo_pwd = fs::read_to_string(&cargo_pwd).expect("cargo pwd should be logged");
    let cargo_call_count =
        fs::read_to_string(&cargo_call_count).expect("cargo call count should be logged");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert_eq!(cargo_call_count, "4");
    assert_eq!(
        cargo_args,
        "test --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_zero -- --exact\ntest --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_zero -- --exact\n"
    );
    assert_eq!(
        cargo_pwd,
        format!("{}\n{}\n{}\n{}\n", repo_root.display(), repo_root.display(), repo_root.display(), repo_root.display())
    );
}

#[test]
fn stops_increment_contract_fix_and_zero_confirmation_after_second_b_failure() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo_with_zero_confirmation(
        "increment-contract-b-fails-twice",
    );
    let temp_dir = unique_temp_dir("increment-contract-b-fails-twice-logs");
    let bin_dir = temp_dir.join("bin");
    let cargo_args = temp_dir.join("cargo-args.log");
    let cargo_pwd = temp_dir.join("cargo-pwd.log");
    let cargo_call_count = temp_dir.join("cargo-call-count.log");
    let status_before = git_status_porcelain(&repo_root);

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$CARGO_ARGS_LOG\"\npwd >> \"$CARGO_PWD_LOG\"\ncount=0\nif [ -f \"$CARGO_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CARGO_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CARGO_CALL_COUNT_LOG\"\nif [ \"$count\" -eq 2 ] || [ \"$count\" -eq 4 ]; then\n  exit 1\nfi\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CARGO_ARGS_LOG", &cargo_args)
        .env("CARGO_PWD_LOG", &cargo_pwd)
        .env("CARGO_CALL_COUNT_LOG", &cargo_call_count)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        )
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let cargo_args = fs::read_to_string(&cargo_args).expect("cargo args should be logged");
    let cargo_pwd = fs::read_to_string(&cargo_pwd).expect("cargo pwd should be logged");
    let cargo_call_count =
        fs::read_to_string(&cargo_call_count).expect("cargo call count should be logged");
    let status_after = git_status_porcelain(&repo_root);
    let diff = git_diff(&repo_root, "src/lib.rs");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert!(stderr.contains("builder_allowed_file_scope=src/lib.rs"));
    assert!(stderr.contains("error=exhausted retry budget while confirming increment contract tests"));
    assert_eq!(cargo_call_count, "4");
    assert_eq!(
        cargo_args,
        "test --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_zero -- --exact\ntest --test increment_contract increment_adds_one_to_input -- --exact\ntest --test increment_contract increment_adds_one_to_zero -- --exact\n"
    );
    assert_eq!(
        cargo_pwd,
        format!("{}\n{}\n{}\n{}\n", repo_root.display(), repo_root.display(), repo_root.display(), repo_root.display())
    );
    assert_ne!(status_after, status_before);
    assert!(status_after.contains(" M src/lib.rs"));
    assert!(!status_after.contains("Cargo.toml"));
    assert!(!status_after.contains("tests/increment_contract.rs"));
    assert!(diff.contains("+    input + 1"));
}

#[test]
fn runs_increment_contract_fix_builder_from_repo_root_with_src_lib_rs_scope_only() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo("increment-contract-builder");
    let temp_dir = unique_temp_dir("increment-contract-builder-logs");
    let bin_dir = temp_dir.join("bin");
    let codex_args = temp_dir.join("codex-args.log");
    let codex_pwd = temp_dir.join("codex-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CODEX_ARGS_LOG\"\npwd > \"$CODEX_PWD_LOG\"\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &codex_args)
        .env("CODEX_PWD_LOG", &codex_pwd)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let codex_args = fs::read_to_string(&codex_args).expect("codex args should be logged");
    let codex_pwd = fs::read_to_string(&codex_pwd).expect("codex pwd should be logged");
    let status = git_status_porcelain(&repo_root);
    let diff = git_diff(&repo_root, "src/lib.rs");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert!(stdout.contains("builder_issue=completed"));
    assert!(stdout.contains("builder_scope_status=within_scope"));
    assert!(stdout.contains("builder_allowed_file_scope=src/lib.rs"));
    assert!(stdout.contains("builder_changed_files=src/lib.rs"));
    assert!(codex_args.contains("exec"));
    assert!(codex_args.contains("-C"));
    assert!(codex_args.contains(&repo_root.display().to_string()));
    assert!(codex_args.contains("Allowed file scope: src/lib.rs"));
    assert!(codex_pwd.contains(&repo_root.display().to_string()));
    assert!(status.contains(" M src/lib.rs"));
    assert!(!status.contains("Cargo.toml"));
    assert!(!status.contains("tests/increment_contract.rs"));
    assert!(diff.contains("+    input + 1"));
}

#[test]
fn runs_exact_increment_contract_proof_command_from_repo_root() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo("increment-contract-proof");
    let temp_dir = unique_temp_dir("increment-contract-proof-logs");
    let bin_dir = temp_dir.join("bin");
    let cargo_args = temp_dir.join("cargo-args.log");
    let cargo_pwd = temp_dir.join("cargo-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CARGO_ARGS_LOG\"\npwd > \"$CARGO_PWD_LOG\"\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CARGO_ARGS_LOG", &cargo_args)
        .env("CARGO_PWD_LOG", &cargo_pwd)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let cargo_args = fs::read_to_string(&cargo_args).expect("cargo args should be logged");
    let cargo_pwd = fs::read_to_string(&cargo_pwd).expect("cargo pwd should be logged");

    assert_eq!(
        cargo_args,
        "test\n--test\nincrement_contract\nincrement_adds_one_to_input\n--\n--exact\n"
    );
    assert_eq!(cargo_pwd, format!("{}\n", repo_root.display()));
}

#[test]
fn retries_increment_contract_fix_exactly_once_after_revision_required_then_succeeds() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo("increment-contract-retry-success");
    let temp_dir = unique_temp_dir("increment-contract-retry-success-logs");
    let bin_dir = temp_dir.join("bin");
    let codex_args = temp_dir.join("codex-args.log");
    let codex_pwd = temp_dir.join("codex-pwd.log");
    let codex_call_count = temp_dir.join("codex-call-count.log");
    let cargo_args = temp_dir.join("cargo-args.log");
    let cargo_pwd = temp_dir.join("cargo-pwd.log");
    let cargo_call_count = temp_dir.join("cargo-call-count.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$CODEX_ARGS_LOG\"\npwd >> \"$CODEX_PWD_LOG\"\ncount=0\nif [ -f \"$CODEX_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CODEX_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CODEX_CALL_COUNT_LOG\"\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$CARGO_ARGS_LOG\"\npwd >> \"$CARGO_PWD_LOG\"\ncount=0\nif [ -f \"$CARGO_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CARGO_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CARGO_CALL_COUNT_LOG\"\nif [ \"$count\" -eq 1 ]; then\n  exit 1\nfi\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &codex_args)
        .env("CODEX_PWD_LOG", &codex_pwd)
        .env("CODEX_CALL_COUNT_LOG", &codex_call_count)
        .env("CARGO_ARGS_LOG", &cargo_args)
        .env("CARGO_PWD_LOG", &cargo_pwd)
        .env("CARGO_CALL_COUNT_LOG", &cargo_call_count)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let codex_call_count =
        fs::read_to_string(&codex_call_count).expect("codex call count should be logged");
    let cargo_call_count =
        fs::read_to_string(&cargo_call_count).expect("cargo call count should be logged");
    let cargo_args = fs::read_to_string(&cargo_args).expect("cargo args should be logged");
    let cargo_pwd = fs::read_to_string(&cargo_pwd).expect("cargo pwd should be logged");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert_eq!(codex_call_count, "2");
    assert_eq!(cargo_call_count, "2");
    assert_eq!(
        cargo_args,
        "test\n--test\nincrement_contract\nincrement_adds_one_to_input\n--\n--exact\ntest\n--test\nincrement_contract\nincrement_adds_one_to_input\n--\n--exact\n"
    );
    assert_eq!(
        cargo_pwd,
        format!("{}\n{}\n", repo_root.display(), repo_root.display())
    );
}

#[test]
fn stops_after_second_increment_contract_proof_failure_exhausts_retry_budget() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_increment_contract_repo("increment-contract-retry-stop");
    let temp_dir = unique_temp_dir("increment-contract-retry-stop-logs");
    let bin_dir = temp_dir.join("bin");
    let codex_call_count = temp_dir.join("codex-call-count.log");
    let cargo_call_count = temp_dir.join("cargo-call-count.log");

    fs::create_dir_all(&bin_dir).expect("fake tool bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\ncount=0\nif [ -f \"$CODEX_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CODEX_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CODEX_CALL_COUNT_LOG\"\nprintf 'pub fn increment(input: i32) -> i32 {\\n    input + 1\\n}\\n' > src/lib.rs\nexit 0\n",
    );
    install_fake_cargo_script(
        &bin_dir,
        "#!/bin/sh\ncount=0\nif [ -f \"$CARGO_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CARGO_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CARGO_CALL_COUNT_LOG\"\nexit 1\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_CALL_COUNT_LOG", &codex_call_count)
        .env("CARGO_CALL_COUNT_LOG", &cargo_call_count)
        .arg(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        )
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let codex_call_count =
        fs::read_to_string(&codex_call_count).expect("codex call count should be logged");
    let cargo_call_count =
        fs::read_to_string(&cargo_call_count).expect("cargo call count should be logged");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert_eq!(codex_call_count, "2");
    assert_eq!(cargo_call_count, "2");
}

#[test]
fn fails_with_explicit_terminal_refusal_for_underspecified_readme_prompt() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_temp_git_repo("underspecified-readme-refusal-repo");
    fs::write(repo_root.join("README.md"), "# Continuum\n")
        .expect("README.md should be written");
    let temp_dir = unique_temp_dir("underspecified-readme-refusal");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", temp_dir.join("codex-pwd.log"))
        .env("CODEX_STDOUT", "builder stdout")
        .env("CODEX_STDERR", "builder stderr")
        .env("CODEX_EXIT_CODE", "0")
        .arg("Generate the README.md for this repository.")
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains(
        "error=refused to act on an underspecified document prompt; add an explicit allowed file scope"
    ));
    assert!(stderr.contains("session_status=stopped"));
    assert!(!stderr.contains("builder_issue="));
    assert!(!args_log.exists());
}

#[test]
fn leaves_repository_unchanged_when_underspecified_readme_prompt_is_refused() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_temp_git_repo("underspecified-readme-no-side-effects-repo");
    let readme_path = repo_root.join("README.md");
    fs::write(&readme_path, "# Continuum\nInitial content\n")
        .expect("README.md should be written");
    let status_before = git_status_porcelain(&repo_root);
    let readme_before = fs::read_to_string(&readme_path).expect("README.md should be readable");
    let temp_dir = unique_temp_dir("underspecified-readme-no-side-effects");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", temp_dir.join("codex-pwd.log"))
        .env("CODEX_STDOUT", "builder stdout")
        .env("CODEX_STDERR", "builder stderr")
        .env("CODEX_EXIT_CODE", "0")
        .arg("Generate the README.md for this repository.")
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let status_after = git_status_porcelain(&repo_root);
    let readme_after = fs::read_to_string(&readme_path).expect("README.md should be readable");

    assert_eq!(status_after, status_before);
    assert_eq!(readme_after, readme_before);
    assert!(!args_log.exists());
    assert!(!repo_root.join("project-directives").exists());
}

#[test]
fn runs_two_file_document_sync_scope_on_repo_with_both_canonical_files() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_temp_git_repo("codex-two-file-success-repo");
    fs::write(
        repo_root.join("README.md"),
        "# Continuum\nSee project-directives/index.md\n",
    )
    .expect("README.md should be written");
    fs::create_dir_all(repo_root.join("project-directives"))
        .expect("project-directives dir should be created");
    fs::write(
        repo_root.join("project-directives/index.md"),
        "# Project Directives Index\nSee README.md\n",
    )
    .expect("project-directives/index.md should be written");
    let temp_dir = unique_temp_dir("codex-two-file-success");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");
    let pwd_log = temp_dir.join("codex-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex(&bin_dir);

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
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
    assert!(stdout.contains(&repo_root.display().to_string()));

    assert!(codex_pwd.contains(&repo_root.display().to_string()));
    assert!(codex_args.contains("exec"));
    assert!(codex_args.contains("-C"));
    assert!(codex_args.contains(&repo_root.display().to_string()));
    assert!(codex_args.contains("Role: Builder"));
    assert!(codex_args.contains(
        "Allowed file scope: README.md, project-directives/index.md"
    ));
}

#[test]
fn stops_when_two_file_sync_run_leaves_project_directives_index_missing() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_temp_git_repo("codex-two-file-missing-index-repo");
    let temp_dir = unique_temp_dir("codex-two-file-missing-index");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");
    let pwd_log = temp_dir.join("codex-pwd.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CODEX_ARGS_LOG\"\npwd > \"$CODEX_PWD_LOG\"\nprintf '# Continuum\\nSee project-directives/index.md\\n' > README.md\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", &pwd_log)
        .arg(
            "Synchronize README.md and project-directives/index.md. Modify only README.md and project-directives/index.md.",
        )
        .output()
        .expect("binary should launch");

    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");
    let codex_args = fs::read_to_string(&args_log).expect("codex args should be logged");
    let codex_pwd = fs::read_to_string(&pwd_log).expect("codex working dir should be logged");

    assert!(stderr.contains("terminal_outcome=failure"));
    assert!(stderr.contains("session_status=stopped"));
    assert!(stderr.contains("builder_issue=completed"));
    assert!(stderr.contains("builder_scope_status=within_scope"));
    assert!(stderr.contains(
        "builder_allowed_file_scope=README.md,project-directives/index.md"
    ));
    assert!(stderr.contains("builder_changed_files=README.md"));
    assert!(codex_pwd.contains(&repo_root.display().to_string()));
    assert!(codex_args.contains(
        "Allowed file scope: README.md, project-directives/index.md"
    ));
}

#[test]
fn completes_two_file_sync_after_one_runtime_retry() {
    let binary_path = std::env::var("CARGO_BIN_EXE_continuum")
        .expect("continuum binary should be built for this test");
    let repo_root = init_temp_git_repo("codex-two-file-retry-repo");
    let temp_dir = unique_temp_dir("codex-two-file-retry");
    let bin_dir = temp_dir.join("bin");
    let args_log = temp_dir.join("codex-args.log");
    let pwd_log = temp_dir.join("codex-pwd.log");
    let call_count_log = temp_dir.join("codex-call-count.log");

    fs::create_dir_all(&bin_dir).expect("fake codex bin dir should be created");
    install_fake_codex_script(
        &bin_dir,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$CODEX_ARGS_LOG\"\npwd >> \"$CODEX_PWD_LOG\"\ncount=0\nif [ -f \"$CODEX_CALL_COUNT_LOG\" ]; then\n  count=$(cat \"$CODEX_CALL_COUNT_LOG\")\nfi\ncount=$((count + 1))\nprintf '%s' \"$count\" > \"$CODEX_CALL_COUNT_LOG\"\nmkdir -p project-directives\nprintf '# Continuum\\nSee project-directives/index.md\\n' > README.md\nif [ \"$count\" -eq 1 ]; then\n  printf '# Project Directives Index\\n' > project-directives/index.md\nelse\n  printf '# Project Directives Index\\nSee README.md\\n' > project-directives/index.md\nfi\nexit 0\n",
    );

    let output = Command::new(binary_path)
        .current_dir(&repo_root)
        .env("PATH", prefixed_path(&bin_dir))
        .env("CODEX_ARGS_LOG", &args_log)
        .env("CODEX_PWD_LOG", &pwd_log)
        .env("CODEX_CALL_COUNT_LOG", &call_count_log)
        .arg(
            "Synchronize README.md and project-directives/index.md. Modify only README.md and project-directives/index.md.",
        )
        .output()
        .expect("binary should launch");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let codex_args = fs::read_to_string(&args_log).expect("codex args should be logged");
    let codex_pwd = fs::read_to_string(&pwd_log).expect("codex working dir should be logged");
    let call_count = fs::read_to_string(&call_count_log).expect("codex call count should exist");
    let readme = fs::read_to_string(repo_root.join("README.md")).expect("README.md should exist");
    let directives_index = fs::read_to_string(repo_root.join("project-directives/index.md"))
        .expect("project-directives/index.md should exist");

    assert!(stdout.contains("terminal_outcome=success"));
    assert!(stdout.contains("session_status=completed"));
    assert!(stdout.contains("builder_issue=completed"));
    assert!(stdout.contains("builder_scope_status=within_scope"));
    assert!(stdout.contains(
        "builder_allowed_file_scope=README.md,project-directives/index.md"
    ));
    assert_eq!(call_count, "2");
    assert!(codex_args.matches("exec").count() >= 2);
    assert!(codex_args.contains(
        "Allowed file scope: README.md, project-directives/index.md"
    ));
    assert!(codex_pwd.matches(&repo_root.display().to_string()).count() >= 2);
    assert!(readme.contains("project-directives/index.md"));
    assert!(directives_index.contains("README.md"));
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
