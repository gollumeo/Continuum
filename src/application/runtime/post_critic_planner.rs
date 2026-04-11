use crate::application::actors::Planner;
use crate::application::runtime::post_critic_signal::PostCriticSignal;
use crate::application::runtime::session_flow_decision::SessionFlowDecision;
use crate::domain::ScholarOutput;

pub struct PostCriticPlanner;

impl Planner for PostCriticPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        SessionFlowDecision::Complete
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        match critic_signal {
            PostCriticSignal::Accepted => SessionFlowDecision::Complete,
            PostCriticSignal::RevisionRequired => SessionFlowDecision::Retry,
        }
    }
}
