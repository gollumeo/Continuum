use continuum::application::planner::ScopePlanner;
use continuum::application::scholar::MissionScholar;
use continuum::{HandoffDecision, RawMission, ScholarOutput};

#[test]
fn transforms_raw_mission_into_structured_scholar_output_before_planner_decides() {
    let mission = RawMission::new("introduce a structured scholar planner handoff");
    let scholar = MissionScholar::new();
    let planner = ScopePlanner::new();

    let scholar_output = scholar.transform(&mission);
    let decision = planner.decide(&scholar_output);

    assert_eq!(
        scholar_output,
        ScholarOutput::new(
            "introduce a structured scholar planner handoff",
            "introduce a structured scholar planner handoff",
        )
    );
    assert_eq!(decision, HandoffDecision::Proceed);
}
