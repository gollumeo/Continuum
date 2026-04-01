use continuum::{StateMachine, WorkflowState};

#[test]
fn allows_initialized_to_mission_analyzed() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::Initialized,
        WorkflowState::MissionAnalyzed,
    );

    assert!(allowed);
}

#[test]
fn allows_mission_analyzed_to_task_selected() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::MissionAnalyzed,
        WorkflowState::TaskSelected,
    );

    assert!(allowed);
}

#[test]
fn allows_task_selected_to_snapshot_captured() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::TaskSelected,
        WorkflowState::SnapshotCaptured,
    );

    assert!(allowed);
}

#[test]
fn allows_snapshot_captured_to_builder_running() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::SnapshotCaptured,
        WorkflowState::BuilderRunning,
    );

    assert!(allowed);
}

#[test]
fn allows_builder_running_to_critic_reviewing() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::BuilderRunning,
        WorkflowState::CriticReviewing,
    );

    assert!(allowed);
}

#[test]
fn allows_critic_reviewing_to_planner_deciding() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::CriticReviewing,
        WorkflowState::PlannerDeciding,
    );

    assert!(allowed);
}

#[test]
fn allows_planner_deciding_to_builder_running() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::PlannerDeciding,
        WorkflowState::BuilderRunning,
    );

    assert!(allowed);
}

#[test]
fn allows_planner_deciding_to_completed() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::PlannerDeciding,
        WorkflowState::Completed,
    );

    assert!(allowed);
}

#[test]
fn allows_planner_deciding_to_stopped() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::PlannerDeciding,
        WorkflowState::Stopped,
    );

    assert!(allowed);
}
