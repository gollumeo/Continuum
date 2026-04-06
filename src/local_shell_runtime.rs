use crate::codex_local_builder::CodexLocalBuilderAdapter;
use continuum::{
    Critic, CriticSignal, MissionScholar, Planner, PostCriticPlanner, PostCriticSignal,
    RawMission, Scholar, ScholarOutput, SessionFlowDecision, SessionRunner,
};
use std::path::PathBuf;

pub fn build_local_shell_session_runner(
    mission: RawMission,
    repository_root: PathBuf,
) -> SessionRunner {
    SessionRunner::new(
        Box::new(ShellScholar::new(mission)),
        Box::new(ShellPlanner::new()),
        Box::new(CodexLocalBuilderAdapter::new(repository_root)),
        Box::new(ShellCritic::new()),
    )
}

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
