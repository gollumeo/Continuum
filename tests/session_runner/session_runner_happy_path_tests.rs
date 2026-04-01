use std::cell::RefCell;
use std::rc::Rc;

use continuum::application::actors::{Builder, Critic, Planner, Scholar};
use continuum::application::session_flow_decision::SessionFlowDecision;
use continuum::{
    AgentRole, ScholarOutput, SessionRunner, SessionStatus, SessionSummary, VerdictError,
};

struct RecordingScholar {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Scholar for RecordingScholar {
    fn run(&mut self) -> ScholarOutput {
        self.activations.borrow_mut().push(AgentRole::Scholar);

        ScholarOutput {
            mission_summary: "happy path mission".to_string(),
            selected_task_scope: "happy path mission".to_string(),
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

fn happy_path_runner(activations: Rc<RefCell<Vec<AgentRole>>>) -> SessionRunner {
    SessionRunner::new(
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
        Box::new(RecordingCritic { activations }),
    )
}

#[test]
fn runs_agents_in_strict_order_for_happy_path() {
    let activations = Rc::new(RefCell::new(Vec::new()));
    let mut runner = happy_path_runner(Rc::clone(&activations));

    runner.run().expect("happy path should complete without runner error");

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

#[test]
fn completes_session_after_approve_then_complete_decision() {
    let activations = Rc::new(RefCell::new(Vec::new()));
    let mut runner = happy_path_runner(Rc::clone(&activations));

    runner.run().expect("happy path should complete the session");

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
    assert_eq!(runner.session_status(), &SessionStatus::Completed);
}

#[test]
fn returns_session_summary_on_success() {
    let activations = Rc::new(RefCell::new(Vec::new()));
    let mut runner = happy_path_runner(Rc::clone(&activations));

    let summary = runner
        .run()
        .expect("happy path should return a session summary");

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
    assert_eq!(
        summary,
        SessionSummary {
            final_session_status: SessionStatus::Completed,
        }
    );
}
