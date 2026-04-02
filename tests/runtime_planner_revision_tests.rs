use continuum::application::actors::{Planner, PostCriticPlanner};
use continuum::application::critic_signal::CriticSignal;
use continuum::application::session_flow_decision::SessionFlowDecision;
use continuum::ScholarOutput;

struct CompletingPlanner;

impl Planner for CompletingPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        SessionFlowDecision::Complete
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        critic_signal: CriticSignal,
    ) -> SessionFlowDecision {
        match critic_signal {
            CriticSignal::RevisionRequired => SessionFlowDecision::Retry,
            CriticSignal::Accepted | CriticSignal::Stop => SessionFlowDecision::Complete,
        }
    }
}

fn planner_retry_after_revision_signal(
    planner: &mut dyn Planner,
    scholar_output: &ScholarOutput,
) -> SessionFlowDecision {
    planner.decide_with_critic_signal(scholar_output, CriticSignal::RevisionRequired)
}

#[test]
fn planner_contract_can_exploit_revision_signal_for_runtime_orchestration() {
    let _ = planner_retry_after_revision_signal;
}

#[test]
fn planner_returns_retry_when_revision_signal_is_explicit() {
    let scholar_output = ScholarOutput::new("mission summary", "task scope");
    let mut planner = CompletingPlanner;

    let decision = planner_retry_after_revision_signal(&mut planner, &scholar_output);

    assert_eq!(decision, SessionFlowDecision::Retry);
}

#[test]
fn production_post_critic_runtime_semantics_are_explicit_in_application() {
    let scholar_output = ScholarOutput::new("mission summary", "task scope");
    let mut planner = PostCriticPlanner;

    let accepted_decision =
        planner.decide_with_critic_signal(&scholar_output, CriticSignal::Accepted);
    let revision_decision =
        planner.decide_with_critic_signal(&scholar_output, CriticSignal::RevisionRequired);

    assert_eq!(accepted_decision, SessionFlowDecision::Complete);
    assert_eq!(revision_decision, SessionFlowDecision::Retry);
}

#[test]
#[should_panic]
fn production_post_critic_runtime_semantics_reject_stop_as_a_local_planner_decision() {
    let scholar_output = ScholarOutput::new("mission summary", "task scope");
    let mut planner = PostCriticPlanner;

    let _ = planner.decide_with_critic_signal(&scholar_output, CriticSignal::Stop);
}
