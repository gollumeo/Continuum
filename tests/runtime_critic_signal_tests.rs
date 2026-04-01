use continuum::application::critic_signal::CriticSignal;

#[test]
fn critic_signal_explicitly_models_minimal_runtime_review_outcomes() {
    let accepted = CriticSignal::Accepted;
    let revision_required = CriticSignal::RevisionRequired;
    let stop = CriticSignal::Stop;

    assert!(matches!(accepted, CriticSignal::Accepted));
    assert!(matches!(revision_required, CriticSignal::RevisionRequired));
    assert!(matches!(stop, CriticSignal::Stop));
}
