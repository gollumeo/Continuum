mod entrypoint_cli;
mod codex_local_builder;
mod terminal_rendering;

use codex_local_builder::CodexLocalBuilderAdapter;
use continuum::{
    Critic, CriticSignal, MissionScholar, Planner, PostCriticPlanner, PostCriticSignal,
    RawMission, Scholar, ScholarOutput, SessionFlowDecision, SessionRunner,
};
use entrypoint_cli::read_cli_runtime_request;
use std::path::PathBuf;
use std::process::ExitCode;
use terminal_rendering::{render_failure, render_success};

struct ShellScholar {
    mission: RawMission,
    mission_scholar: MissionScholar,
}

impl ShellScholar {
    fn new(mission: RawMission) -> Self {
        Self {
            mission,
            mission_scholar: MissionScholar::new(),
        }
    }
}

impl Scholar for ShellScholar {
    fn run(&mut self) -> ScholarOutput {
        self.mission_scholar.transform(&self.mission)
    }
}

struct ShellPlanner {
    post_critic_planner: PostCriticPlanner,
}

impl ShellPlanner {
    fn new() -> Self {
        Self {
            post_critic_planner: PostCriticPlanner,
        }
    }
}

impl Planner for ShellPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        SessionFlowDecision::Build
    }

    fn decide_with_critic_signal(
        &mut self,
        scholar_output: &ScholarOutput,
        critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        self.post_critic_planner
            .decide_with_critic_signal(scholar_output, critic_signal)
    }
}

struct ShellCritic;

impl ShellCritic {
    fn new() -> Self {
        Self
    }
}

impl Critic for ShellCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        CriticSignal::Accepted
    }
}

fn main() -> ExitCode {
    let request = match read_cli_runtime_request() {
        Ok(request) => request,
        Err(error) => return render_failure(None, None, None, Some(&error)),
    };

    let mut session_runner = SessionRunner::new(
        Box::new(ShellScholar::new(request.mission)),
        Box::new(ShellPlanner::new()),
        Box::new(CodexLocalBuilderAdapter::new(request.repository_root.clone())),
        Box::new(ShellCritic::new()),
    );

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
            None,
        ),
    }
}
