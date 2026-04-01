use continuum::HandoffDecision;

#[test]
fn builds_proceed_planner_decision() {
    let decision = HandoffDecision::Proceed;

    assert_eq!(decision, HandoffDecision::Proceed);
}
