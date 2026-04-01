use continuum::{StateMachine, WorkflowState};

#[test]
fn rejects_initialized_to_task_selected() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::Initialized,
        WorkflowState::TaskSelected,
    );

    assert!(!allowed);
}

#[test]
fn rejects_builder_running_to_completed() {
    let allowed = StateMachine::allows_transition(
        WorkflowState::BuilderRunning,
        WorkflowState::Completed,
    );

    assert!(!allowed);
}
