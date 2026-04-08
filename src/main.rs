mod entrypoint_cli;
mod codex_local_builder;
mod local_shell_runtime;
mod terminal_rendering;

use entrypoint_cli::read_cli_runtime_request;
use local_shell_runtime::build_local_shell_session_runner;
use std::process::ExitCode;
use terminal_rendering::{render_failure, render_success};

fn main() -> ExitCode {
    let request = match read_cli_runtime_request() {
        Ok(request) => request,
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
