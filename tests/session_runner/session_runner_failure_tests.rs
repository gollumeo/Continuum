use std::cell::RefCell;
use std::rc::Rc;

use continuum::{
    Builder, BuilderIssue, BuilderRunReport, BuilderScopeStatus, Critic, CriticSignal,
    FailureReport, Planner, PostCriticSignal, Scholar, ScholarOutput,
    SessionFlowDecision, SessionRunner, SessionStatus,
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
            mission_summary: "failure path mission".to_string(),
            selected_task_scope: "failure path mission".to_string(),
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

struct FailingBuilder {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Builder for FailingBuilder {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> BuilderRunReport {
        self.activations.borrow_mut().push(BUILDER);

        BuilderRunReport {
            issue: BuilderIssue::ProcessFailed,
            scope_status: BuilderScopeStatus::NotChecked,
            allowed_file_scope: Vec::new(),
            changed_files: Vec::new(),
            stdout: String::new(),
            stderr: "builder failed".to_string(),
        }
    }
}

struct StopSignalingCritic {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Critic for StopSignalingCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(CRITIC);

        CriticSignal::Stop
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

struct RevisionRequestingCritic {
    activations: Rc<RefCell<Vec<&'static str>>>,
}

impl Critic for RevisionRequestingCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(CRITIC);
        CriticSignal::RevisionRequired
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
            error: None,
        })
    );
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER]
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
            error: None,
        })
    );
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER]
    );
}

#[test]
fn stops_before_builder_when_initial_planner_refuses_underspecified_document_prompt() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        2,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::RefuseUnderspecifiedDocumentPrompt],
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
            error: Some(
                "refused to act on an underspecified document prompt; add an explicit allowed file scope",
            ),
        })
    );
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert!(runner.last_builder_report().is_none());
    assert_eq!(*activations.borrow(), vec![SCHOLAR, PLANNER]);
}

#[test]
fn stops_when_critic_returns_stop_signal() {
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
        Box::new(StopSignalingCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let result = runner.run();

    assert_eq!(
        result,
        Err(FailureReport {
            final_session_status: SessionStatus::Stopped,
            error: None,
        })
    );
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC]
    );
}

#[test]
fn stops_when_builder_report_is_not_successful() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingPlanner {
            activations: Rc::clone(&activations),
            decisions: vec![SessionFlowDecision::Build],
        }),
        Box::new(FailingBuilder {
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
            error: None,
        })
    );
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert_eq!(*activations.borrow(), vec![SCHOLAR, PLANNER, BUILDER]);
    assert_eq!(
        runner.last_builder_report().map(|report| &report.issue),
        Some(&BuilderIssue::ProcessFailed)
    );
}

#[test]
fn returns_failure_report_when_critic_stops_session() {
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
        Box::new(StopSignalingCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let failure_report = runner
        .run()
        .expect_err("critic stop signal should return a failure report");

    assert_eq!(
        failure_report,
        FailureReport {
            final_session_status: SessionStatus::Stopped,
            error: None,
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
            error: None,
        })
    );
    assert_eq!(runner.session_status(), &SessionStatus::Stopped);
    assert_eq!(
        *activations.borrow(),
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER]
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
        Box::new(StopSignalingCritic {
            activations: Rc::clone(&activations),
        }),
    );

    let result = runner.run();
    let recorded_activations = activations.borrow().clone();

    assert_eq!(
        result,
        Err(FailureReport {
            final_session_status: SessionStatus::Stopped,
            error: None,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == BUILDER)
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
            error: None,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == BUILDER)
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
            error: None,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == BUILDER)
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
            error: None,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![
            SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER, BUILDER, CRITIC, PLANNER,
        ]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == BUILDER)
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
            error: None,
        })
    );
    assert_eq!(
        recorded_activations,
        vec![
            SCHOLAR, PLANNER, BUILDER, CRITIC, PLANNER, BUILDER, CRITIC, PLANNER, BUILDER,
            CRITIC, PLANNER,
        ]
    );
    assert_eq!(
        recorded_activations
            .iter()
            .filter(|role| **role == BUILDER)
            .count(),
        3
    );
}
