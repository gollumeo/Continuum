use crate::domain::{RawMission, ScholarOutput};

pub struct MissionScholar;

impl MissionScholar {
    pub fn new() -> Self {
        Self
    }

    pub fn transform(&self, mission: &RawMission) -> ScholarOutput {
        ScholarOutput::new(&mission.content, &mission.content)
    }
}
