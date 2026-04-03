use continuum::application::actors::{Builder, Critic, Planner, PostCriticPlanner, Scholar};
use continuum::application::critic_signal::CriticSignal;
use continuum::application::post_critic_signal::PostCriticSignal;
use continuum::application::scholar::MissionScholar;
use continuum::application::session_flow_decision::SessionFlowDecision;
use continuum::{RawMission, ScholarOutput, SessionRunner, SessionStatus};
use std::env;
use std::process::ExitCode;

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

struct ShellBuilder;

impl ShellBuilder {
    fn new() -> Self {
        Self
    }
}

impl Builder for ShellBuilder {
    fn run(&mut self, _scholar_output: &ScholarOutput) {}
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
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 || args[0].trim().is_empty() {
        eprintln!("terminal_outcome=failure");
        eprintln!("error=expected exactly one non-empty prompt argument");
        return ExitCode::from(1);
    }

    let repository_root = match env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            eprintln!("terminal_outcome=failure");
            eprintln!("error=failed to resolve current repository root: {error}");
            return ExitCode::from(1);
        }
    };

    let mission = RawMission::new(&args[0]);
    let mut session_runner = SessionRunner::new(
        Box::new(ShellScholar::new(mission)),
        Box::new(ShellPlanner::new()),
        Box::new(ShellBuilder::new()),
        Box::new(ShellCritic::new()),
    );

    match session_runner.run() {
        Ok(summary) => {
            println!("terminal_outcome=success");
            println!("repository_root={}", repository_root.display());
            println!(
                "session_status={}",
                render_session_status(summary.final_session_status)
            );
            ExitCode::SUCCESS
        }
        Err(report) => {
            eprintln!("terminal_outcome=failure");
            eprintln!("repository_root={}", repository_root.display());
            eprintln!(
                "session_status={}",
                render_session_status(report.final_session_status)
            );
            ExitCode::from(1)
        }
    }
}

fn render_session_status(status: SessionStatus) -> &'static str {
    match status {
        SessionStatus::Active => "active",
        SessionStatus::Completed => "completed",
        SessionStatus::Stopped => "stopped",
    }
}
