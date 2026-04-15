use crate::infrastructure::execution::codex_local_builder::CodexLocalBuilderAdapter;
use continuum::{
    select_runtime_use_case_authority, Builder, BuilderIssue, BuilderRunReport, BuilderScopeStatus,
    Critic, CriticProofRule, CriticSignal, FailureReport, MissionScholar, Planner,
    PostCriticPlanner, PostCriticSignal, RawMission, RuntimeUseCase, Scholar, ScholarOutput,
    SessionFlowDecision, SessionRunner, SessionSummary,
};
use std::cell::{Cell, RefCell};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::rc::Rc;

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

fn uses_retry_budget(prompt: &str) -> bool {
    is_increment_contract_fix_use_case(prompt)
        || is_increment_contract_fix_and_zero_confirm_prompt(prompt)
        || prompt.contains("Modify only README.md and project-directives/index.md.")
}

pub struct LocalShellSessionRunOutcome {
    pub entered_admitted_path: bool,
    pub result: Result<SessionSummary, FailureReport>,
}

pub fn build_local_shell_session_runner(
    mission: RawMission,
    repository_root: PathBuf,
) -> SessionRunner {
    if uses_retry_budget(&mission.content) {
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

pub fn run_local_shell_session_with_admission_hook<F>(
    mission: RawMission,
    repository_root: PathBuf,
    on_admitted: F,
) -> Result<LocalShellSessionRunOutcome, String>
where
    F: FnMut() -> Result<(), String> + 'static,
{
    let entered_admitted_path = Rc::new(Cell::new(false));
    let admission_hook_error = Rc::new(RefCell::new(None));
    let builder = AdmissionObservedBuilder::new(
        CodexLocalBuilderAdapter::new(repository_root.clone()),
        entered_admitted_path.clone(),
        admission_hook_error.clone(),
        on_admitted,
    );

    let mut session_runner = if uses_retry_budget(&mission.content) {
        SessionRunner::new_with_retry_budget(
            1,
            Box::new(ShellScholar::new(mission)),
            Box::new(ShellPlanner::new()),
            Box::new(builder),
            Box::new(ShellCritic::for_tui(repository_root)),
        )
    } else {
        SessionRunner::new(
            Box::new(ShellScholar::new(mission)),
            Box::new(ShellPlanner::new()),
            Box::new(builder),
            Box::new(ShellCritic::for_tui(repository_root)),
        )
    };

    let result = session_runner.run();

    if let Some(error) = admission_hook_error.borrow_mut().take() {
        return Err(error);
    }

    Ok(LocalShellSessionRunOutcome {
        entered_admitted_path: entered_admitted_path.get(),
        result,
    })
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

struct AdmissionObservedBuilder<F> {
    inner: CodexLocalBuilderAdapter,
    entered_admitted_path: Rc<Cell<bool>>,
    admission_hook_error: Rc<RefCell<Option<String>>>,
    on_admitted: F,
}

impl<F> AdmissionObservedBuilder<F> {
    fn new(
        inner: CodexLocalBuilderAdapter,
        entered_admitted_path: Rc<Cell<bool>>,
        admission_hook_error: Rc<RefCell<Option<String>>>,
        on_admitted: F,
    ) -> Self {
        Self {
            inner,
            entered_admitted_path,
            admission_hook_error,
            on_admitted,
        }
    }
}

impl<F> Builder for AdmissionObservedBuilder<F>
where
    F: FnMut() -> Result<(), String>,
{
    fn run(&mut self, scholar_output: &ScholarOutput) -> BuilderRunReport {
        if !self.entered_admitted_path.replace(true) {
            if let Err(error) = (self.on_admitted)() {
                *self.admission_hook_error.borrow_mut() = Some(error.clone());

                return BuilderRunReport {
                    issue: BuilderIssue::PreconditionFailed,
                    scope_status: BuilderScopeStatus::NotChecked,
                    allowed_file_scope: Vec::new(),
                    changed_files: Vec::new(),
                    stdout: String::new(),
                    stderr: format!("failed to render admitted mission state: {error}"),
                };
            }
        }

        self.inner.run(scholar_output)
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
    suppress_proof_command_output: bool,
}

impl ShellCritic {
    fn new(repository_root: PathBuf) -> Self {
        Self::with_terminal_ownership(repository_root, false)
    }

    fn for_tui(repository_root: PathBuf) -> Self {
        Self::with_terminal_ownership(repository_root, true)
    }

    fn with_terminal_ownership(
        repository_root: PathBuf,
        suppress_proof_command_output: bool,
    ) -> Self {
        Self {
            repository_root,
            suppress_proof_command_output,
        }
    }

    fn proof_command(&self) -> Command {
        let mut command = Command::new("cargo");
        command.current_dir(&self.repository_root);

        if self.suppress_proof_command_output {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }

        command
    }
}

impl Critic for ShellCritic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal {
        if let Some(authority) =
            select_runtime_use_case_authority(&scholar_output.selected_task_scope)
        {
            match authority.critic_proof_rule {
                Some(CriticProofRule::IncrementContractFix) => {
                    let status = match self
                        .proof_command()
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
                    let command_a_status = match self
                        .proof_command()
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

                    let command_b_status = match self
                        .proof_command()
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
