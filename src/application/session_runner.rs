use crate::application::actors::{Builder, BuilderRunReport, Critic, Planner, Scholar};
use crate::application::runtime_policy::{
    BuilderOutcomePolicy, CriticSignalPolicy, PostCriticDecisionPolicy, PreBuildPolicy,
    RetryDirective, RetryPolicy,
};
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
    last_builder_report: Option<BuilderRunReport>,
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
            last_builder_report: None,
        }
    }

    pub fn run(&mut self) -> Result<SessionSummary, FailureReport> {
        let scholar_output = self.scholar.run();
        let initial_decision = self.planner.decide(&scholar_output);

        PreBuildPolicy::admit_or_stop(initial_decision, &mut self.session)?;

        let mut retry_directive = self.run_attempt(&scholar_output)?;

        while retry_directive == RetryDirective::Retry {
            retry_directive = self.run_attempt(&scholar_output)?;
        }

        if retry_directive == RetryDirective::Complete {
            self.session.mark_completed().map_err(|_| FailureReport {
                final_session_status: *self.session.status(),
            })?;
        }

        Ok(SessionSummary {
            final_session_status: *self.session.status(),
        })
    }

    fn run_attempt(
        &mut self,
        scholar_output: &ScholarOutput,
    ) -> Result<RetryDirective, FailureReport> {
        let builder_report = self.builder.run(&scholar_output);
        self.last_builder_report = Some(builder_report.clone());

        BuilderOutcomePolicy::admit_or_stop(&builder_report, &mut self.session)?;

        let critic_signal = self.critic.run(&scholar_output);
        let post_critic_signal =
            CriticSignalPolicy::interpret_or_stop(critic_signal, &mut self.session)?;

        let final_decision = self
            .planner
            .decide_with_critic_signal(&scholar_output, post_critic_signal);

        let post_critic_decision =
            PostCriticDecisionPolicy::admit_or_stop(final_decision, &mut self.session)?;

        RetryPolicy::authorize_or_stop(
            post_critic_decision,
            &mut self.retry_budget_remaining,
            &mut self.session,
        )
    }

    pub fn session_status(&self) -> &SessionStatus {
        self.session.status()
    }

    pub fn last_builder_report(&self) -> Option<&BuilderRunReport> {
        self.last_builder_report.as_ref()
    }
}
