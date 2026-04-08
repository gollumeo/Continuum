use crate::application::critic_signal::CriticSignal;
use crate::application::post_critic_signal::PostCriticSignal;
use crate::application::actors::BuilderRunReport;
use crate::application::session_flow_decision::SessionFlowDecision;
use crate::application::session_runner::FailureReport;
use crate::domain::Session;

pub struct BuilderOutcomePolicy;
pub struct PreBuildPolicy;
pub struct CriticSignalPolicy;
pub struct PostCriticDecisionPolicy;
pub struct RetryPolicy;

const UNDERSPECIFIED_DOCUMENT_PROMPT_REFUSAL: &str =
    "refused to act on an underspecified document prompt; add an explicit allowed file scope";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PostCriticDecision {
    Retry,
    Complete,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RetryDirective {
    Retry,
    Complete,
}

fn stop_session(session: &mut Session) -> FailureReport {
    stop_session_with_error(session, None)
}

fn stop_session_with_error(session: &mut Session, error: Option<&'static str>) -> FailureReport {
    session.mark_stopped().ok();

    FailureReport {
        final_session_status: *session.status(),
        error,
    }
}

impl BuilderOutcomePolicy {
    pub fn admit_or_stop(
        builder_report: &BuilderRunReport,
        session: &mut Session,
    ) -> Result<(), FailureReport> {
        if builder_report.is_success() {
            Ok(())
        } else {
            Err(stop_session(session))
        }
    }
}

impl PreBuildPolicy {
    pub fn admit_or_stop(
        decision: SessionFlowDecision,
        session: &mut Session,
    ) -> Result<(), FailureReport> {
        match decision {
            SessionFlowDecision::Build => Ok(()),
            SessionFlowDecision::RefuseUnderspecifiedDocumentPrompt => Err(
                stop_session_with_error(session, Some(UNDERSPECIFIED_DOCUMENT_PROMPT_REFUSAL)),
            ),
            SessionFlowDecision::Retry | SessionFlowDecision::Complete => Err(stop_session(session)),
        }
    }
}

impl CriticSignalPolicy {
    pub fn interpret_or_stop(
        critic_signal: CriticSignal,
        session: &mut Session,
    ) -> Result<PostCriticSignal, FailureReport> {
        match critic_signal {
            CriticSignal::Stop => Err(stop_session(session)),
            CriticSignal::Accepted => Ok(PostCriticSignal::Accepted),
            CriticSignal::RevisionRequired => Ok(PostCriticSignal::RevisionRequired),
        }
    }
}

impl PostCriticDecisionPolicy {
    pub fn admit_or_stop(
        decision: SessionFlowDecision,
        session: &mut Session,
    ) -> Result<PostCriticDecision, FailureReport> {
        match decision {
            SessionFlowDecision::Retry => Ok(PostCriticDecision::Retry),
            SessionFlowDecision::Complete => Ok(PostCriticDecision::Complete),
            SessionFlowDecision::Build
            | SessionFlowDecision::RefuseUnderspecifiedDocumentPrompt => Err(stop_session(session)),
        }
    }
}

impl RetryPolicy {
    pub fn authorize_or_stop(
        decision: PostCriticDecision,
        retry_budget_remaining: &mut u8,
        session: &mut Session,
    ) -> Result<RetryDirective, FailureReport> {
        match decision {
            PostCriticDecision::Complete => Ok(RetryDirective::Complete),
            PostCriticDecision::Retry if *retry_budget_remaining == 0 => Err(stop_session(session)),
            PostCriticDecision::Retry => {
                *retry_budget_remaining -= 1;
                Ok(RetryDirective::Retry)
            }
        }
    }
}
