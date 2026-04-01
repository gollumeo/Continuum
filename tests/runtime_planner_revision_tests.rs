use continuum::application::actors::Planner;
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
