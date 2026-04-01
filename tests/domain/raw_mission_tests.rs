use continuum::RawMission;

#[test]
fn builds_raw_mission_with_content() {
    let mission = RawMission::new("add a structured scholar handoff");

    assert_eq!(mission.content, "add a structured scholar handoff");
}
