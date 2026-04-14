mod infrastructure;

use infrastructure::cli::bootstrap_tui_shell::run_bootstrap_tui_shell;
use infrastructure::cli::entrypoint_cli::{read_cli_entrypoint, CliEntrypoint};
use infrastructure::cli::terminal_rendering::{render_failure, render_success};
use infrastructure::runtime::local_shell_runtime::build_local_shell_session_runner;
use std::process::ExitCode;

fn main() -> ExitCode {
    let request = match read_cli_entrypoint() {
        Ok(CliEntrypoint::BootstrapTuiShell) => return run_bootstrap_tui_shell(),
        Ok(CliEntrypoint::RuntimeRequest(request)) => request,
        Err(error) => return render_failure(None, None, None, Some(&error)),
    };

    let mut session_runner =
        build_local_shell_session_runner(request.mission, request.repository_root.clone());

    let result = session_runner.run();
    let builder_report = session_runner.last_builder_report().cloned();

    match result {
        Ok(summary) => render_success(
            &request.repository_root,
            builder_report.as_ref(),
            summary.final_session_status,
        ),
        Err(report) => render_failure(
            Some(&request.repository_root),
            builder_report.as_ref(),
            Some(report.final_session_status),
            report.error,
        ),
    }
}
