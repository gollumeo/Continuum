---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7]
inputDocuments:
  - project-directives/planning-artifacts/product-brief.md
workflowType: 'architecture'
project_name: 'Continuum'
user_name: 'Gollumeo'
date: '2026-03-31'
---

# Architecture: Continuum V1

## Architectural Intent

Continuum V1 is a local-only Rust runtime that executes one bounded software task on one local repository through a strict state machine. The architecture optimizes for four things only: enforceable boundaries, deterministic execution flow, explicit evidence, and fast implementation.

V1 deliberately avoids distributed design, plugin systems, parallel execution, GitHub integration, multi-user concepts, and cross-repository orchestration. The correct architecture is therefore a small layered monolith with explicit contracts and narrow IO surfaces.

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**

The product brief defines a single-session runtime that accepts one mission, identifies candidate atomic tasks, selects exactly one task, executes it through Planner, Builder, and Critic under a strict runtime loop, and finishes with either a session summary or a failure report. The runtime must record snapshots, diffs, command evidence, structured events, and terminal artifacts.

**Non-Functional Requirements:**

The dominant NFRs are determinism, auditability, local execution, constrained writes, low architectural complexity, and implementation speed. Reliability matters more than flexibility. Traceability matters more than throughput. Simplicity of restart and diagnosis matters more than future extensibility.

**Scale & Complexity:**

- Primary domain: local backend orchestration runtime
- Complexity level: medium
- Estimated architectural components: 8 to 10 cohesive modules inside one Rust crate

### Technical Constraints & Dependencies

- Local repository only
- One active session at a time
- One active agent at a time
- One selected task per session
- No parallelism
- No multi-worktree support
- No GitHub integration
- No plugin system in V1
- Runtime implemented in Rust
- Hybrid persistence: SQLite plus filesystem artifacts

### Cross-Cutting Concerns Identified

- State-machine enforcement
- Schema validation of all agent outputs
- Write boundary enforcement for Builder
- Snapshot and diff capture before and after writes
- Clear distinction between policy failure, technical error, and Critic stop
- Minimal but queryable persistence

## Foundation Decision

Continuum V1 should start as a single Rust binary crate with internal modules, not a Cargo workspace with multiple crates. This is the fastest path to a working runtime while preserving clear boundaries in code. If boundaries hold at module level, splitting into crates can wait until there is real pressure from compile time, ownership, or team parallelism.

## Decoupage Architectural

### Layering Decision

Continuum V1 uses three layers plus a thin runtime entrypoint:

1. Domain
2. Application
3. Infrastructure
4. Runtime shell

### Domain Layer

The domain layer contains pure business concepts and invariants. It must not depend on filesystem access, shell execution, SQLite, JSON serialization details, or process spawning.

Responsibilities:

- Define the meaning and invariants of Mission, Task, Session, Decision, Verdict, FailureReport, WorkspaceSnapshot metadata, Event metadata, and policy concepts
- Define allowed state transitions as pure rules
- Define validation rules for atomic task scope, iteration budget range, agent role permissions, and verdict semantics
- Define error taxonomy at the business level

Must remain pure:

- Entities and value objects
- Transition guards
- Policy validation
- Classification logic for hard-stop reasons

### Application Layer

The application layer orchestrates use cases. It translates domain rules into executable flows and coordinates infrastructure ports.

Responsibilities:

- Start a session from a mission
- Ask Scholar for mission analysis
- Ask Planner for task selection and `task_contract`
- Capture snapshots
- Run Builder and Critic loops
- Enforce iteration budget and hard-stop rules
- Persist session state and events
- Build terminal artifacts

The application layer may depend on interfaces for IO, never on concrete adapters.

### Infrastructure Layer

The infrastructure layer implements side effects.

Responsibilities:

- SQLite storage
- Filesystem artifact writing
- Repository inspection
- Diff capture
- Snapshot capture
- Sandboxed command execution
- Safe file editing
- Agent invocation plumbing
- Schema validation implementation

This layer may depend on IO, external processes, serialization libraries, and OS interaction.

### Runtime Shell

The runtime shell is thin. It parses CLI input, opens the repository root, wires adapters, starts the application service, and renders final status.

## Modele Coeur

### Mission

Purpose: immutable user intent for one session.

Fields:

- `mission_id`
- `repository_root`
- `user_prompt`
- `created_at_utc`

Invariant: a Mission can yield many candidate tasks but only one selected task in V1.

### Task

Purpose: smallest executable unit of change.

Fields:

- `task_id`
- `title`
- `functional_area`
- `observable_behavior`
- `allowed_file_globs`
- `targeted_test_command`
- `final_validation_command`
- `acceptance_checks`
- `scope_notes`

Invariant: one functional area, bounded file scope, one targeted test command, no cross-cutting change.

### Session

Purpose: bounded runtime container.

Fields:

- `session_id`
- `mission_id`
- `selected_task_id`
- `current_state`
- `iteration_budget`
- `iteration_count`
- `current_snapshot_id`
- `status`

Invariant: exactly one terminal status, `completed` or `stopped`.

### Decision

Purpose: Planner-controlled flow decision.

Values:

- `complete`
- `retry_builder`
- `stop`

Invariant: only Planner can emit a Decision; runtime validates it against remaining budget and current state.

### Verdict

Purpose: Critic evaluation of one Builder iteration.

Values:

- `approve`
- `revise`
- `stop`

Required payload:

- `rule_results`
- `findings`
- `required_changes`

Invariant: `required_changes` is mandatory and non-empty when verdict is `revise`.

### FailureReport

Purpose: terminal explanation for a hard stop.

Fields:

- `failure_id`
- `session_id`
- `stage`
- `reason_code`
- `reason`
- `suggestion`
- `last_valid_event_id`
- `last_valid_snapshot_id`
- `evidence`
- `session_status`

Invariant: every hard stop must produce exactly one FailureReport.

### WorkspaceSnapshot

Purpose: canonical runtime boundary around repository state.

Fields:

- `snapshot_id`
- `session_id`
- `parent_snapshot_id`
- `base_revision`
- `has_git`
- `tracked_files`
- `modified_files`
- `diff_artifact_path`
- `captured_at_utc`

Decision: `WorkspaceSnapshot` is a domain object with lightweight metadata only. Heavy diff content stays outside the domain in filesystem artifacts.

### Event

Purpose: immutable ordered runtime fact.

Fields:

- `event_id`
- `session_id`
- `sequence`
- `event_type`
- `state_before`
- `state_after`
- `actor`
- `payload_ref`
- `created_at_utc`

Decision: event payloads that are large or command-specific are referenced, not duplicated inline in the domain object.

### Agent Output Contracts

All agent outputs conform to `continuum.agent-output.v1` and share the same envelope. The payload is role-specific, but the envelope stays fixed so the runtime can validate, persist, and replay decisions consistently.

Common envelope fields:

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

Builder-specific additions:

- `snapshot_in`
- `snapshot_out`

### Decision Claire sur `task_contract`

Decision: `task_contract` belongs to the application layer, not the domain core.

Why:

- It is not the business intent itself; that is the Task.
- It encodes executable constraints for the runtime and Builder: allowed files, commands, acceptance checks, and iteration budget.
- It is generated by Planner from domain inputs and immediately consumed by runtime orchestration.

Definition:

`task_contract` is an immutable execution specification derived from `Task` plus session policy.

Minimum fields:

- `session_id`
- `task_id`
- `allowed_file_globs`
- `targeted_test_command`
- `final_validation_command`
- `acceptance_checks`
- `iteration_budget`
- `scope_boundaries`
- `forbidden_actions`

Persistence decision:

- Store the normalized contract in SQLite for runtime decisions
- Store the full serialized contract as an event payload artifact for audit

## Moteur d'Execution

### State Machine Decision

The runtime is a strict finite state machine. No agent can self-route to the next step. Only the runtime advances state after validating the current output.

### States

1. `session_initialized`
2. `mission_analyzed`
3. `task_selected`
4. `snapshot_captured`
5. `builder_running`
6. `critic_reviewing`
7. `planner_deciding`
8. `completed`
9. `stopped`

### Allowed Transitions

- `session_initialized -> mission_analyzed`
- `mission_analyzed -> task_selected`
- `task_selected -> snapshot_captured`
- `snapshot_captured -> builder_running`
- `builder_running -> critic_reviewing`
- `critic_reviewing -> planner_deciding`
- `planner_deciding -> builder_running`
- `planner_deciding -> completed`
- `planner_deciding -> stopped`
- `any non-terminal state -> stopped` on validated hard stop

Forbidden transitions:

- No direct `builder_running -> completed`
- No direct `critic_reviewing -> builder_running`
- No second `task_selected` in the same session
- No state change caused only by agent prose

### Single Active Agent Rule

At any instant, exactly one agent role is active. The runtime acquires the role, executes the allowed skill set for that role, validates the agent output, persists the event, and releases the role before moving on.

### Planner / Builder / Critic Loop

Loop shape:

1. Planner creates `task_contract`
2. Runtime captures initial snapshot
3. Builder executes one full six-step iteration
4. Critic returns `approve`, `revise`, or `stop`
5. Planner decides `complete`, `retry_builder`, or `stop`
6. Repeat only through Planner approval and only while budget remains

### Iteration Budget

Decision: budget is stored on Session and copied into `task_contract` at selection time.

Rules:

- Planner chooses budget dynamically within `2..3`
- Runtime decrements after each completed Builder iteration reviewed by Critic
- Planner cannot override the remaining budget mid-session

### Hard Stops

Hard stops are runtime decisions, not free-form agent decisions.

They occur when:

- baseline validation fails before Builder work
- the proposed red test is not actually red
- Builder modifies files outside contract
- command sandbox fails or times out
- schema validation fails
- Planner output is ambiguous or invalid
- Critic emits `stop`
- Planner requests retry with no budget left

When a hard stop occurs, the runtime freezes further execution, emits a terminal event, and writes `failure_report.json`.

## Contrats d'Agents et de Skills

### Frontiere Agent / Skill

Decision: agents decide, skills execute.

- An agent may reason, choose, and emit a structured output.
- A skill may inspect files, run commands, edit files, or collect evidence.
- An agent never touches the repository directly.
- A skill never changes session state directly.

The runtime is the only authority that can:

- activate an agent
- authorize a skill call
- validate output schemas
- persist events
- move the state machine

### Scholar

Responsibilities:

- inspect the repository read-only
- detect stack profile
- identify ambiguities
- propose candidate atomic tasks

Not allowed:

- write files
- choose the final task
- run mutating commands

### Planner

Responsibilities:

- select exactly one task
- produce `task_contract`
- choose iteration budget in `2..3`
- decide `complete`, `retry_builder`, or `stop`

Not allowed:

- write files
- bypass Critic
- expand scope after task selection

### Builder

Responsibilities:

- execute the six-step TDD loop exactly in order
- modify only contract-approved files
- produce explicit step evidence
- end each iteration with a new snapshot

Not allowed:

- change contract or budget
- self-approve work
- modify files outside the contract

### Critic

Responsibilities:

- review Builder output against acceptance and policy
- emit `approve`, `revise`, or `stop`
- provide structured `required_changes` when revising

Not allowed:

- edit files
- invoke write skills
- decide retry budget

### Skills V1 Minimales

Minimal skill set:

1. `repo_search`
2. `repo_read`
3. `stack_detect`
4. `command_run_readonly`
5. `targeted_test_run`
6. `final_validation_run`
7. `safe_edit`
8. `diff_inspect`
9. `snapshot_capture`

Decision: do not model a generic plugin registry. Hardcode the V1 skill catalog in Rust enums and role policies.

### How to Prevent Actions Outside Contract

Enforcement is runtime-driven through four gates:

1. Role policy gate: role-to-skill allowlist
2. Contract gate: Builder writes allowed only on `allowed_file_globs`
3. Command gate: command templates restricted to contract-defined test and validation commands plus approved read-only discovery commands
4. Schema gate: invalid agent output stops the session

## Persistance

### Hybrid Strategy

Use SQLite for structured operational state and filesystem for append-only evidence artifacts.

### What Goes in SQLite

- sessions
- mission records
- selected task metadata
- normalized `task_contract`
- snapshots metadata
- ordered event index
- agent output envelope metadata
- current session pointer and terminal status

### What Stays in Filesystem Artifacts

- `session.jsonl`
- full agent payload JSON files when large
- command execution records in `artifacts/commands/*.json`
- diffs in `artifacts/diffs/*.diff`
- `session_summary.json`
- `failure_report.json`

### Source of Truth

Decision:

- SQLite is the source of truth for runtime state and any data needed to resume or query the session.
- Filesystem artifacts are the source of truth for raw evidence blobs and large append-only outputs.

Operational rule:

- The runtime always decides from SQLite.
- The runtime never reconstructs authoritative session state from JSONL alone in V1.

### Minimal Indexing Strategy

Keep indexing small:

- index `sessions(session_id)`
- index `events(session_id, sequence)`
- index `snapshots(session_id, captured_at_utc)`
- index `task_contracts(session_id)`

No full-text search in V1. Artifact filenames include `session_id` and monotonic sequence for direct lookup.

## Workspace Runtime

### Snapshot Capture

Capture snapshots at three moments:

1. immediately after task selection and before any write
2. after each Builder iteration
3. at terminal success or stop if the last event changed the workspace

Snapshot process:

- detect whether the repo has Git
- record base revision if available
- enumerate modified files relevant to the session
- generate a diff artifact
- persist snapshot metadata in SQLite

### Diff Handling

Decision: diffs are filesystem artifacts referenced by snapshot metadata.

Why:

- diffs can be large
- they are evidence, not control state
- the runtime mostly needs references, not inline storage

### Allowed File Control

Builder writes are checked twice:

1. pre-write: the requested target files must match `allowed_file_globs`
2. post-write: the resulting diff must still match the same boundary

If either check fails, stop immediately.

### Write Boundaries

Only Builder can invoke `safe_edit`. `safe_edit` must reject:

- writes outside repository root
- writes outside contract file globs
- deletes or renames outside explicit contract permission
- changes to dependency manifests, CI files, migrations, or infrastructure paths in V1

### Minimal Resume State in V1

Resume is intentionally narrow.

Supported:

- resume a non-terminal session from the last committed state if SQLite state is valid and the current workspace still matches the last recorded snapshot assumptions

Not supported:

- merging divergent local changes automatically
- resuming after manual repository drift without explicit stop/restart

Decision: if workspace drift is detected at resume time, stop and emit FailureReport instead of attempting repair.

## Gestion des Erreurs et Arrets

### Where Hard Stops Are Decided

Hard stops are decided in the application runtime after evaluating one of three sources:

1. technical execution failure from infrastructure
2. policy violation from runtime validation
3. `stop` verdict from Critic or `stop` decision from Planner

The agent may recommend stop. Only the runtime makes it terminal.

### FailureReport Escalation

Process:

1. classify the stop reason
2. capture last valid event and snapshot references
3. collect supporting evidence references
4. persist terminal event in SQLite and JSONL
5. write `failure_report.json`

### Error Taxonomy

Use three top-level categories:

1. `technical_error`
2. `policy_violation`
3. `critic_stop`

Examples:

- command timeout -> `technical_error`
- schema invalid -> `policy_violation`
- out-of-scope file write -> `policy_violation`
- Critic major scope drift -> `critic_stop`

Decision: Planner `stop` after budget exhaustion still maps to `policy_violation`, because the runtime contract was exhausted, not broken by infrastructure.

## Decoupage Modulaire Rust

### Recommended Structure

Use one crate with this folder structure:

- `src/main.rs`
- `src/runtime/mod.rs`
- `src/domain/mod.rs`
- `src/domain/mission.rs`
- `src/domain/task.rs`
- `src/domain/session.rs`
- `src/domain/decision.rs`
- `src/domain/verdict.rs`
- `src/domain/snapshot.rs`
- `src/domain/event.rs`
- `src/domain/failure.rs`
- `src/application/mod.rs`
- `src/application/session_runner.rs`
- `src/application/contracts.rs`
- `src/application/state_machine.rs`
- `src/application/policies.rs`
- `src/application/ports.rs`
- `src/infrastructure/mod.rs`
- `src/infrastructure/sqlite/`
- `src/infrastructure/filesystem/`
- `src/infrastructure/repo/`
- `src/infrastructure/commands/`
- `src/infrastructure/agents/`
- `src/infrastructure/schema/`

### Module Responsibilities

`domain`

- pure types and invariants

`application`

- use cases, orchestration, ports, runtime policies, state transitions

`infrastructure/sqlite`

- repositories for sessions, events, snapshots, task contracts

`infrastructure/filesystem`

- artifact writing, JSONL appends, diff file management

`infrastructure/repo`

- git detection, file listing, workspace boundary checks, snapshot materialization

`infrastructure/commands`

- sandboxed command execution and captured outputs

`infrastructure/agents`

- adapters for Scholar, Planner, Builder, Critic invocation

`infrastructure/schema`

- validation of `continuum.agent-output.v1` and `continuum.failure-report.v1`

`runtime`

- CLI entry, dependency wiring, process lifecycle

### Allowed Dependencies Between Modules

- `runtime -> application + infrastructure`
- `application -> domain`
- `application -> application::ports`
- `infrastructure -> domain + application::ports`
- `domain -> nothing internal outside domain`

Forbidden:

- `domain -> infrastructure`
- `domain -> application`
- `application -> concrete infrastructure modules`
- `infrastructure -> runtime`

## Risques Techniques V1

1. Snapshot drift between runtime state and real workspace
Why dangerous: replay and resume become misleading.
Containment: compare live workspace with last snapshot at every resume and before each Builder iteration.

2. Builder escaping file boundaries indirectly
Why dangerous: the main V1 safety guarantee collapses.
Containment: enforce both requested-path checks and post-diff checks.

3. Overly loose `task_contract` generation
Why dangerous: Builder receives room to drift and Critic becomes cleanup instead of control.
Containment: validate contract strictness before the first snapshot; reject vague globs and multi-area tasks.

4. Ambiguous agent payloads
Why dangerous: runtime decisions become prose-driven instead of contract-driven.
Containment: strict schema validation and typed payload mapping before persistence.

5. Command execution variability across stacks
Why dangerous: false failures and noisy stop conditions.
Containment: keep stack detection structured but small, and support only a minimal set of command patterns in V1.

6. SQLite and artifact filesystem falling out of sync
Why dangerous: evidence exists without queryable state, or the reverse.
Containment: write SQLite state and artifact references in a single ordered application transaction pattern; stop on partial failure.

7. Resume logic becoming accidental recovery logic
Why dangerous: V1 complexity explodes and state safety drops.
Containment: support resume only when workspace state still matches assumptions; otherwise stop.

8. Critic becoming advisory in practice
Why dangerous: Planner and Builder start interpreting prose and loop control weakens.
Containment: reject `revise` verdicts with empty `required_changes`.

## Implementation Patterns & Consistency Rules

### Naming Rules

- Rust modules use `snake_case`
- domain types use singular nouns
- event types use explicit stable names such as `mission_analyzed`, `task_selected`, `builder_iteration_completed`
- JSON contract fields use `snake_case`

### Structural Rules

- no business rules in infrastructure
- no process spawning outside `infrastructure/commands`
- no file writes outside `infrastructure/filesystem`, `infrastructure/repo`, and approved Builder edit paths
- session transitions happen through `application/state_machine.rs` only

### Format Rules

- all timestamps in UTC ISO-8601
- all terminal artifacts are JSON objects with explicit schema version fields
- all event records append in sequence order only

### Enforcement Rules

- every state transition is validated before persistence
- every agent output is schema-validated before use
- every Builder iteration must have `snapshot_in` and `snapshot_out`

## Ordre d'Implementation Recommande

### Sequence

1. Define pure domain types and state transition rules.
This locks the vocabulary and prevents the runtime from becoming ad hoc.

2. Implement SQLite session, event, snapshot, and task-contract repositories.
This creates the control plane before any agent execution.

3. Implement artifact writing for JSONL, command records, diffs, session summary, and failure report.
This establishes evidence early and supports diagnosis.

4. Implement repository inspection and snapshot capture.
This gives the runtime its state boundary before Builder exists.

5. Implement schema validation for agent outputs and failure reports.
This prevents prose-driven orchestration.

6. Implement the application state machine and `SessionRunner` without real agents.
Use stubs first to prove transition correctness under TDD.

7. Implement Scholar and Planner adapters and make task selection end-to-end.
Stop before Builder until contracts are trustworthy.

8. Implement Builder with only the strict six-step loop and write-boundary enforcement.
Do not add convenience behaviors.

9. Implement Critic and Planner loop-back decisions.
This completes the controlled iteration cycle.

10. Add narrow resume support and terminal CLI reporting.
Only after the core runtime is stable.

### TDD Guidance

Build from pure rules outward:

- first test state transitions and invariants
- then test persistence contracts
- then test snapshot and boundary enforcement
- then test end-to-end session scenarios with fake agents
- only then connect real agent adapters

## Architecture Validation Results

### Coherence Validation

The architecture is coherent with the product brief because it preserves the core constraints: one session, one task, one active agent, explicit Builder loop, prescriptive Critic, dynamic but bounded iteration budget, and mandatory FailureReport on hard stop.

### Coverage Validation

All requested areas are covered explicitly:

- layered architecture
- core model
- strict execution engine
- agent and skill contracts
- hybrid persistence
- workspace runtime
- error and stop handling
- Rust module layout
- key technical risks
- implementation order

### Readiness Assessment

Status: READY FOR IMPLEMENTATION

Reason:

- the control plane is explicit
- boundaries are narrow
- `task_contract` has a fixed architectural place
- persistence and evidence responsibilities are split clearly
- the Rust structure is small enough to start immediately

## Final Architectural Decisions

- Use a layered monolith, not multiple services.
- Use one Rust crate with modules, not a multi-crate workspace in V1.
- Keep domain pure; all IO goes through application ports into infrastructure adapters.
- Treat `Task` as domain intent and `task_contract` as application execution spec.
- Make SQLite authoritative for runtime state and filesystem authoritative for heavy evidence artifacts.
- Make the runtime the sole owner of state transitions and hard-stop decisions.
- Hardcode the V1 skill catalog and role policies; do not add a plugin mechanism.
- Keep resume support minimal and fail closed on workspace drift.

## First Implementation Priority

Start by implementing the pure domain model, the state machine rules, and a test-driven `SessionRunner` with fake ports. That is the smallest slice that proves Continuum is a runtime and not just a document set.
