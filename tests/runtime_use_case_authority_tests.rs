use continuum::{
    select_runtime_use_case_authority, CriticProofRule, RuntimeTerminalRule, RuntimeUseCase,
};

#[test]
fn selects_exact_single_increment_contract_fix_use_case_from_central_authority() {
    let authority = select_runtime_use_case_authority(
        "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
    )
    .expect("the exact proved single increment use case should be selected centrally");

    assert_eq!(authority.use_case, RuntimeUseCase::IncrementContractFix);
    assert_eq!(
        authority.critic_proof_rule,
        Some(CriticProofRule::IncrementContractFix),
    );
    assert_eq!(authority.terminal_rule, None);
}

#[test]
fn does_not_confuse_single_increment_fix_with_zero_confirmation_use_case() {
    let authority = select_runtime_use_case_authority(
        "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.",
    )
    .expect("the exact single increment use case should be selected centrally");

    assert_ne!(
        authority.use_case,
        RuntimeUseCase::IncrementContractFixAndZeroConfirm,
    );
}

#[test]
fn selects_exact_increment_contract_fix_and_zero_confirmation_use_case_from_central_authority() {
    let authority = select_runtime_use_case_authority(
        "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
    )
    .expect("the exact proved use case should be selected centrally");

    assert_eq!(authority.use_case, RuntimeUseCase::IncrementContractFixAndZeroConfirm);
    assert_eq!(
        authority.builder_allowed_file_scope,
        Some(["src/lib.rs"].as_slice()),
    );
    assert_eq!(
        authority.critic_proof_rule,
        Some(CriticProofRule::IncrementContractFixAndZeroConfirm),
    );
    assert_eq!(
        authority.terminal_rule,
        Some(RuntimeTerminalRule::IncrementContractConfirmationRetryExhausted),
    );
}

#[test]
fn does_not_select_zero_confirmation_use_case_for_single_increment_fix_prompt() {
    let authority = select_runtime_use_case_authority(
        "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.",
    );

    assert_ne!(
        authority.expect("the exact zero confirmation use case should be selected").use_case,
        RuntimeUseCase::IncrementContractFix,
    );
}
