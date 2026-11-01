# ADR 0004 — Le mode espagnol

- Statut : acceptée (31/10/2026)

## Contexte

Recherche détaillée : `fuera/research-spanish-learning.md` (projet). Apprenant visé d'abord : Guillaume, B1/B2,
francophone, en Espagne ; ouvert à tous. Espagnol d'Espagne.

## Décision

1. **La règle de la porte est la règle de langue** (intégration intrinsèque, Habgood & Ainsworth 2011) : la couleur
   ne donne jamais la réponse (blocs pastel au hasard, portes toutes pareilles, pas de lueur sur la bonne porte),
   toujours plusieurs portes possibles, des portes leurres.
2. **Mauvaise porte = rebond**, avec la bonne réponse et sa prononciation ; pas de vies.
3. **Notation tirée du jeu** : première réponse fausse → *Again*, après un indice → *Hard*, juste → *Good* ;
   appliquée une seule fois à la fin de la manche. **FSRS-6** (`crates/learn/src/fsrs.rs`) choisit les révisions.
4. Modes 1.0 : Palabras, Escucha, El o la, Verbos (conjugueur maison vérifié sur 21 049 formes du Wiktionnaire), Ser o
   estar, Por o para, Pasado, Subjuntivo, Falsos amigos ; test « ¿Existe? » (vrais mots et pseudo-mots) ; séance du
   jour de 5 manches, série de jours (un jour manqué pardonné), « Mi cuaderno ».
5. **Contenus** écrits pour le jeu et vérifiés automatiquement (`assets/es/`, CC BY-SA 4.0) ; rien sous licence
   non commerciale.

## Plus tard (1.1)

Profils et mode enfant (« Pequeños »), phrases à remettre en ordre (Tatoeba), « Simón dice », enregistrements
humains (Lingua Libre) à la place de la voix du système, images (émojis Twemoji/Fluent) sur les portes, Amérique
latine puis d'autres langues.

*Mise à jour (01/11/2026)* : les voix humaines sont venues avec la 1.0.1, l'espagnol d'Amérique latine et le
français avec la 1.1 (ADR 0007 et 0008). Les profils, « Pequeños », Tatoeba, « Simón dice » et les images restent à
faire (1.2 et après).
