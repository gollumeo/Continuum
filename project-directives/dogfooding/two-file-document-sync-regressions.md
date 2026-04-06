# Two-File Document Sync Regressions

## Regression 1

- contexte: baby step introduisant un repo git temporaire pour le test CLI two-file et le premier critic borne sur l'existence des deux documents canoniques
- test(s) impacte(s): `tests/cli/cli_minimal_runtime_shell_tests.rs` a la compilation, avant execution de `runs_two_file_document_sync_scope_on_repo_with_both_canonical_files`
- cause reelle: `repo_root` etait deplace dans `Command::current_dir(...)` puis relu plus bas dans les assertions terminales
- correction: passage de `&repo_root` a `current_dir(...)` pour conserver le chemin utilisable dans les assertions
- decision prise: corriger localement le test sans toucher au runtime ni aux helpers du use case
- pourquoi cela reste conforme au mandate: la correction est strictement locale au test courant et n'introduit aucune generalisation runtime ou documentaire
