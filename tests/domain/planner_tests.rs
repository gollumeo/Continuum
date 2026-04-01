use continuum::application::planner::ScopePlanner;
use continuum::{HandoffDecision, ScholarOutput};

#[test]
fn returns_proceed_decision_from_scholar_output() {
    let planner = ScopePlanner::new();
    let scholar_output = ScholarOutput::new(
        "summarized mission intent",
        "proposed atomic task scope",
    );

    let decision = planner.decide(&scholar_output);

    assert_eq!(decision, HandoffDecision::Proceed);
}

#[test]
fn returns_stop_decision_when_scholar_output_task_scope_is_empty() {
    let planner = ScopePlanner::new();
    let scholar_output = ScholarOutput::new("summarized mission intent", "");

    let decision = planner.decide(&scholar_output);

    assert_eq!(decision, HandoffDecision::Stop);
}
