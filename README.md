# Continuum

> This repository documents an experimental system. See "Current Limitations" before using.

## Overview

Continuum is currently a bounded local runtime around `codex exec` with a small set of repository-proven use cases.

The repo no longer proves only a single `README.md` generation path. It now proves a few exact, tightly-scoped runtime paths:

- one-file `README.md` generation with explicit scope
- two-file documentation synchronization between `README.md` and `project-directives/index.md`
- an exact increment-contract repair use case limited to `src/lib.rs`
- an exact increment-contract repair use case limited to `src/lib.rs` with an additional zero-confirmation check

The system is still not a general agentic orchestrator. Most real work is still delegated to `codex exec`, and the admitted task surface remains narrow and explicit.

## Current Capabilities

What is proven today, and nothing more:

- `cargo build` works.
- The `continuum` binary runs locally.
- The CLI accepts exactly one non-empty prompt argument.
- The current repository root is resolved via `current_dir()`.
- `main` builds a local shell runtime and runs a single `SessionRunner` session.
- `SessionRunner` executes a bounded `Scholar -> Planner -> Builder -> Critic -> Planner` flow.
- The runtime can complete in one pass or stop terminally on failure.
- The runtime can perform one bounded retry for the currently admitted retrying use cases.
- The `Builder` is connected to `codex exec`.
- The `Builder` sends a bounded prompt that includes an explicit allowed file scope.
- The `Builder` captures `stdout` and `stderr` from `codex` and returns a `BuilderRunReport`.
- The `Builder` checks the resulting file changes against the allowed scope using git status.
- The local shell critic runs exact proof commands for the admitted increment-contract use cases.
- The local shell critic performs a bounded content/existence check for the admitted two-file documentation sync use case.
- The CLI renders terminal success and failure output including session status and builder report data.

## Exact Proven Use Cases

The repo currently proves these concrete prompt paths:

```text
Generate the README.md for this repository. Modify only README.md.
```

```text
Synchronize README.md and project-directives/index.md. Modify only README.md and project-directives/index.md.
```

```text
Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs.
```

```text
Make the failing test 'increment_adds_one_to_input' in tests/increment_contract.rs pass by editing only src/lib.rs, and confirm 'increment_adds_one_to_zero' in tests/increment_contract.rs also passes.
```

These proved paths include the following exact bounded behavior:

- `README.md` generation is admitted only with explicit file scope.
- an underspecified `Generate the README.md for this repository.` prompt is refused before builder launch.
- two-file synchronization is admitted only when both canonical files are allowed together.
- the single increment-contract fix is bounded to `src/lib.rs` and proved by running `cargo test --test increment_contract increment_adds_one_to_input -- --exact`.
- the zero-confirmation increment use case is bounded to `src/lib.rs` and proved by running the `increment_adds_one_to_input` command first and then `increment_adds_one_to_zero`.
- the increment use cases retry exactly once when the critic requests revision and stop after budget exhaustion.

## Runtime Shape

The current runtime is small but real:

- `MissionScholar` currently copies the mission text into `ScholarOutput`.
- the shell planner admits build execution for the currently proved bounded prompts and refuses the underspecified README prompt.
- `CodexLocalBuilderAdapter` launches `codex exec` from the repository root with an explicit allowed file scope.
- `ShellCritic` is local and exact-case-driven: it either runs the proved cargo commands for increment-contract prompts or validates the two-file documentation sync result.
- runtime stop and retry handling is routed through `runtime_policy`.
- exact increment runtime-use-case selection is centralized in `runtime_use_case_authority`.

## Current Limitations

The current limitations remain structural and important:

- This is not a general development agent.
- Prompt admission is narrow and repository-specific.
- The documentation use cases are still tied to exact bounded prompt shapes and exact file scopes.
- The increment-contract paths are still tied to exact prompts, exact proof commands, and exact file scope `src/lib.rs`.
- `Scholar` is still a minimal shell over `MissionScholar`.
- `Planner` is still a minimal shell with bounded runtime decisions, not a general planning agent.
- `Critic` is not a general reviewer; it only performs the exact local checks currently proved by tests.
- Only the `Builder` is connected to a local external engine.
- There is no persistent Codex session.
- There is no natural multi-turn runtime.
- There is no parallel orchestration.
- There are no worktrees.
- The CLI is still a minimal shell, not a final product CLI.
- The product is not ready for production.
- The repo still contains more architecture than generally proven capability.

## What This Is

Continuum is currently a bounded local experimental runtime used to validate a few exact orchestration paths:

- a minimal CLI
- `SessionRunner` as the runtime coordinator
- a `Builder` bridged to `codex exec`
- a small set of explicit planner and critic rules around proven repository use cases

Its current value is practical but narrow:

- prove that a local builder run can be executed safely under explicit file scope
- prove that the runtime can stop, retry once, and render terminal artifacts predictably
- prove a few exact repository tasks end to end without claiming general task execution

## What This Is NOT

This project is not:

- a general agentic orchestrator
- an operational multi-agent system
- a general task runner for arbitrary repo work
- a complete Codex integration
- a mature product
- a production-ready system
- a final product CLI

## Architecture Target

The architectural target remains larger than the currently proved runtime.

The repo is still organized around a `Scholar -> Planner -> Builder -> Critic` pipeline orchestrated by `SessionRunner`, with explicit flow decisions, retry policy, terminal outcomes, and bounded runtime authority.

That target is still only partially realized:

- the runtime pipeline is real and tested
- the builder bridge to `codex exec` is real and tested
- the critic has a few real exact-case checks
- the scholar and planner layers are still minimal compared to the architectural target

The repository should therefore be read as a tightly-bounded experimental runtime, not as a fully implemented autonomous system.
