use std::cell::RefCell;
use std::rc::Rc;

use continuum::{
    Builder, BuilderRunReport, Critic, CriticSignal, FailureReport, Planner, PostCriticSignal,
    Scholar, ScholarOutput, SessionFlowDecision, SessionRunner, SessionStatus,
};

const SCHOLAR: &str = "scholar";
const PLANNER: &str = "planner";
const BUILDER: &str = "builder";
const CRITIC: &str = "critic";

struct RecordingScholar {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Scholar for RecordingScholar {
    fn run(&mut self) -> ScholarOutput {
        self.activations.borrow_mut().push(SCHOLAR);

        ScholarOutput {
            mission_summary: "e2e stop mission".to_string(),
            selected_task_scope: "e2e stop mission".to_string(),
        }
    }
}

struct RecordingPlanner {
    activations: Rc<RefCell<Vec<&'static str>>>,
    decisions: Vec<SessionFlowDecision>,
}

impl Planner for RecordingPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        self.activations.borrow_mut().push(PLANNER);
        self.decisions.remove(0)
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        _critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        self.activations.borrow_mut().push(PLANNER);
        self.decisions.remove(0)
    }
}

struct RecordingBuilder {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Builder for RecordingBuilder {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> BuilderRunReport {
        self.activations.borrow_mut().push(BUILDER);
        BuilderRunReport::completed()
    }
}

struct RecordingCritic {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Critic for RecordingCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(CRITIC);

        CriticSignal::Accepted
    }
}

struct RepeatedRevisionCritic {
    activations: Rc<RefCell<Vec<&'static str>>>,
    signals: Vec<CriticSignal>,
}

impl Critic for RepeatedRevisionCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(CRITIC);
        self.signals.remove(0)
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
            error: None,
        }
    );
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER]
    );
}

#[test]
fn stops_session_when_runtime_revision_is_requested_again_without_retry_budget() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        1,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![
                SessionFlowDecision::Build,
                SessionFlowDecision::Retry,
                SessionFlowDecision::Retry,
            ],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RepeatedRevisionCritic {
            activations: Rc::clone(&activations),
            signals: vec![
                CriticSignal::RevisionRequired,
                CriticSignal::RevisionRequired,
            ],
        }),
    );

    let failure_report = runner
        .run()
        .expect_err("second runtime revision without budget should stop the session");

    assert_eq!(
        failure_report,
        FailureReport {
            final_session_status: SessionStatus::Stopped,
            error: None,
        }
    );
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER, BUILDER, CRITIC, PLANNER,]
    );
    assert_eq!(
        activations
            .borrow()
            .iter()
            .filter(|role| **role == BUILDER)
            .count(),
        2
    );
}
