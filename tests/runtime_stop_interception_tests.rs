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

        ScholarOutput::new("runtime stop mission", "runtime stop mission")
    }
}

struct StopRejectingPlanner {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Planner for StopRejectingPlanner {
    fn decide(&mut self, _scholar_output: &ScholarOutput) -> SessionFlowDecision {
        self.activations.borrow_mut().push(AgentRole::Planner);
        SessionFlowDecision::Build
    }

    fn decide_with_critic_signal(
        &mut self,
        _scholar_output: &ScholarOutput,
        _critic_signal: PostCriticSignal,
    ) -> SessionFlowDecision {
        panic!("post-critic planner must not be called after stop")
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

struct StopCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
}

impl Critic for StopCritic {
    fn run(&mut self, _scholar_output: &ScholarOutput) -> CriticSignal {
        self.activations.borrow_mut().push(AgentRole::Critic);
        CriticSignal::Stop
    }
}

#[test]
fn session_runner_intercepts_terminal_stop_before_post_critic_planner() {
    let activations = Rc::new(RefCell::new(Vec::new()));

    let failure_report = SessionRunner::new(
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(StopRejectingPlanner {
            activations: Rc::clone(&activations),
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
        }),
        Box::new(StopCritic {
            activations: Rc::clone(&activations),
        }),
    )
    .run()
    .expect_err("stop must terminate the runtime before the post-critic planner");

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
        ]
    );
}
