use crate::application::runtime::post_critic_signal::PostCriticSignal;
use crate::application::runtime::builder_run_report::BuilderRunReport;
use crate::domain::ScholarOutput;

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
