# Product Brief: Continuum

## Executive Summary

Continuum is a local-first personal orchestration runtime for disciplined agentic software development on real repositories. It is designed for a single experienced backend engineer who wants strict control over execution, bounded scope, TDD enforcement, and complete traceability rather than open-ended autonomous coding.

V1 is intentionally narrow. A session accepts one mission, analyzes it, selects one atomic task, executes that task through a controlled Builder loop, reviews the outcome through a prescriptive Critic, and stops with either a completed session summary or a failure report. The product value is not general autonomy; it is reliable execution with explicit policies and auditable state.

## The Problem

Current agentic coding tools are powerful but operationally loose. They often blur planning and execution, allow unbounded interpretation of feedback, hide internal TDD behavior, and provide weak guarantees about scope, workspace state, and stopping conditions.

For a rigorous engineering workflow, that creates three practical failures: too much cognitive supervision, too much trust in opaque behavior, and not enough evidence to explain why a run succeeded or failed. Continuum exists to replace that ambiguity with a deterministic local runtime.

## The Solution

Continuum runs a strict state machine over one repository and one selected atomic task. It activates only one agent at a time and only through controlled skills. Scholar analyzes the mission and identifies candidate tasks. Planner selects exactly one atomic task and fixes its execution contract. Builder executes the task through an explicit six-step internal TDD loop. Critic reviews the result and prescribes required changes when revision is needed. Planner then either completes the session, retries Builder within a bounded dynamic budget, or stops the run.

The runtime maintains explicit workspace snapshots, command artifacts, append-only event logs, and terminal outputs that are always structured. Every hard stop must produce a failure report. Every successful run must produce a session summary.

## What Makes This Different

- The runtime is local-only and optimized for one operator, not for collaboration or SaaS workflows.
- The execution model is a state machine, not free-form chat.
- Builder behavior is not a black box; the TDD cycle is explicitly decomposed and validated.
- Critic feedback is prescriptive through `required_changes`, not advisory prose.
- Workspace state is explicit through `WorkspaceSnapshot` objects, not implied by the filesystem alone.
- Hard stops are first-class outputs with a mandatory `FailureReport`.

## Who This Serves

The primary user is a single experienced backend engineer working on local repositories across Rust, TypeScript, PHP, and .NET codebases. The user expects strict TDD, bounded scope, explicit orchestration, and auditability, and does not want the runtime to improvise beyond the defined contract.

## Scope

### In Scope for V1

- Local execution on one repository root.
- One active session at a time.
- One selected atomic task per session.
- One active agent at a time.
- Four agents only: Scholar, Planner, Builder, Critic.
- Controlled skills for reading code, searching code, stack detection, running tests, running linters and formatters, safe file editing, diff inspection, and sandboxed shell execution.
- Explicit Builder iteration with six internal sub-steps.
- Dynamic Builder iteration budget bounded to `2..3` iterations per selected task.
- Structured event logging, command artifacts, workspace snapshots, session summary, and failure report.

### Out of Scope for V1

- GitHub integration.
- Parallel execution.
- Multiple worktrees.
- Multiple selected tasks per session.
- Multi-user support.
- Automatic merge or reviewer automation.
- Long-term memory systems.
- Full multi-language abstraction beyond practical stack detection and skill routing.
- Rich product UI.

## Success Criteria

Continuum V1 is successful when a session can complete one atomic task end to end with the following guarantees:

- The selected task changes one observable behavior in one functional area.
- Builder produces explicit evidence for `propose_test`, `run_test_red`, `implement_minimal`, `run_test_green`, `optional_cleanup`, and `final_validation`.
- Critic returns either `approve`, `revise`, or `stop`, and any `revise` verdict includes actionable `required_changes`.
- Every Builder iteration consumes a `WorkspaceSnapshot` and produces a new `WorkspaceSnapshot`.
- The runtime enforces the Planner-defined iteration budget, bounded to `2..3`.
- The runtime records stack detection as a multi-dimensional structure usable by skills.
- A failed run always ends with a valid `FailureReport`.
- A successful run always ends with a structured session summary and passing final validation.

## Core Domain Model

### Mission

A Mission is the immutable user intent for one repository-local session. In V1, a Mission may lead to analysis of multiple candidate tasks but only one selected task may be executed.

### Task

A Task is the smallest executable unit that changes one observable behavior in one bounded functional area. A valid V1 task must fit a bounded file scope, use one targeted test command, and avoid cross-cutting change.

### Session

A Session is the bounded execution container for one Mission, one selected Task, one repository root, one active state, one iteration budget, and an append-only event log. It terminates as `completed` or `stopped`.

### Agent

An Agent is a role activated by the runtime state machine. Agents do not operate freely; they emit only structured outputs and can invoke only allowed skills for their role.

### Skill

A Skill is a controlled executable capability with validated inputs, bounded behavior, and traceable outputs. No direct action outside a skill contract is allowed.

### WorkspaceSnapshot

A WorkspaceSnapshot is the explicit captured state of the working repository at a specific point in time. It includes base revision metadata, file state, and a diff reference, and is the canonical state boundary for Builder iterations.

### Event

An Event is an immutable record of a state transition, skill execution, snapshot capture, decision, verdict, or terminal outcome. Events are append-only, strictly ordered, and fully referential.

### Decision / Verdict

Decision belongs to Planner and controls the flow of execution. Verdict belongs to Critic and evaluates the Builder result; a revision verdict must specify concrete `required_changes`.

### FailureReport

FailureReport is the mandatory terminal artifact for any hard stop. It communicates why execution stopped, where it stopped, what the last valid state was, and what evidence supports the stop.

## Formal Execution Loop

### 1. Scholar: Analyze Mission

Input: mission, repository root, read-only discovery skills.

Output: `mission_analysis` containing stack detection, ambiguity status, and candidate atomic tasks.

Pass condition: at least one candidate task is atomic and no blocking ambiguity remains.

### 2. Planner: Select Task

Input: `mission_analysis`.

Output: `task_contract` for exactly one selected task.

Pass condition: the contract defines allowed files, targeted test command, final validation command, acceptance checks, dynamic Builder budget in `2..3`, and scope boundaries.

### 3. Runtime: Capture Initial WorkspaceSnapshot

Input: repository root and `task_contract`.

Output: initial `WorkspaceSnapshot`.

Pass condition: snapshot capture succeeds before any write occurs.

### 4. Builder: Execute Iteration

Input: `task_contract`, current snapshot, and optional Critic-prescribed `required_changes` from a previous review.

Output: `build_iteration_report` plus a new `WorkspaceSnapshot`.

Pass condition: Builder completes the internal six-step cycle in order and remains within policy.

### 5. Critic: Review Build

Input: `task_contract`, `build_iteration_report`, diff, and output snapshot.

Output: `verdict` with findings and, when revising, mandatory `required_changes`.

Pass condition: always returns control to Planner.

### 6. Planner: Decide Next State

Input: `verdict`, session history, remaining iteration budget.

Output: `decision` of `complete`, `retry_builder`, or `stop`.

Pass condition: `complete` on approval, `retry_builder` only if budget remains, otherwise `stop`.

### 7. Runtime: Finish Session

Input: terminal state.

Output: `session_summary` on success or `FailureReport` on stop.

Pass condition: session closes permanently with no second task selection.

## Builder Execution Contract

Builder is explicitly iterative and must execute these six sub-steps in this exact order for each iteration:

1. `propose_test`
2. `run_test_red`
3. `implement_minimal`
4. `run_test_green`
5. `optional_cleanup`
6. `final_validation`

### Builder Rules

- `propose_test` must define or adapt the targeted test that proves the missing behavior.
- `run_test_red` must demonstrate failure of the targeted test before implementation is accepted.
- `implement_minimal` may modify only files allowed by the task contract.
- `run_test_green` must demonstrate the targeted test passing.
- `optional_cleanup` may simplify or tidy local code only if it does not expand scope, behavior, or file boundaries.
- `final_validation` must execute the contract-defined final validation command.
- Each iteration must read from `snapshot_in` and write a new `snapshot_out`.

## Critic Contract

Critic is prescriptive, not conversational. Every Critic output must include:

- `verdict`: `approve`, `revise`, or `stop`
- `rule_results`: validation results against acceptance and policy rules
- `findings`: concise review findings
- `required_changes`: mandatory when `verdict = revise`, empty when `verdict = approve`

Each `required_change` must specify:

- `type`
- `file`
- `constraint`
- `priority`

This structure is canonical because Builder must react to explicit constraints rather than interpret free-form criticism.

## Structured Runtime Contracts

### Canonical Agent Output

All agent outputs must conform to `continuum.agent-output.v1` and include:

- `session_id`
- `event_id`
- `sequence`
- `timestamp_utc`
- `agent`
- `step`
- `intention`
- `action`
- `input_event_ids`
- `files_impacted`
- `status`
- `confidence`
- `payload`

Builder outputs additionally require:

- `snapshot_in`
- `snapshot_out`

### WorkspaceSnapshot Contract

Each snapshot must capture:

- base revision information, including commit hash when available
- tracked and modified file state
- diff reference
- parent snapshot reference when applicable

The runtime does not perform automatic rollback in V1. Recovery and auditability rely on the last valid snapshot plus the associated diff artifact.

### Stack Detection Contract

Stack detection must be multi-dimensional and structured as:

```json
{
  "primary": "rust",
  "secondary": ["typescript"],
  "test_runner": "cargo test",
  "formatter": "cargo fmt",
  "linter": "cargo clippy",
  "build_system": "cargo",
  "workspace_shape": "single-package",
  "legacy_markers": []
}
```

This structure is mandatory because skill routing depends on more than the primary language.

### FailureReport Contract

Every hard stop must emit `continuum.failure-report.v1` with:

- `session_id`
- `failure_id`
- `timestamp_utc`
- `stage`
- `reason_code`
- `reason`
- `suggestion` when useful
- `last_valid_event_id`
- `last_valid_snapshot_id` when available
- `evidence`
- `session_status = stopped`

## Guardrails

- One active agent at a time.
- Builder is the only agent allowed to modify code or tests.
- Critic never modifies the workspace or artifacts.
- Scholar and Planner are read-only.
- Planner is the only agent that can choose the task, retry Builder, complete the session, or stop the session.
- The runtime may execute only one selected task per session.
- A V1 task must remain in one functional area and one bounded scope.
- The default file budget is at most `5` modified files unless the Planner sets a tighter contract.
- A V1 task cannot change dependencies, CI, infrastructure, migrations, or multiple bounded contexts.
- Builder iteration budget is dynamic but must be in the range `2..3`.
- If `baseline` fails before Builder work, stop.
- If the red test does not fail, stop.
- If Builder exceeds allowed files or scope, stop.
- If a command exceeds sandbox timeout, stop.
- If an agent output fails schema validation, stop.
- If Scholar or Planner cannot produce an unambiguous contract, stop.
- If Critic detects a major policy violation or scope drift, Critic may return `stop`.

## Minimal Traceability Model

V1 requires these artifacts as the minimum traceability baseline:

- `session.jsonl` containing append-only structured events
- `artifacts/commands/*.json` containing command execution evidence
- `artifacts/diffs/*.diff` containing snapshot diff evidence
- `session_summary.json` for successful completion
- `failure_report.json` for hard-stop termination

This is the minimum required evidence set for replay, audit, and diagnosis.

## Decision Log / Changelog V2

- Replaced opaque Builder execution with an explicit six-step internal Builder loop.
- Upgraded Critic from advisory review to prescriptive review with mandatory `required_changes` on revision.
- Introduced `WorkspaceSnapshot` as a first-class domain concept and runtime contract.
- Replaced fixed retry count with a dynamic Planner-defined Builder budget bounded to `2..3`.
- Replaced single-value stack detection with a multi-dimensional stack profile.
- Added `FailureReport` as the mandatory terminal artifact for every hard stop.
