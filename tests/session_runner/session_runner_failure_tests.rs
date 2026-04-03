use std::cell::RefCell;
use std::rc::Rc;

use continuum::application::actors::{Builder, BuilderRunReport, Critic, Planner, Scholar};
use continuum::application::critic_signal::CriticSignal;
use continuum::application::post_critic_signal::PostCriticSignal;
use continuum::application::session_flow_decision::SessionFlowDecision;
use continuum::{AgentRole, FailureReport, ScholarOutput, SessionRunner, SessionStatus};

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

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        _critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        self.activations.borrow_mut().push(AgentRole::Planner);
        self.decisions.remove(0)
    }
}

struct RecordingBuilder {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Builder for RecordingBuilder {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> BuilderRunReport {
        self.activations.borrow_mut().push(AgentRole::Builder);
        BuilderRunReport::completed()
    }
}

struct InvalidReviseCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Critic for InvalidReviseCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(AgentRole::Critic);

        CriticSignal::Stop
    }
}

struct RecordingCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Critic for RecordingCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(AgentRole::Critic);

        CriticSignal::Accepted
    }
}

struct RevisionAwarePlanner {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Planner for RevisionAwarePlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        self.activations.borrow_mut().push(AgentRole::Planner);
        SessionFlowDecision::Build
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        self.activations.borrow_mut().push(AgentRole::Planner);

        match critic_signal {
            PostCriticSignal::RevisionRequired => SessionFlowDecision::Retry,
            PostCriticSignal::Accepted => SessionFlowDecision::Complete,
        }
    }
}

struct RevisionRequestingCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Critic for RevisionRequestingCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(AgentRole::Critic);
        CriticSignal::RevisionRequired
    }
}

struct RepeatedRevisionCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
    signals: Vec<CriticSignal>,
}

impl Critic for RepeatedRevisionCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(AgentRole::Critic);
        self.signals.remove(0)
    }
}

#[test]
fn stops_when_initial_planner_decision_is_not_admitted_pre_build() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Complete],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingCritic {
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
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert_eq!(
        *activations.borrow(),
        vec![AgentRole::Scholar, AgentRole::Planner]
    );
}

#[test]
fn stops_when_initial_retry_decision_is_not_admitted_pre_build() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Retry],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingCritic {
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
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert_eq!(
        *activations.borrow(),
        vec![AgentRole::Scholar, AgentRole::Planner]
    );
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
fn stops_when_post_critic_planner_returns_build_in_planner_deciding() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Build, SessionFlowDecision::Build],
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingCritic {
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
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
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

#[test]
fn stops_when_revision_requires_retry_but_no_budget_is_available() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        0,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RevisionAwarePlanner {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RevisionRequestingCritic {
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

#[test]
fn stops_after_consuming_last_retry_budget_when_revision_is_requested_again() {
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
        Box::new(RepeatedRevisionCritic {
            activations: Rc::clone(&activations),
            signals: vec![CriticSignal::RevisionRequired, CriticSignal::RevisionRequired],
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
        2
    );
}

#[test]
fn stops_after_third_revision_request_when_retry_budget_of_two_is_fully_consumed() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        2,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RevisionAwarePlanner {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(RepeatedRevisionCritic {
            activations: Rc::clone(&activations),
            signals: vec![
                CriticSignal::RevisionRequired,
                CriticSignal::RevisionRequired,
                CriticSignal::RevisionRequired,
            ],
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
            AgentRole::Builder,
            AgentRole::Critic,
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
        3
    );
}
