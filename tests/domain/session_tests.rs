use continuum::{Session, SessionError, SessionStatus};

#[test]
fn starts_session_in_active_status() {
    let session = Session::new();

    assert_eq!(session.status(), &SessionStatus::Active);
}

#[test]
fn marks_session_completed_as_terminal() {
    let mut session = Session::new();

    let result = session.mark_completed();

    assert_eq!(result, Ok(()));
    assert_eq!(session.status(), &SessionStatus::Completed);
}

#[test]
fn marks_session_stopped_as_terminal() {
    let mut session = Session::new();

    let result = session.mark_stopped();

    assert_eq!(result, Ok(()));
    assert_eq!(session.status(), &SessionStatus::Stopped);
}

#[test]
fn rejects_second_terminal_transition() {
    let mut session = Session::new();

    session.mark_completed().expect("first terminal transition should succeed");

    let result = session.mark_stopped();

    assert_eq!(result, Err(SessionError::AlreadyTerminal));
}
