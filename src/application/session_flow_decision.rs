#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SessionFlowDecision {
    Build,
    Retry,
    Complete,
}
