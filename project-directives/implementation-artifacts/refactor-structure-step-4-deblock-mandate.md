# Refactor Structure Step 4 Deblock Mandate

## 1. Statut

Artefact complementaire de deblocage.

Il complete:

- `project-directives/implementation-artifacts/refactor-structure-mandate.md`
- `project-directives/implementation-artifacts/planner-decision-split-deblock-mandate.md`

Il ne remplace pas le mandate principal en entier.

Il remplace uniquement l'etape 4 du `refactor-structure-mandate.md`.

Toutes les autres etapes du mandate principal restent inchangées.

## 2. Preconditions obligatoires confirmees

Le chantier de deblocage conceptuel est termine.

Etat reel a considerer comme canonique avant execution de la presente etape:

- `HandoffDecision` est stable en `domain`
- `SessionFlowDecision` est stable en `application`
- `PlannerDecision` a disparu
- `RuntimeProceedStopDecision` a disparu

## 3. Etat reel du repo a integrer

Etat observe au moment de reprendre l'etape 4 du mandate principal:

- `ScholarPort`, `PlannerPort`, `BuilderPort`, `CriticPort` sont encore definis inline dans `src/lib.rs`
- `SessionRunner` depend encore explicitement de ces noms dans `src/lib.rs`
- les tests runtime en scope compilent encore contre ces noms racine
- aucun alias de compatibilite racine ambigu n'est autorise

Conclusion obligatoire:

- l'etape 4 du mandate principal n'est pas executable telle qu'ecrite
- creer seulement `src/application/actors.rs` et modifier seulement `src/application/mod.rs` ne suffit pas
- pour rendre le renommage effectif sans compatibilite ambiguë, l'etape 4 doit aussi autoriser le repointage inline de `SessionRunner` dans `src/lib.rs`
- pour garder l'etape executable, l'etape 4 doit aussi autoriser le realignement minimal des tests runtime explicitement listes pour cette etape

## 4. Regles absolues de cette correction

- refactor mecanique uniquement
- zero changement de comportement
- zero nouvelle feature
- aucun redesign global
- aucun renommage au-dela de `ScholarPort -> Scholar`, `PlannerPort -> Planner`, `BuilderPort -> Builder`, `CriticPort -> Critic`
- aucune nouvelle regle metier
- aucune nouvelle structure de donnees metier
- aucune compatibilite magique
- aucun alias racine ambigu
- aucun re-export racine plat pour `Scholar`, `Planner`, `Builder`, `Critic`
- ne pas toucher aux implementations concretes `MissionScholar` et `ScopePlanner`
- ne pas toucher au workflow
- ne pas toucher a `HandoffDecision`
- ne pas toucher a `SessionFlowDecision`
- ne pas extraire encore `SessionRunner` hors de `src/lib.rs`
- ne pas toucher a d'autres tests que ceux explicitement listes ci-dessous
- imports internes de production via `crate::application::actors::*`

## 5. Etape 4 corrigee et executable

### 5.1 But exact

But de l'etape corrigee:

- extraire les abstractions applicatives dans `src/application/actors.rs`
- remplacer les noms `*Port` par les noms canoniques `Scholar`, `Planner`, `Builder`, `Critic`
- repointer inline `SessionRunner` vers ces abstractions extraites sans changer son comportement
- realigner minimalement les deux tests runtime deja en scope pour qu'ils compilent contre les nouveaux noms sans compatibilite racine ambiguë

### 5.2 Fichiers autorises pour cette etape

Fichiers a creer:

- `src/application/actors.rs`

Fichiers a modifier:

- `src/application/mod.rs`
- `src/lib.rs`
- `tests/session_runner/session_runner_happy_path_tests.rs`
- `tests/session_runner/session_runner_failure_tests.rs`

Interdiction explicite:

- ne toucher a aucun autre fichier

### 5.3 Deplacements et renommages autorises

Deplacements autorises:

- deplacer le trait inline `ScholarPort` hors de `src/lib.rs` vers `src/application/actors.rs`
- deplacer le trait inline `PlannerPort` hors de `src/lib.rs` vers `src/application/actors.rs`
- deplacer le trait inline `BuilderPort` hors de `src/lib.rs` vers `src/application/actors.rs`
- deplacer le trait inline `CriticPort` hors de `src/lib.rs` vers `src/application/actors.rs`

Renommages autorises:

- `ScholarPort` devient le trait `Scholar`
- `PlannerPort` devient le trait `Planner`
- `BuilderPort` devient le trait `Builder`
- `CriticPort` devient le trait `Critic`

Limite stricte:

- aucun autre renommage n'est autorise dans cette etape

### 5.4 Contenu autorise de l'extraction des abstractions applicatives

`src/application/actors.rs` doit contenir uniquement:

- le trait `Scholar`
- le trait `Planner`
- le trait `Builder`
- le trait `Critic`

Contraintes sur ces traits:

- signatures identiques aux traits inline actuels, hors renommage mecanique des noms de traits
- `Planner` conserve le meme contrat de retour vers `SessionFlowDecision`
- aucun changement de methode
- aucun changement de parametre
- aucun changement de type de retour hors le renommage deja tranche du nom du trait

`src/application/mod.rs` doit seulement:

- declarer `pub mod actors;`
- conserver les declarations applicatives deja stables
- ne pas ajouter de re-export racine plat pour les traits

### 5.5 Contenu autorise du repointage de `SessionRunner`

`src/lib.rs` peut etre modifie uniquement pour:

- supprimer les definitions inline de `ScholarPort`, `PlannerPort`, `BuilderPort`, `CriticPort`
- importer les abstractions via `crate::application::actors::*`
- remplacer les references de type de `SessionRunner` vers `Scholar`, `Planner`, `Builder`, `Critic`
- remplacer les types des champs, des arguments de constructeurs, et des objets de traits `Box<dyn ...>` pour pointer vers les nouveaux traits

Interdictions explicites dans `src/lib.rs`:

- ne pas modifier la logique de `SessionRunner`
- ne pas modifier le workflow
- ne pas modifier `MissionScholar`
- ne pas modifier `ScopePlanner`
- ne pas modifier `SessionFlowDecision`
- ne pas ajouter de `pub use` racine pour les traits
- ne pas ajouter d'alias de compatibilite pour `ScholarPort`, `PlannerPort`, `BuilderPort`, `CriticPort`

### 5.6 Contenu autorise du realignement minimal des tests en scope

Le realignement des tests n'est autorise que pour les deux fichiers suivants:

- `tests/session_runner/session_runner_happy_path_tests.rs`
- `tests/session_runner/session_runner_failure_tests.rs`

Changements autorises dans ces tests uniquement:

- remplacer les imports racine `ScholarPort`, `PlannerPort`, `BuilderPort`, `CriticPort`
- importer a la place `continuum::application::actors::{Scholar, Planner, Builder, Critic}`
- realigner les `impl` de doubles de test pour implementer `Scholar`, `Planner`, `Builder`, `Critic`
- remplacer les annotations de type qui pointent encore sur les anciens noms

Interdictions explicites dans ces tests:

- ne pas changer la logique des tests
- ne pas changer les assertions
- ne pas modifier les decisions runtime
- ne pas toucher aux autres tests runtime ou e2e

### 5.7 Tests a rejouer

Rejouer uniquement:

- `cargo test --test session_runner_happy_path_tests`
- `cargo test --test session_runner_failure_tests`

### 5.8 Conditions d'arret

Arret immediat si:

- le renommage force autre chose qu'un changement mecanique de noms
- un conflit de nom ou une ambiguïte reapparait
- l'etape exige de toucher un fichier autre que les cinq fichiers autorises
- l'etape exige une compatibilite racine pour conserver `ScholarPort`, `PlannerPort`, `BuilderPort` ou `CriticPort`
- l'etape exige de modifier le comportement de `SessionRunner`
- l'etape exige de modifier `MissionScholar` ou `ScopePlanner`
- l'etape exige de modifier `HandoffDecision` ou `SessionFlowDecision`
- un des deux tests listes echoue pour une raison non liee au renommage mecanique des abstractions applicatives et de leurs imports de tests en scope

### 5.9 Sortie attendue de l'etape

Sortie attendue si l'etape est executee correctement:

- `src/application/actors.rs` existe
- `src/application/mod.rs` expose `actors` comme sous-module
- `src/lib.rs` ne definit plus inline `ScholarPort`, `PlannerPort`, `BuilderPort`, `CriticPort`
- `SessionRunner` depend inline de `Scholar`, `Planner`, `Builder`, `Critic` via `crate::application::actors::*`
- aucun re-export racine plat n'a ete ajoute pour ces traits
- les deux tests runtime en scope compilent contre `continuum::application::actors::{Scholar, Planner, Builder, Critic}`
- les tests listes de l'etape sont verts

## 6. Effet sur le mandate principal

Pour la suite du chantier:

- l'etape 4 du `refactor-structure-mandate.md` doit etre lue a travers le present artefact
- les etapes 1, 2, 3, 5, 6, 7 et 8 du mandate principal restent inchangées
- aucune extension de scope n'est autorisee au-dela de ce qui est ecrit ici
