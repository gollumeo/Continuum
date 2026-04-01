# PlannerDecision Split Deblock Mandate

## 1. Statut

Artefact complementaire de deblocage.

Il complete:

- `project-directives/implementation-artifacts/refactor-structure-mandate.md`
- `project-directives/implementation-artifacts/slice-2-implementation-mandate.md`
- `project-directives/planning-artifacts/architecture.md`

Il remplace integralement la version precedente du present artefact.

## 2. Etat reel obligatoire

Etat observe apres execution terrain:

- l'etape 1 a ete executee
- l'etape 2 a ete executee
- l'etape 3 a ete tentee puis arretee correctement avant modification

Faits confirms dans le repo:

- `HandoffDecision` existe en `domain`
- `RuntimeProceedStopDecision` existe en `application`
- `SessionFlowDecision` existe en `application` avec `Build`, `Retry`, `Complete`
- `PlannerPort::decide` retourne encore `RuntimeProceedStopDecision`
- le runtime actuel declenche `builder.run(...)` sur `RuntimeProceedStopDecision::Proceed`
- le runtime actuel reteste `RuntimeProceedStopDecision::Proceed` pour budget puis completion
- les tests runtime historiques expriment deja un autre contrat avec `Build`, `Retry`, `Complete`

Conclusion obligatoire:

- `RuntimeProceedStopDecision` et `SessionFlowDecision` ne sont pas deux noms pour le meme concept
- le runtime actuel encode encore deux moments de decision distincts avec un seul signal `Proceed`
- la migration ne peut plus etre ecrite comme un simple remplacement de type

## 3. Decisions canoniques

### 3.1 Relation exacte entre `RuntimeProceedStopDecision` et `SessionFlowDecision`

`RuntimeProceedStopDecision` est un type transitoire surcharge.

Il ne possede pas de projection 1:1 vers `SessionFlowDecision`.

Relation exacte:

- `RuntimeProceedStopDecision::Proceed` au premier point de decision runtime correspond a `SessionFlowDecision::Build`
- `RuntimeProceedStopDecision::Proceed` au second point de decision runtime correspond a `SessionFlowDecision::Complete`
- `RuntimeProceedStopDecision::Proceed` ne correspond jamais a `SessionFlowDecision::Retry`
- `RuntimeProceedStopDecision::Stop` n'a **aucune** projection canonique definie vers `SessionFlowDecision` dans le present mandate

Consequence obligatoire:

- le type transitoire doit etre eclate par point de decision
- aucun mapping implicite ou automatique n'est autorise

### 3.2 Signification exacte de `Proceed` dans le runtime actuel

`Proceed` n'est pas une decision runtime canonique.

`Proceed` est un signal transitoire ambivalent qui porte deux significations differentes selon la position de l'appel:

- avant `builder.run(...)`: autoriser une iteration Builder, donc `Build`
- apres succes `Builder` + `Critic`: autoriser la terminaison reussie, donc `Complete`

`Proceed` doit donc etre traite comme un etat transitoire a eclater, pas comme une valeur cible.

### 3.3 Moment exact d'apparition de `Complete`

`Complete` apparait **directement depuis la decision du Planner**.

Position exacte:

- au second point de decision Planner
- apres retour reussi de `Builder`
- apres retour valide de `Critic`
- avant la completion de session par `SessionRunner`

`Complete` ne doit pas etre synthétise par le runtime.

### 3.4 Contrat cible exact entre `PlannerPort::decide(...)` et `SessionRunner`

Contrat cible du chantier:

- `PlannerPort::decide(&ScholarOutput) -> SessionFlowDecision`

Validite par point d'appel dans `SessionRunner`:

- premier appel Planner: valeur attendue `Build`
- second appel Planner, apres succes `Builder` + `Critic`: valeur attendue `Complete` ou `Retry`

Interpretation par `SessionRunner`:

- `Build` au premier appel: lancer `Builder`, puis `Critic`
- `Complete` au second appel: completer la session
- `Retry` au second appel: demander une nouvelle iteration uniquement si le budget runtime le permet deja
- `Retry` au second appel avec budget epuise: arret terminal

Limite explicite du present mandate:

- si la migration vers `Retry` exige d'introduire une boucle supplementaire non deja requise par les tests listés dans ce document, arret immediat

### 3.5 Sort de `RuntimeProceedStopDecision::Stop`

`RuntimeProceedStopDecision::Stop` est un residu transitoire hors projection canonique dans le present chantier.

Decision explicite:

- aucun code de production ou test touche par ce mandate ne doit essayer de le mapper vers `Build`, `Retry` ou `Complete`
- si une dependance active a `RuntimeProceedStopDecision::Stop` apparait pendant la migration, arret immediat et escalade vers un artefact separé

## 4. Repartition exacte des concepts

### `HandoffDecision`

- Couche: `domain`
- Role: decision minimale de handoff amont entre `Scholar` et `Planner`
- Semantique autorisee: `Proceed`, `Stop`
- Statut: canonique et deja stabilise

### `RuntimeProceedStopDecision`

- Couche: `application`
- Role: type transitoire surcharge, strictement reserve au chantier de migration
- Semantique autorisee: `Proceed`, `Stop`
- Statut: non canonique, a supprimer en fin de chantier

### `SessionFlowDecision`

- Couche: `application`
- Role: contrat runtime canonique entre `PlannerPort` et `SessionRunner`
- Semantique autorisee dans le present chantier: `Build`, `Retry`, `Complete`
- Statut: canonique

## 5. Regles absolues

- ne jamais reintroduire `PlannerDecision`
- ne jamais melanger `HandoffDecision` avec un type runtime
- ne jamais traiter `RuntimeProceedStopDecision` comme une API durable
- ne jamais introduire de compatibilite magique entre `RuntimeProceedStopDecision` et `SessionFlowDecision`
- ne jamais ajouter de re-export racine de `SessionFlowDecision`
- ne jamais ajouter de re-export racine de `RuntimeProceedStopDecision`
- ne jamais inferer `Complete` depuis un etat runtime sans decision explicite du Planner
- ne jamais inferer `Retry` depuis l'ancien `Proceed`
- si une etape force une nouvelle branche de comportement non deja couverte par les tests listes pour cette etape, arret immediat
- si une etape force une decision supplementaire sur le contrat reel `PlannerPort <-> SessionRunner`, arret immediat

## 6. Ordre d'execution operable

Ordre obligatoire:

1. couper le type partage et faire disparaitre `PlannerDecision`
2. introduire `SessionFlowDecision`
3. realigner **en une seule coupe** le contrat `PlannerPort <-> SessionRunner` et les tests runtime qui l'exercent
4. supprimer `RuntimeProceedStopDecision`
5. reprendre ensuite seulement le `refactor-structure-mandate.md`

Decision de sequence:

- la migration du contrat runtime et la mise a jour des tests runtime historiques ne doivent plus etre separees en deux etapes distinctes
- elles constituent une coupe atomique unique, sinon le chantier redevient non executable

## 7. Etapes d'execution

### Etape 1. Couper le type partage et faire disparaitre `PlannerDecision`

Statut: deja executee.

Sortie attendue de l'etape:

- `PlannerDecision` n'existe plus nulle part comme type actif
- l'amont utilise `HandoffDecision`
- le runtime actuel utilise `RuntimeProceedStopDecision`

### Etape 2. Introduire le concept runtime cible

Statut: deja executee.

Sortie attendue de l'etape:

- `SessionFlowDecision` existe comme type distinct
- `RuntimeProceedStopDecision` existe encore comme type transitoire

### Etape 3. Coupe atomique du contrat `PlannerPort <-> SessionRunner`

But:

- remplacer le contrat transitoire par le contrat runtime canonique
- eclater l'ancien `Proceed` par point de decision
- realigner en meme temps les tests runtime qui compilent contre ce contrat

Fichiers a creer ou modifier:

- modifier `src/lib.rs`
- modifier `tests/session_runner/session_runner_happy_path_tests.rs`
- modifier `tests/session_runner/session_runner_failure_tests.rs`
- modifier `tests/e2e/session_e2e_happy_path_tests.rs`
- modifier `tests/e2e/session_e2e_stop_tests.rs`
- modifier `tests/e2e/session_e2e_with_data_tests.rs`

Types a renommer / deplacer / introduire:

- remplacer `RuntimeProceedStopDecision` par `SessionFlowDecision` dans `PlannerPort`
- remplacer `RuntimeProceedStopDecision` par `SessionFlowDecision` dans les usages runtime inline de `SessionRunner`
- ne pas toucher `HandoffDecision`
- ne pas supprimer encore `RuntimeProceedStopDecision`

Realignement contractuel obligatoire:

- premier appel Planner dans `SessionRunner`: attendre `SessionFlowDecision::Build`
- second appel Planner apres succes `Builder` + `Critic`: attendre `SessionFlowDecision::Complete` ou `SessionFlowDecision::Retry`
- `SessionFlowDecision::Complete` remplace le second `Proceed`
- `SessionFlowDecision::Build` remplace le premier `Proceed`
- `SessionFlowDecision::Retry` n'est autorise qu'au second appel Planner

Realignement minimal des fixtures de tests autorise et obligatoire dans cette etape:

- remplacer dans les tests listes ci-dessus les anciens usages de `PlannerDecision` par `SessionFlowDecision`
- realigner mechanquement les instanciations de `ScholarOutput` qui ne correspondent plus a la structure actuelle du type, uniquement dans ces memes fichiers de tests

Tests a rejouer:

- `cargo test --test session_runner_happy_path_tests`
- `cargo test --test session_runner_failure_tests`
- `cargo test --test session_e2e_happy_path_tests`
- `cargo test --test session_e2e_stop_tests`
- `cargo test --test session_e2e_with_data_tests`

Condition d'arret:

- si `Build` doit etre accepte a un autre moment que le premier appel Planner
- si `Complete` doit etre accepte a un autre moment que le second appel Planner
- si `Retry` doit etre accepte au premier appel Planner
- si la migration exige de mapper `RuntimeProceedStopDecision::Stop`
- si la migration exige de toucher un fichier non liste ci-dessus
- si un test liste echoue pour une raison non liee au realignement mecanique du contrat de decision ou des fixtures `ScholarOutput` deja connues
- si l'implementation d'un chemin `Retry` avec budget restant exige une nouvelle boucle runtime non deja requise pour faire passer les tests listes ci-dessus

Sortie attendue de l'etape:

- `PlannerPort::decide` retourne `SessionFlowDecision`
- `SessionRunner` n'utilise plus `RuntimeProceedStopDecision`
- les tests runtime historiques en scope compilent contre `SessionFlowDecision`

### Etape 4. Supprimer le type transitoire

But:

- supprimer `RuntimeProceedStopDecision` une fois le contrat runtime reel totalement repointe

Fichiers a creer ou modifier:

- supprimer `src/application/runtime_proceed_stop_decision.rs`
- modifier `src/application/mod.rs`
- modifier `src/lib.rs` uniquement pour retirer les references residuelles, si elles existent encore

Types a renommer / deplacer / introduire:

- supprimer `RuntimeProceedStopDecision`
- conserver `HandoffDecision`
- conserver `SessionFlowDecision`

Tests a rejouer:

- `cargo test --test session_runner_happy_path_tests`
- `cargo test --test session_runner_failure_tests`
- `cargo test --test session_e2e_happy_path_tests`
- `cargo test --test session_e2e_stop_tests`
- `cargo test --test session_e2e_with_data_tests`

Condition d'arret:

- si une reference de production ou de test en scope depend encore du type transitoire
- si la suppression force une compatibilite publique non prevue

Sortie attendue de l'etape:

- `RuntimeProceedStopDecision` n'existe plus
- `SessionFlowDecision` est le seul contrat runtime actif

### Etape 5. Reprise du mandate principal

Precondition obligatoire:

- `HandoffDecision` est stable en `domain`
- `SessionFlowDecision` est stable en `application`
- `RuntimeProceedStopDecision` a disparu
- le contrat `PlannerPort <-> SessionRunner` n'est plus ambigu

Effet:

- seulement apres satisfaction de ces preconditions, reprise de `refactor-structure-mandate.md`

## 8. Moments exacts de disparition des anciens noms

### `PlannerDecision`

`PlannerDecision` cesse d'etre utilise a la fin de l'etape 1.

### `RuntimeProceedStopDecision`

`RuntimeProceedStopDecision` cesse d'etre utilise en production a la fin de l'etape 3.

Il disparait du repo a la fin de l'etape 4.

## 9. Conditions d'arret globales

Stop immediat si le chantier force:

- une nouvelle regle metier
- une nouvelle structure de donnees metier
- une projection implicite de `Stop` vers `Build`, `Retry` ou `Complete`
- une compatibilite magique entre types transitoires et types canoniques
- un changement de comportement non necessaire pour satisfaire les tests explicitement listes
- un elargissement du contrat `PlannerPort <-> SessionRunner` au-dela de ce qui est tranche ici

## 10. Regle finale

Le chantier est termine si, et seulement si:

- `HandoffDecision` est le seul concept amont
- `SessionFlowDecision` est le seul concept runtime canonique
- `PlannerDecision` n'existe plus
- `RuntimeProceedStopDecision` n'existe plus
- le premier point de decision Planner porte `Build`
- le second point de decision Planner porte `Complete` ou `Retry`
- aucun fichier touche par ce chantier ne melange concept amont, type transitoire et contrat runtime canonique
