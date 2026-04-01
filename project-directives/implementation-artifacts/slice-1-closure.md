# Slice 1 Closure

## 1. Ce que le slice 1 prouve reellement

- Le coeur du runtime tient en memoire sur un flux minimal de bout en bout.
- Les invariants de base sont proteges: `TaskContract`, `Verdict`, cycle terminal de `Session`.
- La machine d'etats impose les transitions autorisees et bloque des transitions invalides critiques.
- `SessionRunner` orchestre correctement le chemin nominal et les arrets terminaux.
- Le systeme sait produire un resultat terminal explicite: `SessionSummary` en succes, `FailureReport` en echec.
- Un premier flux de donnees reel existe deja: `ScholarOutput` est propage jusqu'au `Planner`, `Builder` et `Critic`.

## 2. Hors scope explicite

- Toute IO reelle: base, filesystem, shell, git, reseau.
- Tout agent reel, skill reel, snapshot reel, diff reel, event store reel.
- Toute logique de reprise, de persistance ou d'historisation durable.
- Toute detection de stack, validation de schema, ou orchestration infra.
- Toute boucle de revision metier complete avec artefacts inter-iterations.

## 3. Dettes et transitions provisoires tolerees

- Le coeur reste concentre dans `src/lib.rs`; acceptable tant que le perimetre reste petit.
- `ScholarOutput` est volontairement minimal (`content: String`) et ne constitue pas encore un contrat metier stable.
- Les ports sont utiles pour l'orchestration, mais encore trop fins pour modeliser une vraie boucle de revision.
- La preuve E2E couvre surtout le nominal, l'arret sur budget et une propagation de donnees simple.

## 4. Prochain slice recommande

- Introduire une boucle de revision complete en memoire: `Critic` emet des changements requis, `Planner` decide `revise`, `Builder` rejoue avec ces changements, puis terminaison explicite.

## 5. Pourquoi c'est le meilleur levier

- C'est le plus petit pas qui valide la promesse produit au-dela du happy path lineaire.
- Il force les bons contrats entre `Planner`, `Builder` et `Critic` sans introduire d'infrastructure.
- Il eprouve le budget de retry sur un cas metier reel, pas seulement sur un stop artificiel.
- Il reduit le principal risque architecturel actuel: un runtime correct en sequence simple mais encore trop pauvre pour une vraie iteration de travail.
