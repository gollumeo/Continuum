# Slice 1 Implementation Mandate

## 1. Objective

Prove that Continuum already works as a disciplined runtime through a complete in-memory control loop.

## 2. Exact Scope

### Included

- one in-memory session
- one mission input
- one selected task
- one `task_contract`
- strict state machine
- `Scholar`, `Planner`, `Builder`, `Critic` as test doubles only
- in-memory event log
- in-memory `SessionSummary` or `FailureReport`
- domain invariants
- `SessionRunner` orchestration
- end-to-end tests fully in memory

### Excluded

- SQLite
- filesystem
- real artifacts
- real snapshots
- real diffs
- shell commands
- real agents
- real skills
- real stack detection
- real schema validation
- resume logic
- git integration

## 3. Absolute Constraints

Forbidden in slice 1:

- any real IO
- any persistence outside memory
- any code not required by the current failing test
- any implementation for future slices
- any plugin mechanism
- any infrastructure adapter beyond test doubles
- any behavior not covered by the defined test plan

## 4. Allowed Components Only

### Domain

- `Mission`
- `Task`
- `Session`
- `Decision`
- `Verdict`
- `FailureReport`
- `Event`
- `TaskContract`
- `SessionState`
- `SessionStatus`
- `AgentRole`

### Application

- `StateMachine`
- `SessionRunner`
- `RuntimePolicy`
- `ScholarPort`
- `PlannerPort`
- `BuilderPort`
- `CriticPort`
- `EventStorePort`
- `SessionSummary`

### Test Support Only

- `FakeScholar`
- `FakePlanner`
- `FakeBuilder`
- `FakeCritic`
- `InMemoryEventStore`

## 5. Structured Test Plan

### Domain Tests

File order:

1. `tests/domain/task_contract_tests.rs`
2. `tests/domain/verdict_tests.rs`
3. `tests/domain/session_tests.rs`
4. `tests/domain/failure_report_tests.rs`

Test names:

- `builds_task_contract_with_budget_2`
- `builds_task_contract_with_budget_3`
- `rejects_task_contract_with_budget_below_2`
- `rejects_task_contract_with_budget_above_3`
- `stores_single_selected_task_scope`
- `builds_approve_verdict_without_required_changes`
- `builds_stop_verdict_without_required_changes`
- `builds_revise_verdict_with_required_changes`
- `rejects_revise_verdict_without_required_changes`
- `accepts_revise_verdict_with_one_required_change`
- `starts_session_in_initialized_state`
- `marks_session_completed_as_terminal`
- `marks_session_stopped_as_terminal`
- `rejects_second_terminal_transition`
- `increments_iteration_count_within_budget`
- `detects_budget_exhaustion_after_max_iterations`
- `builds_failure_report_for_policy_violation`
- `builds_failure_report_for_technical_error`
- `builds_failure_report_for_critic_stop`
- `failure_report_requires_terminal_stopped_status`

### State Machine Tests

File order:

1. `tests/state_machine/state_machine_transition_tests.rs`
2. `tests/state_machine/state_machine_guard_tests.rs`

Test names:

- `allows_initialized_to_mission_analyzed`
- `allows_mission_analyzed_to_task_selected`
- `allows_task_selected_to_snapshot_captured`
- `allows_snapshot_captured_to_builder_running`
- `allows_builder_running_to_critic_reviewing`
- `allows_critic_reviewing_to_planner_deciding`
- `allows_planner_deciding_to_builder_running`
- `allows_planner_deciding_to_completed`
- `allows_planner_deciding_to_stopped`
- `allows_non_terminal_state_to_stopped`
- `rejects_initialized_to_task_selected`
- `rejects_builder_running_to_completed`
- `rejects_critic_reviewing_to_builder_running`
- `rejects_completed_to_any_other_state`
- `rejects_stopped_to_any_other_state`
- `rejects_second_task_selection_in_same_session`

### SessionRunner Tests

File order:

1. `tests/support/fake_scholar.rs`
2. `tests/support/fake_planner.rs`
3. `tests/support/fake_builder.rs`
4. `tests/support/fake_critic.rs`
5. `tests/support/in_memory_event_store.rs`
6. `tests/session_runner/session_runner_happy_path_tests.rs`
7. `tests/session_runner/session_runner_failure_tests.rs`

Test names:

- `runs_agents_in_strict_order_for_happy_path`
- `activates_only_one_agent_at_a_time`
- `records_events_in_monotonic_sequence`
- `completes_session_after_approve_then_complete_decision`
- `returns_session_summary_on_success`
- `stops_when_critic_returns_invalid_revise_verdict`
- `stops_when_planner_requests_retry_without_budget`
- `stops_when_planner_output_is_invalid_for_current_state`
- `returns_failure_report_on_hard_stop`
- `does_not_call_builder_again_after_terminal_stop`

### End-to-End In-Memory Tests

File order:

1. `tests/e2e/session_e2e_happy_path_tests.rs`
2. `tests/e2e/session_e2e_revision_loop_tests.rs`
3. `tests/e2e/session_e2e_stop_tests.rs`

Test names:

- `completes_single_iteration_session_end_to_end`
- `produces_expected_terminal_summary_for_happy_path`
- `completes_after_one_revision_within_budget`
- `passes_required_changes_from_critic_to_next_builder_iteration`
- `increments_iteration_count_across_revision_loop`
- `stops_when_budget_is_exhausted`
- `stops_when_critic_emits_stop_verdict`
- `stops_when_revise_verdict_has_no_required_changes`
- `emits_exactly_one_terminal_outcome`

## 6. TDD Implementation Order

1. `builds_task_contract_with_budget_2`
2. `rejects_task_contract_with_budget_below_2`
3. `rejects_task_contract_with_budget_above_3`
4. `builds_revise_verdict_with_required_changes`
5. `rejects_revise_verdict_without_required_changes`
6. `starts_session_in_initialized_state`
7. `marks_session_completed_as_terminal`
8. `allows_initialized_to_mission_analyzed`
9. `allows_mission_analyzed_to_task_selected`
10. `rejects_initialized_to_task_selected`
11. `rejects_builder_running_to_completed`
12. create fake ports
13. `runs_agents_in_strict_order_for_happy_path`
14. `completes_session_after_approve_then_complete_decision`
15. `completes_single_iteration_session_end_to_end`
16. `completes_after_one_revision_within_budget`
17. `stops_when_revise_verdict_has_no_required_changes`
18. `stops_when_budget_is_exhausted`

## 7. Dev Agent Discipline Rules

- implement the smallest code that makes the current test pass
- write no production code before a failing test exists
- add no field, type, branch, or helper unless required by a current failing test
- do not implement excluded concerns even if they seem obvious
- do not prepare abstractions for future slices
- do not add infrastructure placeholders beyond the approved test doubles
- stop at green, then move to the next test

## 8. Definition of Done

Slice 1 is done only when all of the following pass:

- `rejects_revise_verdict_without_required_changes`
- `rejects_builder_running_to_completed`
- `completes_single_iteration_session_end_to_end`
- `stops_when_budget_is_exhausted`

And these conditions hold:

- no real IO exists anywhere in the slice
- runtime loop works fully in memory
- one agent is active at a time
- terminal result is always exactly one of `SessionSummary` or `FailureReport`

## 9. First Execution Step

- First test file: `tests/domain/task_contract_tests.rs`
- First test: `builds_task_contract_with_budget_2`
- Test intention: prove that a `TaskContract` can be created with the minimum valid iteration budget.
- Minimum elements to create:
  - `TaskContract`
  - constructor for `TaskContract`
  - field `iteration_budget`
  - minimal error type or result shape needed to express construction success or failure
