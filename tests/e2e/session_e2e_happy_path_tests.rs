use std::cell::RefCell;
use std::rc::Rc;

use continuum::{
    Builder, BuilderRunReport, Critic, CriticSignal, Planner, PostCriticSignal, Scholar,
    ScholarOutput, SessionFlowDecision, SessionRunner, SessionStatus, SessionSummary,
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
            mission_summary: "e2e happy mission".to_string(),
            selected_task_scope: "e2e happy mission".to_string(),
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

struct RevisionAwarePlanner {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Planner for RevisionAwarePlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        self.activations.borrow_mut().push(PLANNER);
        SessionFlowDecision::Build
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        self.activations.borrow_mut().push(PLANNER);

        match critic_signal {
            PostCriticSignal::RevisionRequired => SessionFlowDecision::Retry,
            PostCriticSignal::Accepted => SessionFlowDecision::Complete,
        }
    }
}

struct RevisionThenAcceptedCritic {
    activations: Rc<RefCell<Vec<&'static str>>>,
    signals: Vec<CriticSignal>,
}

impl Critic for RevisionThenAcceptedCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(CRITIC);
        self.signals.remove(0)
    }
}

#[test]
fn completes_single_iteration_session_end_to_end() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        0,
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
        Box::new(RecordingCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let summary = runner
        .run()
        .expect("single iteration happy path should complete successfully");

    assert_eq!(
        summary,
        SessionSummary {
            final_session_status: SessionStatus::Completed,
        }
    );
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER]
    );
}

#[test]
fn completes_session_after_one_runtime_revision_then_explicit_complete() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        1,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RevisionAwarePlanner {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RevisionThenAcceptedCritic {
            activations: Rc::clone(&activations),
            signals: vec![CriticSignal::RevisionRequired, CriticSignal::Accepted],
        }),
    );

    let summary = runner
        .run()
        .expect("runtime session should complete after one bounded revision");

    assert_eq!(
        summary,
        SessionSummary {
            final_session_status: SessionStatus::Completed,
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
