pub struct RawMission {
    pub content: String,
}

impl RawMission {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
}
