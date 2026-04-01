pub mod domain;
pub mod application;

pub use application::session_runner::{FailureReport, SessionRunner, SessionSummary};
pub use application::workflow::{AgentRole, StateMachine, WorkflowState};
pub use domain::{
    HandoffDecision,
    RawMission,
    Session,
    SessionError,
    SessionStatus,
    ScholarOutput,
    TaskContract,
    TaskContractError,
    Verdict,
    VerdictError,
};
