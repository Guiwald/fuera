# ADR 0006 — Modèle mathématique : plateaux sans impasse, prouvés

- Statut : acceptée (01/11/2026)
- Contexte : retours de jeu sur la 1.0 (mode espagnol). Dans « ¿Existe? », une tuile ne pouvait pas entrer dans sa
  porte. Dans « Palabras », envoyer « el arma » vers « weapon » demandait d'utiliser une autre tuile comme butée. Dans
  « Por o para », on se sentait bloqué. Les étiquettes étaient aussi en double (« no existe » deux fois en haut,
  « existe » deux fois en bas).

## Le jeu vu comme un objet mathématique

Un niveau est un **graphe d'états** : un état = la position de chaque bloc encore présent, une arête = un glissement.
Gagner, c'est atteindre un état sans bloc.

- **Gagnable** : il existe un chemin de l'état de départ vers la victoire (atteignabilité). Le solveur A* le décide
  (ADR 0003).
- **Sans impasse** : depuis *tout* état atteignable, la victoire reste atteignable (co-atteignabilité). C'est une
  propriété plus forte, et c'est elle qui compte pour le joueur. `solver::analyze` parcourt tout le graphe et compte
  les états morts par un parcours en arrière depuis les victoires. Depuis cette version, le compte reste exact même
  quand le budget d'exploration est dépassé : un état n'est déclaré mort que si ni une victoire ni un état non exploré
  ne lui sont accessibles.
- **Complexité** : les puzzles de blocs glissants sont PSPACE-complets en général (Hearn et Demaine, logique de
  contraintes non déterministe). Les jeux « sur la glace » (glisser jusqu'à l'obstacle) sont polynomiaux avec un seul
  mobile et des obstacles fixes, mais deviennent NP-difficiles ou PSPACE-complets dès que plusieurs mobiles se servent
  de butée les uns aux autres (Dorbec et al., *Ice sliding games*). Il n'y a donc pas de formule magique pour un
  plateau quelconque : soit on explore, soit on **restreint la géométrie** jusqu'à pouvoir tout prouver. C'est ce
  qu'on fait pour le mode espagnol.

## Géométrie : où une tuile peut-elle s'arrêter ?

Une tuile glisse jusqu'à un mur ou une autre tuile. Les **points d'arrêt** sont donc les cases collées à un mur ou à
une tuile. Deux conséquences :

1. **Une porte doit être sur un point d'arrêt.** Sur un plateau de 3 colonnes, la colonne du milieu n'est jamais un
   point d'arrêt pour un glissement latéral : la tuile passe de gauche à droite d'un trait. Une porte au-dessus de
   cette colonne n'est atteignable qu'avec une butée. Une tuile seule ne peut pas y entrer. C'était le bug de
   « ¿Existe? » et la difficulté de « el arma ».
2. **En mode langue, une mauvaise porte ne retient pas** (la tuile revient, `WrongDoor::Bounce`). Un mur entièrement
   couvert de portes n'offre aucun point d'arrêt vertical. Deux tuiles côte à côte qui veulent chacune la colonne de
   l'autre sont bloquées pour toujours. Il faut des **bouts de mur nu** pour se garer.

**Réseau (lattice).** Toutes les tuiles d'une manche ont la largeur d'une porte et partent sur la grille des portes.
Les points d'arrêt (murs latéraux, murs nus, autres tuiles) sont sur cette grille, donc une tuile reste toujours dans
l'une des `colonnes × 7` cases du réseau. Elle ne s'arrête jamais à cheval sous une porte. Le plateau devient un
objet fini et petit : on peut énumérer **toutes** ses positions.

## Preuve par énumération (`crates/core/src/lattice.rs`)

`lattice::prove` passe en revue toutes les façons de poser de 1 à *k* tuiles sur le réseau, avec toutes les
attributions de portes cibles. Il vérifie que chaque position peut encore être gagnée. Le raisonnement est une
récurrence sur le nombre de tuiles : une position est gagnable si une suite de coups permet à une tuile de sortir
par sa porte, car la position restante, avec une tuile de moins, est gagnable par l'étape précédente. Le calcul
avance par couches (point fixe) : couche 0 = une tuile peut sortir tout de suite, couche n + 1 = un coup mène à la
couche n. Ce qui reste à la fin est une impasse, affichée comme contre-exemple.

C'est une preuve « assistée par ordinateur » au sens des preuves exhaustives : elle vaut pour **tous** les plateaux
ayant cette géométrie, quels que soient les mots. Le test `lattice_model_agrees_with_the_rules` vérifie que le modèle
abstrait et les vraies règles du jeu disent la même chose.

Résultats (`fuera-cli prove`, `fuera-cli prove compare`) :

| Disposition | Résultat |
|---|---|
| 1.0 : 5 portes sur 3 colonnes | **impasse dès 1 tuile** (porte du milieu inatteignable) |
| 1.0 : 4 portes sur 2 colonnes | **impasse dès 2 tuiles** (échange de colonnes impossible) |
| 2 catégories, portes sur toute la largeur, tuiles de 3 | **impasse avec 4 tuiles** (2 rangées qui se croisent) |
| 2 catégories, portes sur toute la largeur, tuiles de 2 | **impasse avec 6 tuiles** (3 + 3) |
| **2 catégories « en diagonale »**, tuiles de 2 ou de 3 | aucune impasse jusqu'à 6 tuiles |
| **3 ou 4 portes dans les coins**, colonne de parking | aucune impasse jusqu'à 5 tuiles (20,8 millions de positions) |

Les dispositions retenues :

- **Deux catégories** (el/la, ser/estar, por/para, existe/no existe) : **une seule porte par catégorie**, toujours la
  première en haut et la seconde en bas. Chaque porte laisse un bout de mur nu à l'autre extrémité : la porte du haut
  couvre la gauche, celle du bas la droite. Chaque colonne latérale a un mur nu où s'arrêter, donc deux tuiles qui
  doivent se croiser peuvent toujours s'écarter.
- **Trois ou quatre portes** (sens des mots, personnes, faux amis, phrases + « ninguna ») : portes de deux cases dans
  les **coins**, mur nu au-dessus et au-dessous de la colonne du milieu. Un glissement latéral s'arrête toujours contre
  un mur, juste sous une porte. La colonne du milieu sert de **parking** : on y glisse verticalement jusqu'au mur.
- Tailles des manches limitées à ce qui est prouvé : 3 tuiles (mots, verbes), 2 (faux amis), 4 (passé et subjonctif,
  désormais 2 phrases × 2 formes), 5 (ser/estar, por/para), 6 (genre, ¿Existe?). Au-delà, le générateur reviendrait à
  l'exploration complète du graphe (`word_check`).

Coût : préparer une manche prend quelques millisecondes (contre plusieurs secondes avant), puisque l'absence
d'impasse n'est plus à rechercher plateau par plateau. Seule la solution optimale est calculée.

## Pourquoi une porte par catégorie, toujours au même endroit

En psychologie cognitive, une correspondance stimulus → réponse **constante** (*consistent mapping*) devient
automatique avec l'entraînement. Une correspondance **variable** (*varied mapping*) demande toujours une attention
contrôlée (Schneider et Shiffrin, 1977). Si « por » change de place à chaque manche, une partie de l'attention part à
chercher la porte au lieu de réfléchir à la phrase. Avec « por » toujours au même endroit, le geste s'automatise et
toute l'attention va à la langue. Deux étiquettes identiques, en plus, doublent la lecture sans rien apprendre.

## Difficulté du casse-tête : une rampe séparée

La difficulté se mesure par le **surplus** (coups optimaux − borne inférieure de l'heuristique : les détours imposés
par les autres tuiles, ADR 0003). En mode langue, le casse-tête doit rester au service de la langue. Le surplus visé
dépend du nombre de manches jouées (`Memory::board_slack`) :

- moins de 10 manches : 0 (aucun détour) ;
- moins de 30 : 0 à 1 ;
- moins de 80 : 0 à 2 ;
- au-delà : 1 à 3.

Au moins un coup de plus que le nombre de tuiles reste exigé, pour qu'il y ait toujours un peu à faire glisser.

## Mode classique

La campagne n'a pas de géométrie restreinte (formes, glace, carapaces, verrous…), donc pas de preuve générale. Les
impasses y font partie du jeu (comme au Sokoban, où l'on distingue cases mortes, blocs gelés, enclos…). Désormais :

- après un coup qui s'écarte de la solution connue, le jeu cherche une solution depuis la position courante (budget
  de 25 000 états, sans à-coup). S'il **prouve** qu'il n'y en a plus, il affiche « Impasse : plus de solution d'ici.
  Annule. » au lieu de laisser le joueur tourner en rond. S'il en trouve une, elle sert directement au prochain indice ;
- `fuera-cli deadends` mesure, niveau par niveau, la part de positions mortes et les premiers coups perdants. Il a
  montré que **le tout premier tutoriel avait 96 % de positions mortes** : 4 des 7 premiers coups possibles perdaient.
  Le bloc rouge devait sortir par une porte d'une case au milieu du mur droit, et une fois monté ou descendu, plus rien
  ne pouvait l'arrêter à cette hauteur. Ses portes couvrent maintenant des côtés entiers : aucune impasse possible.
  Le tutoriel 3 garde volontairement quelques impasses (il apprend à annuler), sans qu'aucun premier coup ne perde.

## Sources

- Hearn, Demaine — *PSPACE-Completeness of Sliding-Block Puzzles and Other Problems through the Nondeterministic
  Constraint Logic Model of Computation* : https://arxiv.org/abs/cs/0205005
- Dorbec et al. — *Ice sliding games* : https://arxiv.org/abs/1507.00559
- Schneider, Shiffrin (1977) — *Controlled and automatic human information processing* :
  https://app.nova.edu/toolbox/instructionalproducts/edd8124/articles/1977-Schneider_and_Shiffrin.pdf
- Typologie des impasses au Sokoban (JSoko) : https://jsokoapplet.sourceforge.io/sokoban/deadlocks.html
