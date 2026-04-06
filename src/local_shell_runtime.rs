use crate::codex_local_builder::CodexLocalBuilderAdapter;
use continuum::{
    Critic, CriticSignal, MissionScholar, Planner, PostCriticPlanner, PostCriticSignal,
    RawMission, Scholar, ScholarOutput, SessionFlowDecision, SessionRunner,
};
use std::fs;
use std::path::PathBuf;

pub fn build_local_shell_session_runner(
    mission: RawMission,
    repository_root: PathBuf,
) -> SessionRunner {
    let is_two_file_document_sync = mission
        .content
        .contains("Modify only README.md and project-directives/index.md.");

    if is_two_file_document_sync {
        SessionRunner::new_with_retry_budget(
            1,
            Box::new(ShellScholar::new(mission)),
            Box::new(ShellPlanner::new()),
            Box::new(CodexLocalBuilderAdapter::new(repository_root.clone())),
            Box::new(ShellCritic::new(repository_root)),
        )
    } else {
        SessionRunner::new(
            Box::new(ShellScholar::new(mission)),
            Box::new(ShellPlanner::new()),
            Box::new(CodexLocalBuilderAdapter::new(repository_root.clone())),
            Box::new(ShellCritic::new(repository_root)),
        )
    }
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

struct ShellCritic {
    repository_root: PathBuf,
}

impl ShellCritic {
    fn new(repository_root: PathBuf) -> Self {
        Self { repository_root }
    }
}

impl Critic for ShellCritic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal {
        if scholar_output
            .selected_task_scope
            .contains("Modify only README.md and project-directives/index.md.")
        {
            let readme_path = self.repository_root.join("README.md");
            let directives_index_path = self.repository_root.join("project-directives/index.md");

            if !readme_path.is_file() || !directives_index_path.is_file() {
                return CriticSignal::Stop;
            }

            let readme = match fs::read_to_string(&readme_path) {
                Ok(readme) => readme,
                Err(_) => return CriticSignal::Stop,
            };
            let directives_index = match fs::read_to_string(&directives_index_path) {
                Ok(directives_index) => directives_index,
                Err(_) => return CriticSignal::Stop,
            };

            if !readme.contains("project-directives/index.md")
                || !directives_index.contains("README.md")
            {
                return CriticSignal::RevisionRequired;
            }
        }

        CriticSignal::Accepted
    }
}
