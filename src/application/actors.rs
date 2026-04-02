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

pub struct PostCriticPlanner;

impl Planner for PostCriticPlanner {
    fn decide(
        &mut self,
        _scholar_output: &ScholarOutput,
    ) -> crate::application::session_flow_decision::SessionFlowDecision {
        crate::application::session_flow_decision::SessionFlowDecision::Complete
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        critic_signal: CriticSignal,
    ) -> crate::application::session_flow_decision::SessionFlowDecision {
        match critic_signal {
            CriticSignal::Accepted => {
                crate::application::session_flow_decision::SessionFlowDecision::Complete
            }
            CriticSignal::RevisionRequired => {
                crate::application::session_flow_decision::SessionFlowDecision::Retry
            }
            CriticSignal::Stop => panic!("stop is not a local planner decision"),
        }
    }
}

pub trait Builder {
    fn run(&mut self, scholar_output: &ScholarOutput);
}

pub trait Critic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal;
}
