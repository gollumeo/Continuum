use continuum::{Critic, CriticSignal, ScholarOutput};

#[test]
fn critic_signal_explicitly_models_minimal_runtime_review_outcomes() {
    let accepted = CriticSignal::Accepted;
    let revision_required = CriticSignal::RevisionRequired;
    let stop = CriticSignal::Stop;

    assert!(matches!(accepted, CriticSignal::Accepted));
    assert!(matches!(revision_required, CriticSignal::RevisionRequired));
    assert!(matches!(stop, CriticSignal::Stop));
}

fn critic_run_signal(
    critic: &mut dyn Critic,
    scholar_output: &ScholarOutput,
) -> CriticSignal {
    critic.run(scholar_output)
}

#[test]
fn critic_contract_explicitly_returns_runtime_signal() {
    let _ = critic_run_signal;
}
