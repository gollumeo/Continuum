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
            mission_summary: "propagated mission context".to_string(),
            selected_task_scope: "propagated mission context".to_string(),
        }
    }
}

struct DataAwarePlanner {
    activations: Rc<RefCell<Vec<AgentRole>>>,
    observed_data: Rc<RefCell<Vec<String>>>,
    call_count: u8,
}

impl Planner for DataAwarePlanner {
    fn decide(&mut self, scholar_output: &ScholarOutput) -> SessionFlowDecision {
        self.activations.borrow_mut().push(AgentRole::Planner);
        self.observed_data
            .borrow_mut()
            .push(format!("planner:{}", scholar_output.mission_summary));

        self.call_count += 1;

        if self.call_count == 1
            && scholar_output.mission_summary == "propagated mission context"
        {
            SessionFlowDecision::Build
        } else {
            SessionFlowDecision::Complete
        }
    }
}

struct RecordingBuilder {
    activations: Rc<RefCell<Vec<AgentRole>>>,
    observed_data: Rc<RefCell<Vec<String>>>,
}

impl Builder for RecordingBuilder {
    fn run(&mut self, scholar_output: &ScholarOutput) {
        self.activations.borrow_mut().push(AgentRole::Builder);
        self.observed_data
            .borrow_mut()
            .push(format!("builder:{}", scholar_output.mission_summary));
    }
}

struct RecordingCritic {
    activations: Rc<RefCell<Vec<AgentRole>>>,
    observed_data: Rc<RefCell<Vec<String>>>,
}

impl Critic for RecordingCritic {
    fn run(&mut self, scholar_output: &ScholarOutput) -> Result<(), VerdictError> {
        self.activations.borrow_mut().push(AgentRole::Critic);
        self.observed_data
            .borrow_mut()
            .push(format!("critic:{}", scholar_output.mission_summary));

        Ok(())
    }
}

#[test]
fn propagates_scholar_output_through_session() {
    let activations = Rc::new(RefCell::new(Vec::new()));
    let observed_data = Rc::new(RefCell::new(Vec::new()));

    let mut runner = SessionRunner::new_with_retry_budget(
        0,
        Box::new(RecordingScholar {
            activations: Rc::clone(&activations),
        }),
        Box::new(DataAwarePlanner {
            activations: Rc::clone(&activations),
            observed_data: Rc::clone(&observed_data),
            call_count: 0,
        }),
        Box::new(RecordingBuilder {
            activations: Rc::clone(&activations),
            observed_data: Rc::clone(&observed_data),
        }),
        Box::new(RecordingCritic {
            activations: Rc::clone(&activations),
            observed_data: Rc::clone(&observed_data),
        }),
    );

    let summary = runner
        .run()
        .expect("propagated scholar output should complete the session");

    assert_eq!(
        summary,
        SessionSummary {
            final_session_status: SessionStatus::Completed,
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
    assert_eq!(
        *observed_data.borrow(),
        vec![
            "planner:propagated mission context".to_string(),
            "builder:propagated mission context".to_string(),
            "critic:propagated mission context".to_string(),
            "planner:propagated mission context".to_string(),
        ]
    );
}
