use crate::application::critic_signal::CriticSignal;
use crate::application::post_critic_signal::PostCriticSignal;
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
        critic_signal: PostCriticSignal,
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
        critic_signal: PostCriticSignal,
    ) -> crate::application::session_flow_decision::SessionFlowDecision {
        match critic_signal {
            PostCriticSignal::Accepted => {
                crate::application::session_flow_decision::SessionFlowDecision::Complete
            }
            PostCriticSignal::RevisionRequired => {
                crate::application::session_flow_decision::SessionFlowDecision::Retry
            }
        }
    }
}

pub trait Builder {
    fn run(&mut self, scholar_output: &ScholarOutput);
}

pub trait Critic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> CriticSignal;
}
