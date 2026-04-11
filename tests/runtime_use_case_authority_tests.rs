use continuum::{
    select_runtime_use_case_authority, CriticProofRule, RuntimeTerminalRule, RuntimeUseCase,
};

#[test]
fn selects_exact_increment_contract_fix_and_zero_confirmation_use_case_from_central_authority() {
    let authority = select_runtime_use_case_authority(
        "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
    )
    .expect("the exact proved use case should be selected centrally");

    assert_eq!(authority.use_case, RuntimeUseCase::IncrementContractFixAndZeroConfirm);
    assert_eq!(authority.builder_allowed_file_scope, &["src/lib.rs"]);
    assert_eq!(
        authority.critic_proof_rule,
        CriticProofRule::IncrementContractFixAndZeroConfirm,
    );
    assert_eq!(
        authority.terminal_rule,
        RuntimeTerminalRule::IncrementContractConfirmationRetryExhausted,
    );
}

#[test]
fn does_not_select_zero_confirmation_use_case_for_single_increment_fix_prompt() {
    let authority = select_runtime_use_case_authority(
        "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
    );

    assert_eq!(authority, None);
}
