# Refactor Structure Step 5 Deblock Mandate

## 1. Statut

Artefact complementaire de deblocage.

Il complete:

- `project-directives/implementation-artifacts/refactor-structure-mandate.md`
- `project-directives/implementation-artifacts/refactor-structure-step-4-deblock-mandate.md`
- `project-directives/implementation-artifacts/slice-2-implementation-mandate.md`
- `project-directives/planning-artifacts/architecture.md`

Il ne remplace pas le mandate principal en entier.

Il remplace uniquement l'etape 5 du `refactor-structure-mandate.md`.

Toutes les autres etapes du mandate principal restent inchangees.

## 2. Preconditions obligatoires confirmees

Etat canonique a considerer avant execution de la presente etape:

- `application::actors::{Scholar, Planner, Builder, Critic}` existent
- `SessionRunner` depend deja de ces traits runtime
- `HandoffDecision` est stable en `domain`
- `SessionFlowDecision` est stable en `application`
- `PlannerDecision` a disparu
- `RuntimeProceedStopDecision` a disparu

## 3. Etat reel du repo a integrer

Etat observe au moment de reprendre l'etape 5 du mandate principal:

- `MissionScholar` expose seulement `new() -> Self` et `transform(&RawMission) -> ScholarOutput`
- `ScopePlanner` expose seulement `new() -> Self` et `decide(&ScholarOutput) -> HandoffDecision`
- `application::actors::Scholar` exige `fn run(&mut self) -> ScholarOutput`
- `application::actors::Planner` exige `fn decide(&mut self, &ScholarOutput) -> SessionFlowDecision`

Conclusion obligatoire:

- `MissionScholar` n'est pas une implementation mecanique du port runtime `application::actors::Scholar`
- `ScopePlanner` n'est pas une implementation mecanique du port runtime `application::actors::Planner`
- l'etape 5 du mandate principal repose sur une hypothese invalide si elle exige ces implementations

## 4. Decision canonique sur la nature de `MissionScholar` et `ScopePlanner`

Decision explicite:

- `MissionScholar` et `ScopePlanner` sont des concretes applicatives amont du handoff slice 2
- ils ne sont pas des concretes runtime pour `SessionRunner`
- ils ne doivent pas implementer `application::actors::Scholar`
- ils ne doivent pas implementer `application::actors::Planner`

Interpretation autorisee:

- `MissionScholar` est une concrete amont qui transforme `RawMission` en `ScholarOutput`
- `ScopePlanner` est une concrete amont qui lit `ScholarOutput` et retourne `HandoffDecision`
- ces deux types restent dans `application/` car ils appartiennent au flux applicatif amont, mais ils sont distincts des ports runtime

Interdictions explicites:

- ne pas projeter `HandoffDecision` vers `SessionFlowDecision` dans cette etape
- ne pas ajouter `run()` a `MissionScholar`
- ne pas ajouter une implementation de `application::actors::Planner` a `ScopePlanner`
- ne pas introduire de compatibilite magique entre handoff amont et runtime

## 5. Regles absolues de cette correction

- refactor mecanique uniquement
- zero changement de comportement
- zero nouvelle feature
- aucun redesign global
- aucun renommage supplementaire
- aucune nouvelle regle metier
- aucune nouvelle structure de donnees metier
- aucune modification de signature publique non prevue
- ne pas toucher a `SessionRunner`
- ne pas toucher au workflow
- ne pas toucher a `HandoffDecision`
- ne pas toucher a `SessionFlowDecision`
- ne pas toucher a `application::actors::*`
- aucun re-export racine plat pour `MissionScholar` ou `ScopePlanner`
- ne pas ajouter d'alias de compatibilite ambigu
- ne pas etendre le scope au-dela des fichiers explicitement listes ci-dessous

## 6. Etape 5 corrigee et executable

### 6.1 But exact

But de l'etape corrigee:

- extraire les concretes applicatives amont inline hors de `src/lib.rs`
- placer `MissionScholar` dans `src/application/scholar.rs`
- placer `ScopePlanner` dans `src/application/planner.rs`
- conserver leur comportement et leurs signatures amont exactes
- realigner minimalement les tests amont en scope pour qu'ils compilent contre les chemins modules explicites sans re-export racine plat

### 6.2 Fichiers autorises pour cette etape

Fichiers a creer:

- `src/application/scholar.rs`
- `src/application/planner.rs`

Fichiers a modifier:

- `src/application/mod.rs`
- `src/lib.rs`
- `tests/domain/scholar_tests.rs`
- `tests/domain/planner_tests.rs`
- `tests/e2e/session_e2e_scholar_planner_handoff_tests.rs`

Interdiction explicite:

- ne toucher a aucun autre fichier

### 6.3 Deplacements autorises

Deplacements autorises uniquement:

- deplacer la definition inline de `MissionScholar` hors de `src/lib.rs` vers `src/application/scholar.rs`
- deplacer l'impl inline de `MissionScholar` hors de `src/lib.rs` vers `src/application/scholar.rs`
- deplacer la definition inline de `ScopePlanner` hors de `src/lib.rs` vers `src/application/planner.rs`
- deplacer l'impl inline de `ScopePlanner` hors de `src/lib.rs` vers `src/application/planner.rs`

Limite stricte:

- aucun autre deplacement n'est autorise dans cette etape

### 6.4 Contenu autorise de `src/application/scholar.rs`

`src/application/scholar.rs` doit contenir uniquement:

- `MissionScholar`
- `impl MissionScholar`

Contraintes obligatoires:

- conserver `new() -> Self`
- conserver `transform(&RawMission) -> ScholarOutput`
- conserver exactement le comportement actuel de transformation
- ne pas implementer `application::actors::Scholar`
- ne pas introduire de nouvel etat
- ne pas introduire de nouvelle methode

### 6.5 Contenu autorise de `src/application/planner.rs`

`src/application/planner.rs` doit contenir uniquement:

- `ScopePlanner`
- `impl ScopePlanner`

Contraintes obligatoires:

- conserver `new() -> Self`
- conserver `decide(&ScholarOutput) -> HandoffDecision`
- conserver exactement le comportement actuel de decision
- ne pas implementer `application::actors::Planner`
- ne pas projeter `HandoffDecision` vers `SessionFlowDecision`
- ne pas introduire de nouvelle methode

### 6.6 Contenu autorise de `src/application/mod.rs`

`src/application/mod.rs` doit seulement:

- declarer `pub mod scholar;`
- declarer `pub mod planner;`
- conserver `actors` et `session_flow_decision`
- ne pas ajouter de re-export racine plat

### 6.7 Contenu autorise de `src/lib.rs`

`src/lib.rs` peut etre modifie uniquement pour:

- retirer les definitions inline de `MissionScholar`
- retirer les definitions inline de `ScopePlanner`
- realigner les imports internes necessaires vers `crate::application::scholar::MissionScholar` et `crate::application::planner::ScopePlanner` uniquement si cela est strictement necessaire a la compilation du fichier

Interdictions explicites dans `src/lib.rs`:

- ne pas re-exporter `MissionScholar` a la racine
- ne pas re-exporter `ScopePlanner` a la racine
- ne pas toucher a `SessionRunner`
- ne pas toucher aux traits runtime `Scholar`, `Planner`, `Builder`, `Critic`
- ne pas ajouter d'alias de compatibilite pour conserver `continuum::MissionScholar` ou `continuum::ScopePlanner`

### 6.8 Contenu autorise du realignement minimal des tests en scope

Le realignement des tests n'est autorise que pour les trois fichiers suivants:

- `tests/domain/scholar_tests.rs`
- `tests/domain/planner_tests.rs`
- `tests/e2e/session_e2e_scholar_planner_handoff_tests.rs`

Changements autorises dans ces tests uniquement:

- remplacer les imports racine `MissionScholar` et `ScopePlanner`
- importer a la place `continuum::application::scholar::MissionScholar`
- importer a la place `continuum::application::planner::ScopePlanner`
- conserver les imports racine de `RawMission`, `ScholarOutput`, `HandoffDecision` si necessaire

Interdictions explicites dans ces tests:

- ne pas changer la logique des tests
- ne pas changer les assertions
- ne pas modifier les decisions amont
- ne pas toucher a d'autres tests

### 6.9 Tests a rejouer

Rejouer uniquement:

- `cargo test --test scholar_tests`
- `cargo test --test planner_tests`
- `cargo test --test session_e2e_scholar_planner_handoff_tests`

### 6.10 Conditions d'arret

Arret immediat si:

- l'extraction force autre chose qu'un deplacement mecanique
- l'etape exige de faire implementer `MissionScholar` par `application::actors::Scholar`
- l'etape exige de faire implementer `ScopePlanner` par `application::actors::Planner`
- l'etape exige d'ajouter `run()` a `MissionScholar`
- l'etape exige de mapper `HandoffDecision` vers `SessionFlowDecision`
- l'etape exige de toucher un fichier autre que les six fichiers autorises
- l'etape exige une compatibilite racine pour conserver `continuum::MissionScholar` ou `continuum::ScopePlanner`
- un des tests listes echoue pour une raison non liee au deplacement mecanique et au realignement des imports de tests en scope

### 6.11 Sortie attendue de l'etape

Sortie attendue si l'etape est executee correctement:

- `src/application/scholar.rs` existe et porte `MissionScholar`
- `src/application/planner.rs` existe et porte `ScopePlanner`
- `src/lib.rs` ne definit plus inline `MissionScholar` ni `ScopePlanner`
- `MissionScholar` reste une concrete amont de transformation `RawMission -> ScholarOutput`
- `ScopePlanner` reste une concrete amont de decision `ScholarOutput -> HandoffDecision`
- aucun re-export racine plat n'a ete ajoute pour ces deux concretes
- les trois tests en scope compilent contre les chemins explicites `continuum::application::scholar::MissionScholar` et `continuum::application::planner::ScopePlanner`
- les tests listes de l'etape sont verts

## 7. Effet sur le mandate principal

Pour la suite du chantier:

- l'etape 5 du `refactor-structure-mandate.md` doit etre lue a travers le present artefact
- la phrase `MissionScholar implemente application::actors::Scholar` est remplacee par `MissionScholar est une concrete applicative amont distincte des ports runtime`
- la phrase `ScopePlanner implemente application::actors::Planner` est remplacee par `ScopePlanner est une concrete applicative amont distincte des ports runtime`
- les etapes 1, 2, 3, 4, 6, 7 et 8 du mandate principal restent inchangees
- aucune extension de scope n'est autorisee au-dela de ce qui est ecrit ici
