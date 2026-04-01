use crate::domain::{HandoffDecision, ScholarOutput};

pub struct ScopePlanner;

impl ScopePlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn decide(&self, scholar_output: &ScholarOutput) -> HandoffDecision {
        if scholar_output.selected_task_scope.is_empty() {
            HandoffDecision::Stop
        } else {
            HandoffDecision::Proceed
        }
    }
}
