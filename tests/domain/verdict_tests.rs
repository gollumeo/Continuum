use continuum::Verdict;

#[test]
fn builds_revise_verdict_with_required_changes() {
    let verdict = Verdict::revise(vec!["clarify acceptance criteria".to_string()])
        .expect("revise verdict should accept at least one required change");

    assert_eq!(
        verdict.required_changes,
        vec!["clarify acceptance criteria"]
    );
}

#[test]
fn rejects_revise_verdict_without_required_changes() {
    let result = Verdict::revise(Vec::new());

    assert!(result.is_err());
}
