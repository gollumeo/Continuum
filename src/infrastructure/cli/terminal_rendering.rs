use continuum::{BuilderIssue, BuilderRunReport, BuilderScopeStatus, SessionStatus};
use std::path::Path;
use std::process::ExitCode;

pub fn render_success(
    repository_root: &Path,
    builder_report: Option<&BuilderRunReport>,
    session_status: SessionStatus,
) -> ExitCode {
    println!("terminal_outcome=success");
    println!("repository_root={}", repository_root.display());
    if let Some(report) = builder_report {
        render_builder_report_stdout(report);
    }
    println!("session_status={}", render_session_status(session_status));
    ExitCode::SUCCESS
}

pub fn render_failure(
    repository_root: Option<&Path>,
    builder_report: Option<&BuilderRunReport>,
    session_status: Option<SessionStatus>,
    error: Option<&str>,
) -> ExitCode {
    eprintln!("terminal_outcome=failure");
    if let Some(error) = error {
        eprintln!("error={error}");
    }
    if let Some(repository_root) = repository_root {
        eprintln!("repository_root={}", repository_root.display());
    }
    if let Some(builder_report) = builder_report {
        render_builder_report_stderr(builder_report);
    }
    if let Some(session_status) = session_status {
        eprintln!("session_status={}", render_session_status(session_status));
    }
    ExitCode::from(1)
}

fn render_builder_report_stdout(report: &BuilderRunReport) {
    println!("builder_issue={}", render_builder_issue(&report.issue));
    println!(
        "builder_scope_status={}",
        render_builder_scope_status(&report.scope_status)
    );
    println!(
        "builder_allowed_file_scope={}",
        report.allowed_file_scope.join(",")
    );
    println!("builder_changed_files={}", report.changed_files.join(","));
    println!("builder_stdout={}", render_terminal_field(&report.stdout));
    println!("builder_stderr={}", render_terminal_field(&report.stderr));
}

fn render_builder_report_stderr(report: &BuilderRunReport) {
    eprintln!("builder_issue={}", render_builder_issue(&report.issue));
    eprintln!(
        "builder_scope_status={}",
        render_builder_scope_status(&report.scope_status)
    );
    eprintln!(
        "builder_allowed_file_scope={}",
        report.allowed_file_scope.join(",")
    );
    eprintln!("builder_changed_files={}", report.changed_files.join(","));
    eprintln!("builder_stdout={}", render_terminal_field(&report.stdout));
    eprintln!("builder_stderr={}", render_terminal_field(&report.stderr));
}

fn render_builder_issue(issue: &BuilderIssue) -> &'static str {
    match issue {
        BuilderIssue::Completed => "completed",
        BuilderIssue::PreconditionFailed => "precondition_failed",
        BuilderIssue::LaunchFailed => "launch_failed",
        BuilderIssue::ProcessFailed => "process_failed",
        BuilderIssue::ScopeViolated => "scope_violated",
    }
}

fn render_builder_scope_status(status: &BuilderScopeStatus) -> &'static str {
    match status {
        BuilderScopeStatus::NotChecked => "not_checked",
        BuilderScopeStatus::WithinScope => "within_scope",
        BuilderScopeStatus::Violated => "violated",
    }
}

fn render_terminal_field(value: &str) -> String {
    value.replace('\r', "").replace('\n', "\\n")
}

fn render_session_status(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Completed => "completed",
        SessionStatus::Stopped => "stopped",
    }
}
