use continuum::{MissionScholar, RawMission, ScholarOutput};

#[test]
fn transforms_raw_mission_into_scholar_output() {
    let mission = RawMission::new("introduce a structured scholar handoff");
    let scholar = MissionScholar::new();

    let output = scholar.transform(&mission);

    assert_eq!(
        output,
        ScholarOutput::new(
            "introduce a structured scholar handoff",
            "introduce a structured scholar handoff",
        )
    );
}
