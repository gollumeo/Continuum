pub struct Verdict {
    pub required_changes: Vec<String>,
}

#[derive(Debug)]
pub enum VerdictError {
    MissingRequiredChanges,
}

impl Verdict {
    pub fn revise(required_changes: Vec<String>) -> Result<Self, VerdictError> {
        if required_changes.is_empty() {
            return Err(VerdictError::MissingRequiredChanges);
        }

        Ok(Self { required_changes })
    }
}
