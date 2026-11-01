# ADR 0003 — Solveur A* et générateur de niveaux

- Statut : acceptée (31/10/2026)

## Décision

- **Solveur** : A* (`crates/core/src/solver.rs`) avec une heuristique admissible et cohérente : pour chaque bloc, le
  plus petit nombre de glissements qu'il lui faudrait seul sur le plateau en pouvant s'arrêter n'importe où. Les blocs
  identiques sont interchangeables (états canoniques). Vérifié contre une recherche en largeur sur des centaines de
  niveaux aléatoires (`astar_matches_bfs`).
- **Indice** : la solution optimale depuis l'état courant, gardée tant que le joueur la suit.
- **Générateur** : plateau découpé (U, L, anneau, croix…), portes sur le pourtour, blocs placés au hasard, éléments
  spéciaux, puis résolution. La difficulté est mesurée par le **surplus** (coups optimaux − borne de l'heuristique :
  combien les blocs se gênent), la longueur, les impasses. La campagne suit une courbe en dents de scie avec un
  niveau difficile tous les 5 et un élément nouveau introduit seul.
- `fuera-cli` : `check` (chaque niveau validé et résolu), `campaign` (reconstruit la campagne), `gen`, `solve`.

## Conséquences

- L'indice le plus lent de la campagne prend environ 0,4 s sur ordinateur.
