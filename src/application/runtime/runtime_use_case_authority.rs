#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuntimeUseCase {
    IncrementContractFix,
    IncrementContractFixAndZeroConfirm,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CriticProofRule {
    IncrementContractFix,
    IncrementContractFixAndZeroConfirm,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuntimeTerminalRule {
    IncrementContractConfirmationRetryExhausted,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RuntimeUseCaseAuthority {
    pub use_case: RuntimeUseCase,
    pub builder_allowed_file_scope: Option<&'static [&'static str]>,
    pub critic_proof_rule: Option<CriticProofRule>,
    pub terminal_rule: Option<RuntimeTerminalRule>,
}

const INCREMENT_CONTRACT_FIX_PROMPT: &str =
    "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.";

const INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_PROMPT: &str =
    "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.";

const INCREMENT_CONTRACT_FIX_AUTHORITY: RuntimeUseCaseAuthority = RuntimeUseCaseAuthority {
    use_case: RuntimeUseCase::IncrementContractFix,
    builder_allowed_file_scope: Some(&["src/lib.rs"]),
    critic_proof_rule: Some(CriticProofRule::IncrementContractFix),
    terminal_rule: None,
};

const INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_AUTHORITY: RuntimeUseCaseAuthority =
    RuntimeUseCaseAuthority {
        use_case: RuntimeUseCase::IncrementContractFixAndZeroConfirm,
        builder_allowed_file_scope: Some(&["src/lib.rs"]),
        critic_proof_rule: Some(CriticProofRule::IncrementContractFixAndZeroConfirm),
        terminal_rule: Some(RuntimeTerminalRule::IncrementContractConfirmationRetryExhausted),
    };

pub fn select_runtime_use_case_authority(prompt: &str) -> Option<RuntimeUseCaseAuthority> {
    if prompt == INCREMENT_CONTRACT_FIX_PROMPT {
        Some(INCREMENT_CONTRACT_FIX_AUTHORITY)
    } else if prompt == INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_PROMPT {
        Some(INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_AUTHORITY)
    } else {
        None
    }
}
