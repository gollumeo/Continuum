use std::cell::RefCell;
use std::rc::Rc;

use continuum::application::actors::{Builder, Critic, Planner, Scholar};
use continuum::application::session_flow_decision::SessionFlowDecision;
use continuum::{
    AgentRole, FailureReport, ScholarOutput, SessionRunner, SessionStatus, VerdictError,
};

struct RecordingScholar {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Scholar for RecordingScholar {
    fn run(&mut self) -> ScholarOutput {
        self.activations.borrow_mut().push(AgentRole::Scholar);

        ScholarOutput {
            mission_summary: "e2e stop mission".to_string(),
            selected_task_scope: "e2e stop mission".to_string(),
        }
    }
}

struct RecordingPlanner {
    activations: Rc<RefCell<Vec<AgentRole>>>,
    decisions: Vec<SessionFlowDecision>,
}

impl Planner for RecordingPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        self.activations.borrow_mut().push(AgentRole::Planner);
        self.decisions.remove(0)
    }
}

struct RecordingBuilder {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Builder for RecordingBuilder {
    fn run(&mut self, _scholar_output: &ScholarOutput) {
        self.activations.borrow_mut().push(AgentRole::Builder);
    }
}

struct RecordingCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Critic for RecordingCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> Result<(), VerdictError> {
        self.activations.borrow_mut().push(AgentRole::Critic);

        Ok(())
    }
}

#[test]
fn stops_when_budget_is_exhausted() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        0,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Build, SessionFlowDecision::Retry],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let failure_report = runner
        .run()
        .expect_err("retry without remaining budget should stop the session");

    assert_eq!(
        failure_report,
        FailureReport {
            final_session_status: SessionStatus::Stopped,
        }
    );
    assert_eq!(
        *activations.borrow(),
        vec![
            AgentRole::Scholar,
            AgentRole::Planner,
            AgentRole::Builder,
            AgentRole::Critic,
            AgentRole::Planner,
        ]
    );
}
