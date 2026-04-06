# Runtime Entrypoint Infrastructure Regressions

## Regression 1

- contexte:
  - extraction de la composition CLI minimale hors de `main.rs`
- responsabilite extraite concernee:
  - lecture/validation de l'argument prompt et resolution du repo courant
- test(s) impacte(s):
  - `wsl cargo test --test cli_minimal_runtime_shell_tests --test session_runner_happy_path_tests --test session_runner_failure_tests --test runtime_stop_interception_tests --test runtime_planner_revision_tests --test runtime_critic_signal_tests --test session_e2e_happy_path_tests --test session_e2e_stop_tests --test session_e2e_with_data_tests`
  - `wsl cargo test`
- cause:
  - suppression accidentelle de l'import `RawMission` encore requis par `ShellScholar`
- correction appliquee:
  - restauration de l'import `RawMission` dans `src/main.rs`
- decision prise:
  - conserver l'extraction CLI et corriger l'import
- pourquoi cette decision reste conforme au mandate:
  - la regression ne concernait pas le protocole terminal ni la semantique du run reel
  - la responsabilite CLI reste correctement separee de `main.rs`
