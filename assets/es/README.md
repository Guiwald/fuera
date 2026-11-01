# Contenus d'espagnol

Écrits pour ¡Fuera! (espagnol d'Espagne, niveaux A1 à B2), puis vérifiés automatiquement :

- `lexicon_a.tsv` (A1–A2, 1 062 mots) et `lexicon_b.tsv` (B1–B2, 1 020 mots) : sens en français et en anglais,
  genre, thème, image (émoji), notes (faux amis, variantes d'Amérique latine). Chaque mot existe dans le Wiktionnaire
  avec la même nature, et le genre de chaque nom y a été comparé.
- `grammar/` : ser/estar (90), por/para (90), pretérito/imperfecto (90), subjonctif/indicatif (110), faux amis (70),
  genres trompeurs (70), et 200 pseudo-mots pour le test « ¿Existe? » (absents des deux sources ci-dessous).

Le format est décrit dans `docs/data/SPEC-es.md`. Les scripts de vérification sont dans `tools/data/checks/`.

**Sources et licence.** Ordre de fréquence : liste *FrequencyWords* d'Hermit Dave (sous-titres OpenSubtitles 2018,
CC BY-SA 4.0). Vérifications : Wiktionnaire anglais extrait par kaikki.org (CC BY-SA 4.0). Ces fichiers sont donc
publiés sous **CC BY-SA 4.0** (https://creativecommons.org/licenses/by-sa/4.0/), séparément du code (MIT).

À faire avant de les considérer comme définitifs : une relecture par une personne de langue maternelle espagnole
(les auteurs ont noté les cas discutables dans le journal du projet).
