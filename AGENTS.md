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
