use crate::application::critic_signal::CriticSignal;
use crate::domain::ScholarOutput;

pub trait Scholar {
    fn run(&mut self) -> ScholarOutput;
}

pub trait Planner {
    fn decide(
        &mut self,
        scholar_output: &ScholarOutput,
    ) -> crate::application::session_flow_decision::SessionFlowDecision;

    fn decide_with_critic_signal(
        &mut self,
        scholar_output: &ScholarOutput,
        critic_signal: CriticSignal,
    ) -> crate::application::session_flow_decision::SessionFlowDecision;
}

pub trait Builder {
    fn run(&mut self, scholar_output: &ScholarOutput);
}

pub trait Critic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal;
}
