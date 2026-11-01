# ADR 0001 — Ce qu'on clone, et les règles exactes

- Statut : acceptée (31/10/2026)

## Contexte

Guillaume veut un clone léger de « Block Out! – Color Sort Puzzle » (Grand Games), en Rust, pour Android, Mac,
Windows (et le navigateur, ajouté au premier tour de questions). La recherche (`fuera/research-mechanics.md` dans le
projet) montre deux variantes du genre : *Color Block Jam* (le bloc suit le doigt) et *Block Out!* (le bloc glisse
jusqu'à l'obstacle).

## Décision

1. **Glisser jusqu'à l'obstacle** : un bloc lancé dans une direction avance jusqu'à ce qu'il soit bloqué ; il sort
   si chaque ligne (ou colonne) qu'il occupe finit sur une porte ouverte qui l'accepte (`crates/core/src/rules.rs`).
   Les blocs ne tournent jamais. Les impasses existent : on annule sans limite, sans vies, sans minuteur (sauf défis).
2. **Éléments** de la 1.0 : murs intérieurs, portes de 1 à 3 cases, blocs de 1 à 4 cases, glace (fond après N
   sorties), rail (un seul axe), deux couches (la couche extérieure part à sa porte), chaîne + clé, volet (s'ouvre
   et se ferme à chaque sortie). Ascenseurs et générateurs : plus tard.
3. **Nettoyage juridique** : nom, niveaux, dessins et sons originaux ; un symbole sur chaque couleur (daltonisme).
   Nom : **¡Fuera!** (« Dehors ! »), qui annonce aussi le mode espagnol.

## Conséquences

- Pas de pièces, de vies ni de boosters : l'indice vient du solveur.
