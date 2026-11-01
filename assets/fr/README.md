# Contenus de français

Écrits pour ¡Fuera! (français de France, graphie traditionnelle, niveaux A1 à B2), puis vérifiés automatiquement et
relus :

- `lexicon_a.tsv` (A1–A2, 1 204 mots) et `lexicon_b.tsv` (B1–B2, 1 049 mots) : sens en espagnol (Espagne et
  Amérique latine quand le mot courant diffère : `coche|carro`) et en anglais, genre, thème, image (émoji), notes
  (*h* aspiré, féminins irréguliers, faux amis, verbes irréguliers).
- `grammar/` : genres trompeurs (84), être ou avoir au passé composé (93), partitifs (91), passé composé ou imparfait
  (90), subjonctif ou indicatif (100), faux amis pour hispanophones et anglophones (83), et 200 pseudo-mots pour le
  test « Ça existe ? » (absents des deux Wiktionnaires et de la liste de fréquences).
- `homophones.txt` : les mots du lexique qui se prononcent comme un autre (jamais joués à l'oreille).
- `voces/` : prononciations enregistrées (voir `voces/README.md`).

Le format est décrit dans `docs/data/SPEC-fr.md`. Les scripts de vérification et de construction sont dans
`tools/data/fr/`.

**Sources et licence.** Ordre de fréquence : liste *FrequencyWords* d'Hermit Dave (sous-titres OpenSubtitles 2018,
CC BY-SA 4.0), rattachée aux lemmes du Wiktionnaire anglais. Vérifications : Wiktionnaire anglais et Wiktionnaire
français extraits par kaikki.org / wiktextract (CC BY-SA 4.0 et 3.0). Ces fichiers sont donc publiés sous
**CC BY-SA 4.0** (https://creativecommons.org/licenses/by-sa/4.0/), séparément du code (MIT).

À faire avant de les considérer comme définitifs : la relecture des points du « cahier des doutes » par une personne
de langue maternelle française, et l'avis du premier joueur hispanophone.

---

French content written for ¡Fuera! (France's French, traditional spelling, A1–B2), with Spanish and English meanings,
checked against the English and French Wiktionaries (kaikki.org / wiktextract) and ordered by Hermit Dave's
FrequencyWords. Released under CC BY-SA 4.0, separately from the MIT code.
