# Continuum

> This repository documents an experimental system. See "Current Limitations" before using.

## Overview

At this stage, Continuum is essentially a controlled wrapper around `codex exec` for a single-file use case (`README.md`).

The only capability that has actually been proven is README generation via the Builder connected to Codex, with a strictly limited write scope.

The current binary runs locally via `continuum "<prompt>"`, resolves the current repo via `current_dir()`, launches a single run via `SessionRunner`, then terminates in success or failure.

In the current state of the repo, most of the actual work is delegated to Codex, and is not performed by Continuum itself.

The Scholar, Planner, and Critic roles exist in the code but do not yet correspond to actually operational agents.

The system must not be interpreted as a functional agentic orchestrator.

The repo contains more architectural structure than actually proven capabilities.

## Current Capabilities

What is proven today, and nothing more:

- `cargo build` works.
- The `continuum` binary runs locally.
- The CLI accepts a single non-empty prompt.
- The current repo is resolved via `current_dir()`.
- `SessionRunner` orchestrates the current run.
- The `Builder` is connected to `codex exec`.
- The prompt passed to Codex is bounded by Continuum.
- The write scope is explicitly controlled.
- The currently admitted scope is limited to `README.md`.
- The system captures `stdout` and `stderr` from the `codex` process.
- The `Builder` returns a `BuilderRunReport` exploitable by the runtime.
- If the `Builder` does not return `Completed`, the runtime stops terminally.
- A real run generated a `README.md`.
- An out-of-scope mission fails with `precondition_failed`.

Minimum currently useful command:

```bash
continuum "Generate the README.md for this repository. Modify only README.md."
```

## Current Limitations

The current limitations are structural, major, and not optional:

- `Scholar` is a minimal shell.
- `Planner` is a minimal shell.
- `Critic` is a minimal shell.
- Only the `Builder` is actually connected to a local external engine.
- This connection only covers a single-file use case centered on `README.md`.
- The current system is not a real operational multi-agent.
- The current system must not be read as a general orchestrator of development tasks.
- There is no proven support for general tasks outside the README path.
- There is no persistent Codex session.
- There is no natural multi-turn prompting.
- There is no parallel orchestration.
- There are no worktrees.
- There is no required SQLite dependency for this current real path.
- The current CLI is a minimal shell, not a final product CLI.
- The product is not ready for production.
- The target architecture is far from implemented.

## What This Is

Continuum is today a very bounded local experimental runtime.

In its current state, Continuum acts mainly as a controller, not as an autonomous intelligent agent.

In its current state, it serves to validate a first real bridge between:

- a minimal CLI
- `SessionRunner` as the runtime orchestrator
- a `Builder` connected to `codex exec`

Its current value is experimental and architectural:

- prove that a real local run can be executed
- verify that a strict write scope can be enforced
- observe the real limits of the system in contact with a local external engine

## What This Is NOT

This project is not:

- a functional agentic orchestrator
- an operational multi-agent system
- a complete integration of Codex into Continuum
- a system proven capable of handling general tasks
- a mature product
- a system ready for production
- a complete product CLI

If you are looking for a complete agentic runtime, this repo does not provide that yet.

## Architecture Target

This section describes the architectural target of the project. It does not describe the currently proven real state of the repo.

The target defined in the planning artifacts is a bounded local runtime with a `Scholar -> Planner -> Builder -> Critic` pipeline, orchestrated by `SessionRunner`, with explicit decisions, guardrails, retry budget, terminal artifacts, and prescriptive critique.

This target is not implemented end to end today.

More precisely:

- the full pipeline exists as code structure and as architectural intent
- the only real bridge to a local external engine concerns the `Builder` today
- `Scholar`, `Planner`, and `Critic` must not be interpreted as actually operational agents

## First Real Run

The first real run proven at this stage is a README run.

Valid path:

- `cargo build`
- local execution of the `continuum` binary
- single prompt bounded to the generation of `README.md`
- resolution of the current repo via `current_dir()`
- orchestration of the run by `SessionRunner`
- invocation of `codex exec` by the `Builder`
- write scope strictly bounded to `README.md`
- clear terminal output in success or failure

This run proves a real `Builder -> Codex` bridge.

It does not prove:

- complete multi-agent orchestration
- a general capability to solve tasks
- a complete integration of Codex into the system

Example of failure currently expected:

- if the mission does not allow an explicitly bounded scope on `README.md` to be determined, the `Builder` fails with `precondition_failed`
