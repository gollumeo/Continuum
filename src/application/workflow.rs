#[derive(Debug, PartialEq, Eq)]
pub enum WorkflowState {
    Initialized,
    MissionAnalyzed,
    TaskSelected,
    SnapshotCaptured,
    BuilderRunning,
    CriticReviewing,
    PlannerDeciding,
    Completed,
    Stopped,
}

pub struct StateMachine;

impl StateMachine {
    pub fn allows_transition(from: WorkflowState, to: WorkflowState) -> bool {
        matches!(
            (from, to),
            (WorkflowState::Initialized, WorkflowState::MissionAnalyzed)
                | (WorkflowState::MissionAnalyzed, WorkflowState::TaskSelected)
                | (WorkflowState::TaskSelected, WorkflowState::SnapshotCaptured)
                | (WorkflowState::SnapshotCaptured, WorkflowState::BuilderRunning)
                | (WorkflowState::BuilderRunning, WorkflowState::CriticReviewing)
                | (WorkflowState::CriticReviewing, WorkflowState::PlannerDeciding)
                | (WorkflowState::PlannerDeciding, WorkflowState::BuilderRunning)
                | (WorkflowState::PlannerDeciding, WorkflowState::Completed)
                | (WorkflowState::PlannerDeciding, WorkflowState::Stopped)
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AgentRole {
    Scholar,
    Planner,
    Builder,
    Critic,
}
