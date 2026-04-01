use crate::application::actors::{Builder, Critic, Planner, Scholar};
use crate::domain::*;

#[derive(Debug, PartialEq, Eq)]
pub struct SessionSummary {
    pub final_session_status: SessionStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FailureReport {
    pub final_session_status: SessionStatus,
}

pub struct SessionRunner {
    session: Session,
    retry_budget_remaining: u8,
    scholar: Box<dyn Scholar>,
    planner: Box<dyn Planner>,
    builder: Box<dyn Builder>,
    critic: Box<dyn Critic>,
}

impl SessionRunner {
    pub fn new(
        scholar: Box<dyn Scholar>,
        planner: Box<dyn Planner>,
        builder: Box<dyn Builder>,
        critic: Box<dyn Critic>,
    ) -> Self {
        Self::new_with_retry_budget(0, scholar, planner, builder, critic)
    }

    pub fn new_with_retry_budget(
        retry_budget_remaining: u8,
        scholar: Box<dyn Scholar>,
        planner: Box<dyn Planner>,
        builder: Box<dyn Builder>,
        critic: Box<dyn Critic>,
    ) -> Self {
        Self {
            session: Session::new(),
            retry_budget_remaining,
            scholar,
            planner,
            builder,
            critic,
        }
    }

    pub fn run(&mut self) -> Result<SessionSummary, FailureReport> {
        let scholar_output = self.scholar.run();
        let initial_decision = self.planner.decide(&scholar_output);

        if initial_decision == crate::application::session_flow_decision::SessionFlowDecision::Build {
            self.builder.run(&scholar_output);

            let critic_signal = self.critic.run(&scholar_output);

            if matches!(
                critic_signal,
                crate::application::critic_signal::CriticSignal::Stop
            ) {
                self.session.mark_stopped().ok();

                return Err(FailureReport {
                    final_session_status: self.session.status,
                });
            }

            let mut final_decision = self
                .planner
                .decide_with_critic_signal(&scholar_output, critic_signal);

            while final_decision
                == crate::application::session_flow_decision::SessionFlowDecision::Retry
            {
                if self.retry_budget_remaining == 0 {
                    self.session.mark_stopped().ok();

                    return Err(FailureReport {
                        final_session_status: self.session.status,
                    });
                }

                self.retry_budget_remaining -= 1;
                self.builder.run(&scholar_output);

                let retry_critic_signal = self.critic.run(&scholar_output);

                if matches!(
                    retry_critic_signal,
                    crate::application::critic_signal::CriticSignal::Stop
                ) {
                    self.session.mark_stopped().ok();

                    return Err(FailureReport {
                        final_session_status: self.session.status,
                    });
                }

                final_decision = self
                    .planner
                    .decide_with_critic_signal(&scholar_output, retry_critic_signal);
            }

            if final_decision
                == crate::application::session_flow_decision::SessionFlowDecision::Complete
            {
                self.session.mark_completed().map_err(|_| FailureReport {
                    final_session_status: self.session.status,
                })?;
            }
        }

        Ok(SessionSummary {
            final_session_status: self.session.status,
        })
    }

    pub fn session_status(&self) -> &SessionStatus {
        &self.session.status
    }
}
