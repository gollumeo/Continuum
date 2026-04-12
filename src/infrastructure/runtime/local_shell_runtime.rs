use crate::infrastructure::execution::codex_local_builder::CodexLocalBuilderAdapter;
use continuum::{
    select_runtime_use_case_authority, Critic, CriticProofRule, CriticSignal, MissionScholar,
    Planner, PostCriticPlanner, PostCriticSignal, RawMission, RuntimeUseCase, Scholar,
    ScholarOutput, SessionFlowDecision, SessionRunner,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn is_increment_contract_fix_use_case(prompt: &str) -> bool {
    select_runtime_use_case_authority(prompt)
        .map(|authority| authority.use_case == RuntimeUseCase::IncrementContractFix)
        .unwrap_or(false)
}

fn is_increment_contract_fix_and_zero_confirm_prompt(prompt: &str) -> bool {
    select_runtime_use_case_authority(prompt)
        .map(|authority| authority.use_case == RuntimeUseCase::IncrementContractFixAndZeroConfirm)
        .unwrap_or(false)
}

pub fn build_local_shell_session_runner(
    mission: RawMission,
    repository_root: PathBuf,
) -> SessionRunner {
    let is_increment_contract_fix = is_increment_contract_fix_use_case(&mission.content);
    let is_increment_contract_fix_and_zero_confirm =
        is_increment_contract_fix_and_zero_confirm_prompt(&mission.content);
    let is_two_file_document_sync = mission
        .content
        .contains("Modify only README.md and project-directives/index.md.");

    if is_increment_contract_fix
        || is_increment_contract_fix_and_zero_confirm
        || is_two_file_document_sync
    {
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
        if _scholar_output.selected_task_scope == "Generate the README.md for this repository." {
            SessionFlowDecision::RefuseUnderspecifiedDocumentPrompt
        } else {
            SessionFlowDecision::Build
        }
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
        if let Some(authority) = select_runtime_use_case_authority(&scholar_output.selected_task_scope)
        {
            match authority.critic_proof_rule {
                Some(CriticProofRule::IncrementContractFix) => {
                    let status = match Command::new("cargo")
                        .current_dir(&self.repository_root)
                        .args([
                            "test",
                            "--test",
                            "increment_contract",
                            "increment_adds_one_to_input",
                            "--",
                            "--exact",
                        ])
                        .status()
                    {
                        Ok(status) => status,
                        Err(_) => return CriticSignal::Stop,
                    };

                    return if status.success() {
                        CriticSignal::Accepted
                    } else {
                        CriticSignal::RevisionRequired
                    };
                }
                Some(CriticProofRule::IncrementContractFixAndZeroConfirm) => {
                    let command_a_status = match Command::new("cargo")
                        .current_dir(&self.repository_root)
                        .args([
                            "test",
                            "--test",
                            "increment_contract",
                            "increment_adds_one_to_input",
                            "--",
                            "--exact",
                        ])
                        .status()
                    {
                        Ok(status) => status,
                        Err(_) => return CriticSignal::Stop,
                    };

                    if !command_a_status.success() {
                        return CriticSignal::RevisionRequired;
                    }

                    let command_b_status = match Command::new("cargo")
                        .current_dir(&self.repository_root)
                        .args([
                            "test",
                            "--test",
                            "increment_contract",
                            "increment_adds_one_to_zero",
                            "--",
                            "--exact",
                        ])
                        .status()
                    {
                        Ok(status) => status,
                        Err(_) => return CriticSignal::Stop,
                    };

                    return if command_b_status.success() {
                        CriticSignal::Accepted
                    } else {
                        CriticSignal::RevisionRequired
                    };
                }
                None => {}
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_contract_fix_and_zero_confirm_prompt_admission_is_exact() {
        assert!(is_increment_contract_fix_and_zero_confirm_prompt(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
        ));
        assert!(!is_increment_contract_fix_and_zero_confirm_prompt(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        ));
        assert!(!is_increment_contract_fix_and_zero_confirm_prompt(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_one' in tests/increment_contract.rs also passes.",
        ));
    }

    #[test]
    fn increment_contract_fix_prompt_admission_is_exact() {
        assert!(is_increment_contract_fix_use_case(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
        ));
        assert!(!is_increment_contract_fix_use_case(
            "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/main.rs.",
        ));
    }

    #[test]
    fn planner_refuses_generate_readme_prompt_without_explicit_allowed_scope() {
        let mut planner = ShellPlanner::new();
        let scholar_output = ScholarOutput::new(
            "Generate the README.md for this repository.",
            "Generate the README.md for this repository.",
        );

        assert_eq!(
            planner.decide(&scholar_output),
            SessionFlowDecision::RefuseUnderspecifiedDocumentPrompt,
        );
    }
}
