use continuum::TaskContract;

#[test]
fn builds_task_contract_with_budget_2() {
    let contract = TaskContract::new(2).expect("budget 2 should be valid");

    assert_eq!(contract.iteration_budget, 2);
}

#[test]
fn builds_task_contract_with_budget_3() {
    let contract = TaskContract::new(3).expect("budget 3 should be valid");

    assert_eq!(contract.iteration_budget, 3);
}

#[test]
fn rejects_task_contract_with_budget_below_2() {
    let result = TaskContract::new(1);

    assert!(result.is_err());
}

#[test]
fn rejects_task_contract_with_budget_above_3() {
    let result = TaskContract::new(4);

    assert!(result.is_err());
}
