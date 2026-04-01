use crate::domain::{ScholarOutput, VerdictError};

pub trait Scholar {
    fn run(&mut self) -> ScholarOutput;
}

pub trait Planner {
    fn decide(
        &mut self,
        scholar_output: &ScholarOutput,
    ) -> crate::application::session_flow_decision::SessionFlowDecision;
}

pub trait Builder {
    fn run(&mut self, scholar_output: &ScholarOutput);
}

pub trait Critic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> Result<(), VerdictError>;
}
