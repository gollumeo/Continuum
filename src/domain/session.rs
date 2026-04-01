#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SessionStatus {
    Active,
    Completed,
    Stopped,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionError {
    AlreadyTerminal,
}

pub struct Session {
    pub status: SessionStatus,
}

impl Session {
    pub fn new() -> Self {
        Self {
            status: SessionStatus::Active,
        }
    }

    pub fn mark_completed(&mut self) -> Result<(), SessionError> {
        if self.status == SessionStatus::Completed || self.status == SessionStatus::Stopped {
            return Err(SessionError::AlreadyTerminal);
        }

        self.status = SessionStatus::Completed;

        Ok(())
    }

    pub fn mark_stopped(&mut self) -> Result<(), SessionError> {
        if self.status == SessionStatus::Completed || self.status == SessionStatus::Stopped {
            return Err(SessionError::AlreadyTerminal);
        }

        self.status = SessionStatus::Stopped;

        Ok(())
    }
}
