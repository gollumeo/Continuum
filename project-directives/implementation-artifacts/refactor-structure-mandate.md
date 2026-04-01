# Refactor Structure Mandate

## 1. Objectif exact

Realigner la forme du code de production avec l'architecture deja visee, sans ajouter de nouvelle feature et sans changer le comportement.

Le refacto porte uniquement sur l'extraction hors de `src/lib.rs`, la repartition `domain/` vs `application/`, et la clarification de l'API publique.

## 2. Conventions de nommage retenues

- Pas de suffixe `Port` dans les abstractions applicatives.
- Les abstractions applicatives sont nommees par leur role: `Scholar`, `Planner`, `Builder`, `Critic`.
- Les implementations concretes actuelles sont nommees:
  - `MissionScholar`
  - `ScopePlanner`
- Ne pas utiliser `InMemoryScholar` ou `InMemoryPlanner` tant qu'il ne s'agit pas d'adapters memoire explicites.
- Ne pas introduire d'alias ambigus a la racine si un trait du meme nom existe deja.

## 3. Structure cible des fichiers et modules

```text
src/
  lib.rs
  domain/
    mod.rs
    raw_mission.rs
    scholar_output.rs
    planner_decision.rs
    task_contract.rs
    verdict.rs
    session.rs
  application/
    mod.rs
    actors.rs
    scholar.rs
    planner.rs
    workflow.rs
    session_runner.rs
```

## 4. Repartition exacte domain / application

`domain/`

- `RawMission`
- `ScholarOutput`
- `PlannerDecision`
- `TaskContract`
- `TaskContractError`
- `Verdict`
- `VerdictError`
- `Session`
- `SessionStatus`
- `SessionError`

`application/`

- traits `Scholar`, `Planner`, `Builder`, `Critic`
- `MissionScholar`
- `ScopePlanner`
- `WorkflowState`
- `StateMachine`
- `AgentRole`
- `SessionRunner`
- `SessionSummary`
- `FailureReport`

## 5. Strategie de re-export dans `lib.rs`

`lib.rs` reste une facade mince.

Expose a la racine:

- tous les types `domain`
- `WorkflowState`, `StateMachine`, `AgentRole`
- `SessionRunner`, `SessionSummary`, `FailureReport`

Ne pas exposer a la racine:

- les traits `Scholar`, `Planner`, `Builder`, `Critic`
- `MissionScholar`
- `ScopePlanner`

Exposition explicite par chemin:

- `continuum::application::actors::Scholar`
- `continuum::application::actors::Planner`
- `continuum::application::actors::Builder`
- `continuum::application::actors::Critic`
- `continuum::application::scholar::MissionScholar`
- `continuum::application::planner::ScopePlanner`

## 6. Checklist d'execution

### 1. Scaffolding

Fichiers:

- creer `src/domain/mod.rs`
- creer `src/application/mod.rs`
- modifier `src/lib.rs`

Deplacements:

- aucun

Imports / re-exports:

- `lib.rs`: declarer `pub mod domain;` et `pub mod application;`
- garder les re-exports racine deja stables
- ne rien exposer de nouveau a la racine pour les abstractions applicatives

Tests a rejouer:

- `cargo test --test task_contract_tests`
- `cargo test --test raw_mission_tests`

Arret si:

- la facade `lib.rs` ne peut pas etre preservee sans alias ambigu

### 2. Extraire le domaine slice 1

Fichiers:

- creer `src/domain/task_contract.rs`
- creer `src/domain/verdict.rs`
- creer `src/domain/session.rs`
- modifier `src/domain/mod.rs`
- modifier `src/lib.rs`

Deplacements:

- `TaskContract`, `TaskContractError`
- `Verdict`, `VerdictError`
- `Session`, `SessionStatus`, `SessionError`

Imports / re-exports:

- imports internes via `crate::domain::*`
- re-export racine inchange pour ces types

Tests a rejouer:

- `cargo test --test task_contract_tests`
- `cargo test --test verdict_tests`
- `cargo test --test session_tests`

Arret si:

- un deplacement force une nouvelle signature ou une nouvelle regle metier

### 3. Extraire le domaine slice 2

Fichiers:

- creer `src/domain/raw_mission.rs`
- creer `src/domain/scholar_output.rs`
- creer `src/domain/planner_decision.rs`
- modifier `src/domain/mod.rs`
- modifier `src/lib.rs`

Deplacements:

- `RawMission`
- `ScholarOutput`
- `PlannerDecision`

Imports / re-exports:

- imports internes via `crate::domain::{RawMission, ScholarOutput, PlannerDecision}`
- re-export racine inchange pour ces types

Tests a rejouer:

- `cargo test --test raw_mission_tests`
- `cargo test --test scholar_output_tests`
- `cargo test --test planner_decision_tests`

Arret si:

- l'extraction exige un nouveau type metier ou un renommage metier

### 4. Extraire les abstractions applicatives

Fichiers:

- creer `src/application/actors.rs`
- modifier `src/application/mod.rs`

Deplacements:

- `ScholarPort` vers trait `Scholar`
- `PlannerPort` vers trait `Planner`
- `BuilderPort` vers trait `Builder`
- `CriticPort` vers trait `Critic`

Imports / re-exports:

- imports internes via `crate::application::actors::*`
- pas de re-export racine plat pour ces traits

Tests a rejouer:

- `cargo test --test session_runner_happy_path_tests`
- `cargo test --test session_runner_failure_tests`

Arret si:

- le renommage force autre chose qu'un changement mecanique de noms

### 5. Extraire les implementations applicatives amont

Fichiers:

- creer `src/application/scholar.rs`
- creer `src/application/planner.rs`
- modifier `src/application/mod.rs`

Deplacements:

- struct concrete `Scholar` vers `MissionScholar`
- struct concrete `Planner` vers `ScopePlanner`

Imports / re-exports:

- `MissionScholar` implemente `application::actors::Scholar`
- `ScopePlanner` implemente `application::actors::Planner`
- pas de re-export racine plat

Tests a rejouer:

- `cargo test --test scholar_tests`
- `cargo test --test planner_tests`
- `cargo test --test session_e2e_scholar_planner_handoff_tests`

Arret si:

- les tests exigent de conserver `continuum::Scholar` ou `continuum::Planner` comme concretes racine

### 6. Extraire le workflow

Fichiers:

- creer `src/application/workflow.rs`
- modifier `src/application/mod.rs`
- modifier `src/lib.rs`

Deplacements:

- `WorkflowState`
- `StateMachine`
- `AgentRole`

Imports / re-exports:

- imports internes via `crate::application::workflow::*`
- re-export racine conserve pour ces types

Tests a rejouer:

- `cargo test --test state_machine_transition_tests`
- `cargo test --test state_machine_guard_tests`

Arret si:

- l'extraction force un couplage nouveau avec `SessionRunner`

### 7. Extraire `SessionRunner` en dernier

Fichiers:

- creer `src/application/session_runner.rs`
- modifier `src/application/mod.rs`
- modifier `src/lib.rs`

Deplacements:

- `SessionSummary`
- `FailureReport`
- `SessionRunner`

Imports / re-exports:

- `session_runner.rs` depend de `crate::domain::*`
- `session_runner.rs` depend de `crate::application::actors::{Scholar, Planner, Builder, Critic}`
- re-export racine conserve pour `SessionRunner`, `SessionSummary`, `FailureReport`

Tests a rejouer:

- `cargo test --test session_runner_happy_path_tests`
- `cargo test --test session_runner_failure_tests`
- `cargo test --test session_e2e_happy_path_tests`
- `cargo test --test session_e2e_stop_tests`
- `cargo test --test session_e2e_with_data_tests`

Arret si:

- l'extraction force une decision metier sur le double appel planner, le retry budget, ou le contrat `Builder` / `Critic`

### 8. Nettoyage final

Fichiers:

- modifier `src/lib.rs`

Deplacements:

- aucun

Imports / re-exports:

- supprimer les dernieres definitions inline
- garder uniquement modules et re-exports definis dans ce mandat
- ne pas ajouter d'alias racine ambigus

Tests a rejouer:

- tous les tests existants

Arret si:

- le nettoyage impose une couche de compatibilite non triviale

## 7. Conditions d'arret globales

Stop immediat si le refacto force:

- une nouvelle regle metier
- une nouvelle structure de donnees metier
- un changement de signature publique non prevu
- une redefinition du role de `PlannerDecision`
- une decision sur le contrat reel entre amont et aval
- une compatibilite racine ambiguë entre traits et concretes

## 8. Regle absolue

Refacto mecanique uniquement.

Zero changement de comportement.

Zero nouvelle feature.

`SessionRunner` en dernier.
