# Refactor Structure Step 7 Deblock Mandate

## 1. Statut

Artefact complementaire de deblocage.

Il complete:

- `project-directives/implementation-artifacts/refactor-structure-mandate.md`
- `project-directives/implementation-artifacts/refactor-structure-step-4-deblock-mandate.md`
- `project-directives/implementation-artifacts/refactor-structure-step-5-deblock-mandate.md`

Il ne remplace pas le mandate principal en entier.

Il remplace uniquement l'etape 7 du `refactor-structure-mandate.md`.

Toutes les autres etapes du mandate principal restent inchangees.

## 2. Preconditions obligatoires confirmees

Etat canonique a considerer avant execution de la presente correction:

- `HandoffDecision` est stable en `domain`
- `SessionFlowDecision` est stable en `application`
- `application::actors::{Scholar, Planner, Builder, Critic}` existent
- `MissionScholar` est extrait dans `src/application/scholar.rs`
- `ScopePlanner` est extrait dans `src/application/planner.rs`
- `WorkflowState`, `StateMachine` et `AgentRole` sont extraits dans `src/application/workflow.rs`

## 3. Etat reel du repo a integrer

Etat observe apres tentative terrain de l'etape 7:

- l'extraction mecanique de `SessionSummary`, `FailureReport` et `SessionRunner` hors de `src/lib.rs` vers `src/application/session_runner.rs` a ete appliquee correctement
- `src/application/mod.rs` et `src/lib.rs` ont ete modifies correctement
- `session_runner_happy_path_tests` est vert
- `session_runner_failure_tests` est vert
- les trois tests E2E runtime encore listes dans l'etape 7 echouent a la compilation

Cause constatee:

- `tests/e2e/session_e2e_happy_path_tests.rs` importe encore `BuilderPort`, `CriticPort`, `PlannerPort`, `ScholarPort`
- `tests/e2e/session_e2e_stop_tests.rs` importe encore `BuilderPort`, `CriticPort`, `PlannerPort`, `ScholarPort`
- `tests/e2e/session_e2e_with_data_tests.rs` importe encore `BuilderPort`, `CriticPort`, `PlannerPort`, `ScholarPort`

Conclusion obligatoire:

- l'extraction de production de l'etape 7 est correcte
- l'etape 7 n'est pas cloturable en vert telle qu'ecrite, car le mandate sous-estime encore le perimetre mecanique reel des tests runtime en scope
- la cloture de l'etape 7 doit autoriser un realignement minimal des trois tests E2E runtime listés ci-dessus, sans toucher a leur logique

## 4. Regles absolues de cette correction

- refactor mecanique uniquement
- zero changement de comportement
- zero nouvelle feature
- aucun redesign global
- aucun renommage supplementaire au-dela du remplacement des anciens noms `*Port` par les traits runtime deja stabilises
- aucune nouvelle regle metier
- aucune nouvelle structure de donnees metier
- aucune modification de signature publique non prevue
- ne pas rouvrir les decisions deja tranchees sur le double appel planner, le retry budget, ou le contrat `Builder` / `Critic`
- ne pas toucher a `SessionRunner` de production dans cette correction
- ne pas toucher a `MissionScholar`
- ne pas toucher a `ScopePlanner`
- ne pas toucher a `HandoffDecision`
- ne pas toucher a `SessionFlowDecision`
- ne pas toucher a `application::actors::*`
- ne pas toucher a `application::workflow::*`
- ne pas ajouter d'alias de compatibilite ambigu
- ne pas toucher a d'autres tests que ceux explicitement listes ci-dessous

## 5. Etape 7 corrigee et cloturable

### 5.1 But exact

But de l'etape corrigee:

- reconnaitre que l'extraction de production de `SessionSummary`, `FailureReport` et `SessionRunner` est deja correcte
- autoriser uniquement la cloture mecanique restante de l'etape 7
- realigner minimalement les trois tests E2E runtime deja en scope pour qu'ils compilent contre les traits runtime stabilises
- obtenir les cinq tests listés par l'etape 7 au vert sans changer la logique de production ni la logique de tests

### 5.2 Fichiers autorises pour la cloture de l'etape 7

Fichiers deja extraits correctement et a ne pas retoucher sauf reference residuelle strictement necessaire a la compilation:

- `src/application/session_runner.rs`
- `src/application/mod.rs`
- `src/lib.rs`

Fichiers a modifier pour cloturer l'etape 7:

- `tests/e2e/session_e2e_happy_path_tests.rs`
- `tests/e2e/session_e2e_stop_tests.rs`
- `tests/e2e/session_e2e_with_data_tests.rs`

Interdiction explicite:

- ne toucher a aucun autre fichier

### 5.3 Realignements autorises dans les tests E2E runtime en scope

Realignements autorises uniquement dans les trois fichiers listés ci-dessus:

- remplacer les imports racine `ScholarPort`, `PlannerPort`, `BuilderPort`, `CriticPort`
- importer a la place `continuum::application::actors::{Scholar, Planner, Builder, Critic}`
- realigner les `impl` de doubles de test pour implementer `Scholar`, `Planner`, `Builder`, `Critic`
- remplacer les annotations de type qui pointent encore sur les anciens noms `*Port`

Interdictions explicites dans ces tests:

- ne pas changer la logique des tests
- ne pas changer les assertions
- ne pas changer les donnees de fixture
- ne pas changer les decisions runtime `SessionFlowDecision`
- ne pas changer l'ordre d'activation attendu
- ne pas toucher a `SessionRunner`, `SessionSummary` ou `FailureReport` de production

### 5.4 Tests a rejouer

Rejouer uniquement:

- `cargo test --test session_runner_happy_path_tests`
- `cargo test --test session_runner_failure_tests`
- `cargo test --test session_e2e_happy_path_tests`
- `cargo test --test session_e2e_stop_tests`
- `cargo test --test session_e2e_with_data_tests`

### 5.5 Conditions d'arret

Arret immediat si:

- le realignement exige autre chose qu'une mise a jour mecanique des imports, `impl`, ou annotations de type dans les trois fichiers de tests autorises
- l'etape exige de modifier la logique de production de `SessionRunner`
- l'etape exige de rouvrir une decision sur le double appel planner
- l'etape exige de rouvrir une decision sur le retry budget
- l'etape exige de rouvrir une decision sur le contrat `Builder` / `Critic`
- l'etape exige de toucher un fichier autre que les trois fichiers de tests autorises ci-dessus, hors references residuelles deja correctes de production
- l'etape exige une compatibilite racine pour conserver `ScholarPort`, `PlannerPort`, `BuilderPort` ou `CriticPort`
- un des cinq tests listés echoue pour une raison non liee au realignement mecanique des anciens noms `*Port` dans les trois tests E2E en scope

### 5.6 Sortie attendue de l'etape

Sortie attendue si l'etape est executee correctement:

- `src/application/session_runner.rs` reste la source canonique de `SessionRunner`, `SessionSummary` et `FailureReport`
- `src/application/mod.rs` et `src/lib.rs` restent alignes avec cette extraction
- les trois tests E2E runtime en scope compilent contre `continuum::application::actors::{Scholar, Planner, Builder, Critic}`
- aucun alias de compatibilite racine n'a ete ajoute
- les cinq tests listés par l'etape 7 sont verts

## 6. Effet sur le mandate principal

Pour la suite du chantier:

- l'etape 7 du `refactor-structure-mandate.md` doit etre lue a travers le present artefact
- l'extraction de production de `SessionRunner` est consideree comme correcte et non remise en question
- la cloture de l'etape 7 inclut explicitement le realignement minimal des trois tests E2E runtime en scope
- les etapes 1, 2, 3, 4, 5, 6 et 8 du mandate principal restent inchangees
- aucune extension de scope n'est autorisee au-dela de ce qui est ecrit ici
