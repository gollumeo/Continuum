#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuntimeUseCase {
    IncrementContractFixAndZeroConfirm,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CriticProofRule {
    IncrementContractFixAndZeroConfirm,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuntimeTerminalRule {
    IncrementContractConfirmationRetryExhausted,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct RuntimeUseCaseAuthority {
    pub use_case: RuntimeUseCase,
    pub builder_allowed_file_scope: &'static [&'static str],
    pub critic_proof_rule: CriticProofRule,
    pub terminal_rule: RuntimeTerminalRule,
}

const INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_PROMPT: &str =
    "Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.";

const INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_AUTHORITY: RuntimeUseCaseAuthority =
    RuntimeUseCaseAuthority {
        use_case: RuntimeUseCase::IncrementContractFixAndZeroConfirm,
        builder_allowed_file_scope: &["src/lib.rs"],
        critic_proof_rule: CriticProofRule::IncrementContractFixAndZeroConfirm,
        terminal_rule: RuntimeTerminalRule::IncrementContractConfirmationRetryExhausted,
    };

pub fn select_runtime_use_case_authority(prompt: &str) -> Option<RuntimeUseCaseAuthority> {
    if prompt == INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_PROMPT {
        Some(INCREMENT_CONTRACT_FIX_AND_ZERO_CONFIRM_AUTHORITY)
    } else {
        None
    }
}
