#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuilderIssue {
    Completed,
    PreconditionFailed,
    LaunchFailed,
    ProcessFailed,
    ScopeViolated,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuilderScopeStatus {
    NotChecked,
    WithinScope,
    Violated,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BuilderRunReport {
    pub issue: BuilderIssue,
    pub scope_status: BuilderScopeStatus,
    pub allowed_file_scope: Vec<String>,
    pub changed_files: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

impl BuilderRunReport {
    pub fn completed() -> Self {
        Self {
            issue: BuilderIssue::Completed,
            scope_status: BuilderScopeStatus::WithinScope,
            allowed_file_scope: Vec::new(),
            changed_files: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.issue == BuilderIssue::Completed
    }
}
