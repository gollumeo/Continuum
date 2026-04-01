#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScholarOutput {
    pub mission_summary: String,
    pub selected_task_scope: String,
}

impl ScholarOutput {
    pub fn new(mission_summary: &str, selected_task_scope: &str) -> Self {
        Self {
            mission_summary: mission_summary.to_string(),
            selected_task_scope: selected_task_scope.to_string(),
        }
    }
}
