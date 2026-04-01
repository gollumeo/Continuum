# Slice 3 Runtime Critic Signal Decision

## 1. Statut et position normative

Cet artefact est une decision complementaire de cadrage pour l'implementation du slice 3.

Il complete et contraint:

- `project-directives/implementation-artifacts/slice-3-design-mandate.md`
- `project-directives/implementation-artifacts/slice-2-post-refactor-lock.md`
- `project-directives/planning-artifacts/architecture.md`

Il doit etre lu dans l'etat actuel du repo apres refactor structurel complet.

Il ne modifie pas le slice 2.

Il ne redefinit pas l'architecture globale.

Il fixe uniquement le contrat conceptuel minimal du signal runtime emis par le `Critic` pour le slice 3.

## 2. Probleme exact a resoudre

Le slice 3 doit prouver une boucle de revision runtime bornee.

Pour rendre cette preuve executable, le runtime doit pouvoir recevoir du `Critic` un signal explicite, structure et typable indiquant l'etat d'une iteration runtime.

Le type `Verdict` existant est insuffisant comme contrat runtime du slice 3 pour les raisons suivantes:

- `Verdict` appartient deja au domaine actuel et ne porte pas explicitement l'ensemble minimal des issues runtime attendues pour la boucle du slice 3
- `Verdict` existant exprime aujourd'hui une forme partielle centree sur `required_changes`, mais ne constitue pas un signal runtime explicite, complet et autoporteur
- reutiliser strictement `Verdict` tel qu'existant imposerait une semantique implicite du type: presence de changements requis signifie revision, absence ou impossibilite de les exprimer signifie autre chose
- une telle semantique implicite est non conforme a l'objectif du slice 3, qui exige une orchestration runtime explicite, testable et non ambiguë
- reutiliser strictement `Verdict` tel qu'existant risquerait d'ouvrir une relecture globale du domaine au lieu d'introduire le plus petit contrat runtime necessaire

Conclusion obligatoire:

- le slice 3 ne doit pas reutiliser strictement `Verdict` tel qu'existant comme signal runtime du `Critic`

## 3. Decision canonique

Decision canonique:

- un nouveau type minimal dedie doit emerger pour porter le signal runtime du `Critic` dans le slice 3

Ce nouveau type est obligatoire parce qu'il est le plus petit moyen correct de:

- rendre explicite l'issue runtime d'une revue du `Critic`
- permettre au runtime de raisonner sur cette issue sans convention cachee
- tester la boucle de revision du slice 3 sans reinterpreter `Verdict` existant
- maintenir la separation canonique entre handoff amont et orchestration runtime

Decision de portee:

- ce nouveau type n'est pas une nouvelle feature generale du systeme
- ce nouveau type est un contrat local, minimal et contraint au runtime du slice 3

## 4. Couche correcte et responsabilite exacte

Ce nouveau type appartient a la couche `application`.

Il n'appartient pas a la couche `domain`.

Justification normative:

- il sert a piloter l'orchestration runtime
- il sert a exprimer l'issue exploitable par `SessionRunner` d'une revue `Critic`
- il ne decrit pas le handoff amont
- il ne formalise pas une nouvelle verite metier generale du systeme
- il n'a pas vocation a devenir un concept transverse du domaine global

Responsabilite exacte:

- exprimer, de facon explicite et minimale, le resultat runtime d'une iteration revue par le `Critic` pour permettre au runtime de poursuivre, relancer ou arreter la boucle selon les regles du slice 3

Regle executable:

- si un type sert principalement a faire avancer la boucle `Builder -> Critic -> Planner` dans `SessionRunner`, alors ce type appartient a `application`

## 5. Surface semantique minimale autorisee

La surface semantique autorisee pour ce nouveau type est strictement minimale.

Ce type doit porter uniquement ce qui est necessaire pour exprimer explicitement l'issue runtime d'une revue du `Critic` dans le slice 3.

Il doit permettre de distinguer au minimum:

- une iteration acceptable sans rework supplementaire
- une iteration qui exige une revision explicite
- un arret explicite si le `Critic` conclut que la session doit s'interrompre

Si le cas de revision est present, le type peut porter uniquement le minimum necessaire pour rendre la revision exploitable par les tests runtime du slice 3.

Contrainte de minimalite:

- aucune donnee ne doit etre portee si elle n'est pas strictement necessaire pour prouver la boucle de revision runtime bornee du slice 3

## 6. Ce que ce type ne doit surtout pas porter

Ce type ne doit pas porter:

- une decision de handoff amont
- une `SessionFlowDecision`
- une projection ou un alias de `HandoffDecision`
- une projection ou un alias implicite de `SessionFlowDecision`
- un contrat d'execution global
- un `TaskContract` complet
- une policy de fichiers, de commandes, d'infrastructure ou de persistance
- des donnees de snapshot, diff, event store, resume ou evidence reelle
- des concepts metier globaux qui depassent la boucle runtime du slice 3
- des champs ajoutes par anticipation pour des slices ulterieurs

Regle executable:

- si un champ n'est pas necessaire pour exprimer explicitement l'issue runtime minimale du `Critic` dans le slice 3, alors ce champ est interdit

## 7. Interdictions absolues

Les interdictions suivantes sont absolues:

- Il est interdit de reutiliser strictement `Verdict` tel qu'existant comme signal runtime du `Critic`
- Il est interdit de modifier `Verdict` existant pour le faire devenir le contrat runtime du slice 3
- Il est interdit de promouvoir ce nouveau type comme nouveau concept general du domaine sans nouvelle decision explicite hors du slice 3
- Il est interdit de placer ce nouveau type en `domain`
- Il est interdit de presenter ce nouveau type comme extension du handoff amont
- Il est interdit de projeter `HandoffDecision` vers ce nouveau type
- Il est interdit de projeter ce nouveau type vers `HandoffDecision`
- Il est interdit de projeter implicitement ce nouveau type vers `SessionFlowDecision`
- Il est interdit de fusionner semantiquement critique runtime et decision planner runtime dans un seul type
- Il est interdit de donner a ce type la responsabilite de piloter seul la session
- Il est interdit d'en faire un receptacle de `task_contract`, budget complet, policy riche ou metadonnees de persistence
- Il est interdit d'ajouter des variantes, champs ou capacites "au cas ou"
- Il est interdit d'utiliser ce point de decision pour rouvrir la modelisation globale de `Verdict`

Regle executable generale:

- toute proposition qui transforme ce type minimal en pont conceptuel, en type fourre-tout, ou en redesign du domaine est non conforme et doit etre rejetee

## 8. Implications directes pour l'implementation du slice 3

Les implications suivantes sont immediates et contraignantes:

- les premiers tests du slice 3 doivent forcer l'existence de ce signal runtime explicite au niveau `application`
- les tests ne doivent pas s'appuyer sur une convention cachee dans `Verdict` existant
- les tests doivent prouver que `Critic` emet ce signal runtime minimal de facon explicite
- les tests doivent prouver que `Planner` lit le contexte runtime approprie pour produire `SessionFlowDecision::Retry`, sans que ce nouveau type soit lui-meme une `SessionFlowDecision`
- `SessionRunner` doit rester l'orchestrateur unique de la boucle runtime; ce nouveau type ne doit jamais devenir un substitut d'orchestration
- toute implementation doit rester minimale et limiter ce nouveau type au perimetre necessaire pour faire passer les tests du slice 3

## 9. Sortie attendue de cette decision

Cette decision est correctement appliquee si, et seulement si, les assertions suivantes restent vraies pendant l'implementation du slice 3:

- un type runtime minimal dedie existe pour le signal du `Critic`
- ce type est situe en `application`
- `Verdict` existant n'a pas ete reutilise strictement comme contrat runtime du slice 3
- aucun pont conceptuel n'a ete introduit avec `HandoffDecision`
- aucune projection implicite n'a ete introduite avec `SessionFlowDecision`
- aucune inflation de modele n'a ete ajoutee au-dela du besoin strict du slice 3
