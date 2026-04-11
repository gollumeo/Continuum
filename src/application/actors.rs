use crate::application::runtime::critic_signal::CriticSignal;
use crate::application::runtime::post_critic_signal::PostCriticSignal;
use crate::domain::ScholarOutput;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuilderIssue {
    Completed,
    PreconditionFailed,
    LaunchFailed,
    ProcessFailed,
    ScopeViolated,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuilderScopeStatus {
    NotChecked,
    WithinScope,
    Violated,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuilderRunReport {
    pub issue: BuilderIssue,
    pub scope_status: BuilderScopeStatus,
    pub allowed_file_scope: Vec<String>,
    pub changed_files: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

impl BuilderRunReport {
    pub fn completed() -> Self {
        Self {
            issue: BuilderIssue::Completed,
            scope_status: BuilderScopeStatus::WithinScope,
            allowed_file_scope: Vec::new(),
            changed_files: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.issue == BuilderIssue::Completed
    }
}

pub trait Scholar {
    fn run(&mut self) -> ScholarOutput;
}

pub trait Planner {
    fn decide(
        &mut self,
        scholar_output: &ScholarOutput,
    ) -> crate::application::runtime::session_flow_decision::SessionFlowDecision;

    fn decide_with_critic_signal(
        &mut self,
        scholar_output: &ScholarOutput,
        critic_signal: PostCriticSignal,
    ) -> crate::application::runtime::session_flow_decision::SessionFlowDecision;
}

pub trait Builder {
    fn run(&mut self, scholar_output: &ScholarOutput) -> BuilderRunReport;
}

pub trait Critic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal;
}
