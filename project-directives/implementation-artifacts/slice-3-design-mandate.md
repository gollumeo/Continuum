# Slice 3 Design Mandate

## 1. Statut et position normative

Cet artefact definit le prochain slice d'implementation dans le modele post-refactor stabilise.

Il est contraint par:

- `project-directives/implementation-artifacts/slice-1-closure.md`
- `project-directives/implementation-artifacts/slice-2-implementation-mandate.md`
- `project-directives/implementation-artifacts/slice-2-post-refactor-lock.md`
- `project-directives/planning-artifacts/architecture.md`

Il doit etre lu dans l'etat final du repo apres refactor structurel complet et suite de tests verte.

Il ne redefinit ni le slice 1, ni le slice 2, ni l'architecture globale.

Il ouvre uniquement un perimetre runtime propre pour le slice 3.

## 2. Objectif exact du slice 3

Le slice 3 doit prouver, en memoire uniquement, qu'un runtime minimal deja valide en flux lineaire peut executer une boucle de revision runtime explicite et bornee sans ambiguite de role.

Le slice 3 doit prouver exactement:

- qu'un `Critic` peut produire une demande de revision structuree en runtime
- qu'un `Planner` runtime peut convertir cette situation en decision `SessionFlowDecision::Retry`
- qu'un `Builder` peut etre relance pour une nouvelle iteration runtime dans la meme session
- que le budget de retry est applique par le runtime sur un cas reel de revision, pas seulement sur un arret artificiel
- que la session peut ensuite se terminer explicitement en succes apres revision

Pourquoi c'est le plus petit prochain pas utile:

- le slice 1 a deja prouve un runtime minimal lineaire en memoire
- le slice 2 a deja verrouille le handoff amont comme perimetre distinct
- le principal risque residuel est maintenant un runtime correct sur chemin nominal simple, mais pas encore prouve sur une vraie iteration de rework
- ce slice ajoute la plus petite preuve runtime manquante sans rouvrir l'amont ni l'infrastructure

## 3. Perimetre inclus

Le slice 3 peut mobiliser uniquement les elements runtime et domain deja compatibles avec ce but:

- `SessionRunner`
- `SessionFlowDecision`
- `application::actors::{Scholar, Planner, Builder, Critic}`
- `Session`, `SessionStatus`, `FailureReport`, `SessionSummary`
- `ScholarOutput` comme donnee runtime deja propagee dans la session
- `Verdict` comme support structure minimal de demande de revision si necessaire
- `TaskContract` uniquement pour porter ou verifier un budget borné si cela est strictement necessaire au runtime du slice
- tests unitaires runtime et tests E2E runtime en memoire uniquement

Les comportements runtime autorises dans ce slice sont strictement:

- un premier passage `Scholar -> Planner -> Builder -> Critic`
- une emission de revision structuree par le `Critic`
- une decision runtime `Retry` par le `Planner`
- une deuxieme iteration `Builder -> Critic -> Planner`
- une terminaison explicite en `Completed` apres revision reussie
- un arret explicite si un retry est demande sans budget disponible

Contrainte de perimetre:

- le slice 3 doit rester entierement en memoire et ne doit dependre d'aucune IO reelle

## 4. Perimetre exclu

Les elements suivants sont explicitement hors scope du slice 3:

- toute modification du handoff amont du slice 2
- toute reinterpretation du slice 1 comme simple precurseur du slice 3
- toute projection entre `HandoffDecision` et `SessionFlowDecision`
- toute implementation de `MissionScholar` pour `application::actors::Scholar`
- toute implementation de `ScopePlanner` pour `application::actors::Planner`
- toute utilisation de `MissionScholar` ou `ScopePlanner` comme composants runtime du `SessionRunner`
- toute IO reelle: DB, filesystem, shell, git, reseau
- toute persistance, reprise, snapshot reel, diff reel, event store reel
- toute boucle de revision illimitee ou riche au-dela du budget borne
- toute expansion du contrat Builder vers des edits reels ou des politiques de fichiers
- toute extension infra ou schema externe
- toute redescription du systeme global ou redesign du modele de couches
- toute nouvelle feature non necessaire a la preuve d'une boucle de revision runtime en memoire

Regle executable:

- si un changement n'est pas requis pour prouver une boucle de revision runtime bornee en memoire, alors ce changement est hors scope du slice 3

## 5. Frontiere explicite avec les slices 1 et 2

### 5.1 Acquis du slice 1

Le slice 1 a deja acquis et ferme les points suivants:

- un runtime minimal en memoire existe
- `SessionRunner` orchestre un chemin nominal lineaire
- un resultat terminal explicite existe: `SessionSummary` ou `FailureReport`
- un premier arret sur budget existe deja
- `ScholarOutput` est deja propage au runtime minimal

Regle executable:

- le slice 3 doit etendre la preuve runtime du slice 1 sur la revision, sans redefinir ce que le slice 1 a deja valide

### 5.2 Acquis et verrou du slice 2

Le slice 2 a deja acquis et verrouille les points suivants:

- `MissionScholar`, `ScopePlanner`, `RawMission`, `ScholarOutput`, `HandoffDecision` appartiennent au handoff amont
- le slice 2 s'arrete strictement a `HandoffDecision`
- `SessionRunner`, `SessionFlowDecision` et `application::actors::*` appartiennent au runtime
- aucun pont implicite entre amont et runtime n'est autorise

Regle executable:

- le slice 3 n'ajoute rien au handoff amont et ne doit jamais utiliser le slice 2 comme justification pour modifier le runtime par glissement conceptuel

### 5.3 Apport propre du slice 3

Le slice 3 ajoute uniquement la preuve suivante:

- le runtime peut executer une revision bornee apres critique, avec decision explicite de retry et terminaison explicite apres rework

Le slice 3 n'ajoute pas de nouvelle frontiere entre amont et runtime.

Le slice 3 n'ajoute pas de nouveau sens a `HandoffDecision`.

## 6. Types et responsabilites autorises

Les responsabilites runtime autorisees sont les suivantes:

- `Scholar` runtime produit le `ScholarOutput` consomme par le runtime courant
- `Planner` runtime decide `Build`, `Retry` ou `Complete` dans la session runtime
- `Builder` runtime execute une iteration de travail en memoire dans le cadre de la session
- `Critic` runtime evalue une iteration et peut exiger une revision structuree
- `SessionRunner` reste l'unique orchestrateur des appels, du budget et de la terminaison

Les concepts qui peuvent emerger dans ce slice sont strictement des concepts runtime:

- une preuve structuree qu'une iteration doit etre revisee
- une propagation explicite de cette revision dans la boucle runtime suivante
- un comptage borne du retry applique par le runtime

Les concepts amont qui doivent rester inchanges sont strictement:

- `MissionScholar`
- `ScopePlanner`
- `RawMission`
- `ScholarOutput` en tant que sortie amont deja stabilisee
- `HandoffDecision`

Contrainte de responsabilite:

- si un concept sert a piloter une iteration Builder/Critic/Planner, alors il appartient au runtime
- si un concept sert a transformer une mission brute en handoff structure puis `Proceed` ou `Stop`, alors il appartient a l'amont

Contrainte de vocabulaire:

- le mot `handoff` designe exclusivement le flux amont du slice 2
- le mot `revision` designe exclusivement la boucle runtime du slice 3
- le mot `decision` doit toujours etre qualifie par son contexte effectif quand une ambiguite est possible: `HandoffDecision` ou `SessionFlowDecision`

## 7. Interdictions absolues

Les interdictions suivantes sont absolues:

- Il est interdit de faire implementer `MissionScholar` par `application::actors::Scholar`
- Il est interdit de faire implementer `ScopePlanner` par `application::actors::Planner`
- Il est interdit d'utiliser `MissionScholar` ou `ScopePlanner` comme preuve runtime du slice 3
- Il est interdit de projeter `HandoffDecision::Proceed` vers `SessionFlowDecision::Build`
- Il est interdit de projeter `HandoffDecision::Stop` vers un arret runtime
- Il est interdit d'introduire un type, alias ou helper dont le but est de recoller conceptuellement amont et runtime
- Il est interdit de redefinir `ScholarOutput` pour lui faire porter un contrat de revision runtime
- Il est interdit d'etendre le slice 3 vers l'infrastructure reelle, la persistance, la reprise, les snapshots reels ou les diffs reels
- Il est interdit d'etendre le slice 3 vers une policy riche de scope fichiers ou de commandes
- Il est interdit de transformer le slice 3 en redesign du moteur global ou du contrat complet `task_contract`
- Il est interdit d'introduire une boucle non bornee ou de laisser un agent s'auto-router hors des decisions runtime explicites
- Il est interdit de modifier le sens deja stabilise de `SessionSummary`, `FailureReport`, `SessionStatus` ou `SessionRunner`
- Il est interdit de presenter le slice 3 comme une extension du handoff amont

Regle executable generale:

- toute proposition qui brouille la separation canonique entre amont et runtime est non conforme et doit etre rejetee

## 8. Definition of done

Le slice 3 est termine si, et seulement si, toutes les conditions suivantes sont vraies:

- un test runtime prouve qu'une session peut effectuer au moins une vraie boucle de revision en memoire
- cette boucle contient un premier passage Builder/Critic suivi d'une decision runtime `Retry`
- une seconde iteration Builder/Critic est effectivement executee dans la meme session
- la session se termine ensuite explicitement avec `SessionSummary` en succes apres rework
- un test prouve que le budget de retry est applique sur ce cas de revision reel
- aucun composant amont `MissionScholar`, `ScopePlanner` ou `HandoffDecision` n'est mobilise pour satisfaire cette preuve runtime
- aucun pont implicite ou explicite n'est introduit entre `HandoffDecision` et `SessionFlowDecision`
- aucun element hors scope du slice 3 n'a ete introduit pour faire passer les tests

Condition minimale de preuve:

- le slice 3 doit demontrer un chemin `Build -> Critique demandant revision -> Retry -> Build -> Critique acceptable -> Complete` dans le runtime, ou l'equivalent strictement isomorphe dans les noms stabilises du repo

## 9. Ordre d'implementation recommande

L'ordre suivant est obligatoire pour une implementation TDD stricte du slice 3:

1. Ajouter un test unitaire runtime qui force l'existence d'un signal de revision structure runtime, distinct de tout concept amont.

2. Ajouter un test unitaire runtime qui force le `Critic` a pouvoir emettre cette revision de facon exploitable par l'orchestration.

3. Ajouter un test unitaire runtime qui force le `Planner` a produire `SessionFlowDecision::Retry` dans un cas de revision, sans introduire de pont avec `HandoffDecision`.

4. Ajouter un test unitaire sur `SessionRunner` qui force une seconde iteration Builder apres une premiere critique demandant revision.

5. Ajouter un test unitaire sur `SessionRunner` qui force l'application effective du budget de retry sur ce chemin de revision reel.

6. Ajouter un test E2E runtime qui prouve une session complete avec une seule revision bornee puis terminaison explicite en succes.

7. Ajouter un test E2E runtime qui prouve l'arret terminal quand une revision est demandee mais qu'aucun budget de retry n'est disponible.

8. Implementer uniquement le minimum de code necessaire pour faire passer ces tests, sans toucher au handoff amont et sans rouvrir les decisions deja verrouillees.

## 10. Sortie attendue du slice 3

Si le slice 3 est execute correctement, il doit produire exactement cette nouvelle garantie:

- Continuum ne prouve plus seulement un runtime lineaire minimal, mais aussi une boucle de revision runtime bornee en memoire, sous controle explicite de `SessionRunner`, sans aucune confusion avec le handoff amont verrouille du slice 2.
