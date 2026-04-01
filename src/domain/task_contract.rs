pub struct TaskContract {
    pub iteration_budget: u8,
}

#[derive(Debug)]
pub enum TaskContractError {
    IterationBudgetBelowMinimum,
    IterationBudgetAboveMaximum,
}

impl TaskContract {
    pub fn new(iteration_budget: u8) -> Result<Self, TaskContractError> {
        if iteration_budget < 2 {
            return Err(TaskContractError::IterationBudgetBelowMinimum);
        }

        if iteration_budget > 3 {
            return Err(TaskContractError::IterationBudgetAboveMaximum);
        }

        Ok(Self { iteration_budget })
    }
}
