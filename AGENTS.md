# AGENTS.md

This file codifies only rules already justified in this repository by the current dogfooding mandates and the existing CLI, runtime, e2e, and session runner tests. It does not add doctrine beyond those cases.

## 1. Mission and Non-Generalization Rule

- Implement only the exact prompt, exact file scope, exact command, exact refusal, and exact terminal outcome proved by the current repository use case.
- Reject any renamed, parameterized, shared, table-driven, pattern-matched, or otherwise broadened form of an exact prompt, exact file scope, exact command, or exact refusal.
- Reject any change that makes any nearby prompt, extra file, extra command, or repo-wide workflow work without a current failing repository test for that exact addition.
- Reject any change to `SessionRunner` or the terminal protocol unless a current failing repository test fails on that exact surface and no smaller local change can make that test pass.

## 2. TDD Evidence Rule

- Reject any code, branch, structure, or admission rule added before a current failing repository test exists for that exact case and before that exact case is selected by an accepted repository use case or mandate.
- Reject any structure, split, abstraction, intermediate type, or preparatory path unless one current failing repository test requires it and that test cannot pass with a smaller local change.
- Implement only the smallest local change that makes the current red test pass; reject any change that also prepares an unproven second case or follows a test that extends capability beyond the selected use case.
- When the exact active use case is green, stop; reject any further change not required to keep current repository tests green.
- When `Cargo.toml` registers integration tests through explicit `[[test]]` entries, adding one exact newly-authorized repository proof may include adding the single matching `[[test]]` entry required to execute that proof; this does not authorize dependency changes or broader test-surface expansion.

## 3. Driving vs Non-Driving Test Distinction

- Treat closure-level tests as driving tests only when they prove an exact case already selected by an accepted repository use case or mandate. In this repository, that is usually `tests/cli/**`; use runtime or e2e as driving tests only when they are the smallest existing surface that can expose the failing case for that already-selected case.
- Classify any test not traceable to an accepted repository use case or mandate as non-driving; reject any local, runtime, e2e, or CLI test that introduces behavior not already demanded by a selected use case.
- Reject any new seam, helper, intermediate abstraction, split branch, or extracted layer unless the current failing repository test cannot pass without it, no smaller local change can make that test pass, and the justification is not cleanliness, symmetry, naming, or anticipated reuse.
- Reject any duplicate coverage when an existing repository test already proves the exact prompt, exact file scope, exact command, exact refusal, or exact terminal outcome.

## 4. Trial-and-Error Boundary

- Use retry only where current runtime semantics already allow retry for the exact case under test; reject any new retry path not already justified by repository evidence.
- Reject any retry used to guess intent, widen scope, discover tests, discover files, discover commands, or simulate a generic revision engine.
- For underspecified prompts, stop before `Builder`; any `Builder` launch, `codex exec` launch, retry consumption, or repository side effect is a violation.
- If making the next test pass would require generic prompt understanding, generic file-set handling, generic command sequencing, a critic framework, or runtime redesign, stop and ask for arbitration.

## 5. Automatic Rejection Conditions

- Reject any prompt classification implemented as a family, heuristic, score, taxonomy, pattern list, or renamed equivalent instead of one exact repository-proven case.
- Reject any multi-file support that admits any file set beyond the exact named files already proven by repository tests.
- Reject any code-fixing path that can target any failing test, source file, or proof command beyond the exact named case already proven by repository tests.
- Reject any command handling represented as a list, map, pair, pipeline, table, loop, sequence abstraction, or renamed equivalent when the use case proves only exact named commands.
- Reject any repository test used as implementation authority when that test is not traceable to an accepted repository use case, mandate, or previously accepted tension.
- Reject any repo-wide validation when the active use case is bounded to exact named files or exact named commands.
- Reject any write outside the exact allowed file scope named by the current repository use case.
- Reject any implementation that reaches `Builder` in a use case already proven by repository tests to require pre-build refusal.
- Reject any test that extends prompt admission, file scope, command scope, or runtime surface beyond an exact selected use case or mandate.
- Reject any terminal or runtime surface change not named by a current failing repository test, not selected by an accepted repository use case or mandate, and not impossible to avoid with a smaller local change.

## 6. Evidence-Reporting Obligation When Requested

- Report the exact prompt string that was admitted or refused.
- Report the exact allowed file scope string.
- Report the exact command or exact commands that were executed, in exact order, including any command that was skipped by rule.
- Report the exact driving repository test that proves the behavior, and any narrower locking tests only if they constrain already-proven behavior.
- Report whether the run completed, retried, or stopped, and the exact number of attempts consumed.
- Report the exact files changed.
- Do not claim any capability unless that exact prompt, exact file scope, exact command, exact refusal, and exact terminal outcome are proved by repository evidence.

## 7. Transverse Coherence Rule

- Evaluate every local change against repository-wide coherence before accepting it, including: layer coherence (`domain` / `application` / `infrastructure`), responsibility coherence, public surface coherence in `lib.rs`, repository geography coherence, and proof-surface coherence across tests.
- Reject any local change that passes the current test but introduces a cross-repository incoherence in layering, responsibility placement, public exports, file geography, or test-surface alignment.
- Reject any local change that makes one repository area more coherent only by making another area less coherent.
- If a local change introduces a transverse incoherence and no accepted repository mandate explicitly authorizes that repository-wide change, stop immediately and report the exact tension.

## 8. Non-Fragmentation Rule

- Reject any change that improves one local case at the price of degrading the repository's global coherence.
- Reject any "minimal slice" that is locally correct but leaves the repository with a new divergence of structure, responsibility, public surface, or proof organization.
- Treat repository-scale consistency as a validity condition of every local decision, not as optional cleanup for later.
- Do not accept a local slice as complete unless it is valid both for the current exact test and for the current repository structure as a whole.

## 9. Structural Tension Handling Rule

- If a structural incoherence is detected while executing a local change, stop immediately in execution mode and report the exact incoherence instead of partially correcting it.
- If the user is explicitly working in architecture or mandate-design mode, convert the detected incoherence into a proposal for a separate repository mandate rather than fixing it opportunistically.
- Reject any partial structural fix that would leave the repository in a half-migrated or locally special-cased state.
- Reject any hidden scope expansion used to "make the problem go away" when the real issue is a repository-level structural tension.

## 10. Strict Change Typology Rule

- Classify every requested change as exactly one of the following and only one: structural refactor (repository geography only), doctrinal refactor (modeling, naming, or responsibility boundaries), or functional extension (new behavior).
- Reject any implementation that mixes these categories inside one change unless an accepted repository mandate explicitly authorizes that exact mixed scope.
- For structural refactor work, reject any logic change, naming change, responsibility redesign, or behavioral extension.
- For doctrinal refactor work, reject any opportunistic behavior change or repository-geography rewrite not required by the accepted mandate.
- For functional extension work, reject any unmandated structural cleanup or doctrinal correction performed "while here".
- If completing the requested work would require changing category, stop and ask for arbitration or a separate mandate.

## 11. Slice Drift Prevention and Transverse Validation Rule

- Allow local slices only when they preserve repository-wide coherence and do not introduce a new implicit pattern, a new unstated convention, or a new divergence of structure or responsibility.
- Reject any slice that teaches the repository a new pattern, naming convention, layer exception, export rule, or test-placement rule without an accepted repository-level mandate for that exact convention.
- Accept a change only if it can be described as either: (1) a local improvement with no repository-wide incoherence, or (2) a repository-level change explicitly mandated at repository scope.
- If a change cannot be described in one of those two ways, reject it.

## 12. Demonstrable Coherence Requirement

- A change is considered coherent only if the agent can explicitly demonstrate, before applying it, that it preserves: layer boundaries (`domain` must not depend on `application` or `infrastructure`); responsibility isolation (no file gains a second unrelated responsibility); public surface stability (no unintended change in `lib.rs` exposure); repository geography consistency (files remain aligned with their layer purpose); and proof-surface alignment (tests still reflect the same scope and intent).
- If coherence cannot be explicitly demonstrated on all these axes, the change must be rejected.
- `Seems consistent`, `likely coherent`, or `no obvious issue` are invalid justifications.

## 13. Default Deny Rule

- Any change that is not explicitly allowed by: a current failing repository test, and an accepted repository use case or mandate, and all coherence rules in this document, must be rejected.
- If there is uncertainty about whether a change is allowed, coherent, or within scope, the agent must stop and report instead of proceeding.
- Absence of explicit prohibition does not imply permission.

## 14. Pre-Action Justification Constraint

- Before applying any non-trivial change, the agent must be able to state: which exact rule allows the change, which exact test requires it, and why no smaller change would satisfy the same constraint.
- If this justification cannot be stated clearly and concretely before the change, the change must not be applied.
- Post-hoc justification is invalid.

## 15. Atomic Commit Rule

- When commit creation is explicitly requested or otherwise already authorized in the current conversation, each commit must be atomic and contain only one exact selected slice, one exact proof-driven correction, or one exact repository-level mandate change.
- Reject any commit that batches multiple independently reviewable changes, multiple exact cases, multiple story slices, or mixed code-and-doctrine work unless an accepted repository mandate explicitly authorizes that exact mixed scope.
- If the current work contains more than one exact change, split it into separate commits aligned to the smallest coherent review unit instead of producing one aggregate commit.
- Commit size is not the rule; atomicity is. A large commit is acceptable only when every changed line is required for one exact authorized slice and no smaller commit boundary can preserve correctness and proof coherence.
- This rule governs commit boundaries only. It does not authorize creating commits when commits were not explicitly requested.

## 16. TDD Commit Granularity Rule

- Each red→green cycle for one exact targeted test constitutes one atomic commit boundary.
- A commit may not be created before the targeted test has been proven red first.
- A commit may not batch more than one red→green transition unless a single implementation change simultaneously satisfies two tests already proven red.
- Reject any commit that groups a red proof, an implementation, and a green proof across more than one targeted test.
- Stop and ask for arbitration if making the next test green would require touching a second test's scope.