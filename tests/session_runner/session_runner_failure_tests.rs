use std::cell::RefCell;
use std::rc::Rc;

use continuum::application::actors::{Builder, Critic, Planner, Scholar};
use continuum::application::session_flow_decision::SessionFlowDecision;
use continuum::{
    AgentRole, ScholarOutput, FailureReport, SessionRunner, SessionStatus, Verdict, VerdictError,
};

struct RecordingScholar {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Scholar for RecordingScholar {
    fn run(&mut self) -> ScholarOutput {
        self.activations.borrow_mut().push(AgentRole::Scholar);

        ScholarOutput {
            mission_summary: "failure path mission".to_string(),
            selected_task_scope: "failure path mission".to_string(),
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

struct InvalidReviseCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Critic for InvalidReviseCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> Result<(), VerdictError> {
        self.activations.borrow_mut().push(AgentRole::Critic);

        Verdict::revise(Vec::new()).map(|_| ())
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
fn stops_when_critic_returns_invalid_revise_verdict() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Build, SessionFlowDecision::Complete],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(InvalidReviseCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let result = runner.run();

    assert_eq!(
        result,
        Err(FailureReport {
            final_session_status: SessionStatus::Stopped,
        })
    );
    assert_eq!(
        *activations.borrow(),
        vec![
            AgentRole::Scholar,
            AgentRole::Planner,
            AgentRole::Builder,
            AgentRole::Critic,
        ]
    );
}

#[test]
fn returns_failure_report_on_hard_stop() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Build, SessionFlowDecision::Complete],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(InvalidReviseCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let failure_report = runner
        .run()
        .expect_err("invalid revise verdict should return a failure report");

    assert_eq!(
        failure_report,
        FailureReport {
            final_session_status: SessionStatus::Stopped,
        }
    );
}

#[test]
fn does_not_call_builder_again_after_terminal_stop() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Build, SessionFlowDecision::Complete],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(InvalidReviseCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let result = runner.run();
    let recorded_activations = activations.borrow().clone();

    assert_eq!(
        result,
        Err(FailureReport {
            final_session_status: SessionStatus::Stopped,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![
            AgentRole::Scholar,
            AgentRole::Planner,
            AgentRole::Builder,
            AgentRole::Critic,
        ]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == AgentRole::Builder)
            .count(),
        1
    );
}

#[test]
fn stops_when_planner_requests_retry_without_budget() {
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

    let result = runner.run();
    let recorded_activations = activations.borrow().clone();

    assert_eq!(
        result,
        Err(FailureReport {
            final_session_status: SessionStatus::Stopped,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![
            AgentRole::Scholar,
            AgentRole::Planner,
            AgentRole::Builder,
            AgentRole::Critic,
            AgentRole::Planner,
        ]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == AgentRole::Builder)
            .count(),
        1
    );
}
