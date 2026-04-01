# Slice 2 Implementation Mandate

## 1. Objectif exact

Prouver, en memoire uniquement, un handoff amont propre entre `Scholar` et `Planner`:

- une `RawMission` brute est transformee en `ScholarOutput` structure
- le `Planner` consomme ce `ScholarOutput` structure
- le `Planner` produit une decision minimale coherente

Le slice 2 ne traite pas l'execution du travail. Il traite uniquement la qualite du passage d'information amont.

## 2. Perimetre

### Inclus

- `RawMission` comme entree explicite
- un `ScholarOutput` structure, non reductible a un simple `String`
- un `Scholar` minimal en memoire
- un `Planner` minimal en memoire
- une decision Planner volontairement pauvre: `Proceed` ou `Stop`
- tests unitaires minimaux et un test E2E du handoff amont

### Exclus

- tout agent reel
- toute infra reelle
- DB, filesystem, shell, git, reseau
- event store reel
- `Builder`, `Critic`, boucle complete de revision
- `TaskContract` complet
- policy riche
- toute commande d'execution
- toute persistance ou reprise

## 3. Structures minimales a faire emerger

```rust
pub struct RawMission {
    pub content: String,
}

pub struct ScholarOutput {
    pub mission_summary: String,
    pub selected_task_scope: String,
}

pub enum PlannerDecision {
    Proceed,
    Stop,
}
```

Contraintes de design:

- `RawMission` formalise la frontiere d'entree.
- `ScholarOutput` doit porter au moins deux champs nommes, pour prouver un vrai handoff structure.
- `PlannerDecision` doit rester pauvre et ne pas tirer ce slice vers un contrat d'execution.

## 4. Test E2E principal a viser

Fichier:

- `tests/e2e/session_e2e_scholar_planner_handoff_tests.rs`

Test:

- `transforms_raw_mission_into_structured_scholar_output_before_planner_decides`

Ce test doit prouver exactement:

- une `RawMission` brute entre dans le flux
- le `Scholar` produit un `ScholarOutput` structure
- le `Planner` lit cette structure
- le `Planner` retourne `Proceed` ou `Stop` de facon coherente
- aucun composant d'execution aval n'est requis

## 5. Ordre d'implementation recommande

1. Ajouter le test qui force l'existence de `RawMission`.
2. Ajouter le test qui force `ScholarOutput` a devenir structurel.
3. Ajouter le test unitaire `RawMission -> ScholarOutput`.
4. Ajouter le test unitaire `ScholarOutput -> PlannerDecision`.
5. Ajouter le test E2E du handoff amont.
6. Implementer uniquement le minimum de code necessaire pour faire passer ces tests.

## 6. Definition of done

Le slice 2 est termine si, et seulement si:

- `RawMission` existe comme type explicite d'entree
- `ScholarOutput` n'est plus un simple `String`
- le `Scholar` produit ce `ScholarOutput` en memoire
- le `Planner` consomme cette structure et retourne une decision minimale
- le test E2E principal passe en memoire uniquement
- aucun element hors scope n'a ete introduit pour faire passer les tests
