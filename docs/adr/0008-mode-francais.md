# ADR 0008 — Le cours de français

- Statut : acceptée (01/11/2026)
- Contexte : ADR 0007 (le cours comme paramètre). Apprenant visé d'abord : un jeune hispanophone du Honduras,
  débutant complet, sur son téléphone ; ouvert aussi aux anglophones. Français de France, graphie traditionnelle.

## Décision

1. **Mêmes règles que le mode espagnol** (ADR 0004 et 0006) : la porte est la règle de langue, une mauvaise porte
   renvoie le bloc, la note vient du jeu (FSRS), et chaque manche tombe sur une disposition prouvée sans impasse
   (test `every_round_fits_a_proven_layout`).
2. **Modes, nommés en français** :

   | Mode | Portes | Entre dans la séance |
   |---|---|---|
   | Mots, Écoute | sens ↔ mot ; à l'oreille | A1 |
   | Un ou une | « un » en haut, « une » en bas | A1 |
   | Verbes | personnes (*je*, *tu*, *il / elle*, *nous*, *vous*, *ils / elles*) | A1 (présent) |
   | Du, de la, des | deux phrases à trou + « aucune » (moule de « Pasado ») | A1 |
   | Faux amis | vrai sens et sens piège | A1 |
   | Être ou avoir | « être » en haut, « avoir » en bas, phrases au passé composé | A2 |
   | Passé | passé composé ou imparfait, deux phrases + « aucune » | A2 |
   | Subjonctif | subjonctif ou indicatif, deux phrases + « aucune » | B1 |
   | Ça existe ? | test de niveau, vrais mots et pseudo-mots | — |

   - **Un ou une** plutôt que *le / la* : l'élision (*l'arbre*) cache le genre de 20 % des noms ; *un / une* est
     toujours une vraie forme. Le retour montre les deux (*un arbre · l'arbre*).
   - **Être ou avoir** sur des phrases, pas des infinitifs : les verbes doubles (*je suis sorti* / *j'ai sorti la
     poubelle*) et les pronominaux en dépendent. Les tuiles longues passent sur deux lignes.
   - **Partitifs** dans le moule « Pasado » : cinq formes (*du, de la, de l', des, de*) ne tiennent pas sur quatre
     portes prouvées ; deux phrases + « aucune » les couvrent toutes, avec le piège des hispanophones
     (*j'aime le café* / *je bois du café*, *pas de*).
3. **Conjugueur français maison** (`crates/learn/src/conj_fr.rs`) : six temps (présent, passé composé, imparfait,
   futur, conditionnel, subjonctif présent), règles du premier groupe (*commençons, mangions, lève, appelle, cède /
   céderai, nettoie, enverrai*), deuxième groupe avec ses faux amis du troisième, modèles du troisième groupe et
   leurs composés, pronominaux (*me suis levé(e)*, *m'appelle*), accord noté comme dans les grammaires
   (*sommes allé(e)s*). Vérifié contre les tables du Wiktionnaire anglais (kaikki.org) : environ 25 600 cases pour
   830 verbes (les 600 plus fréquents, une liste de verbes irréguliers et tous les verbes du lexique), toutes justes
   (`tests/conj_fr_wiktionary.rs`).
4. **Contenus** (`assets/fr/`, CC BY-SA 4.0) : 2 253 mots A1–B2 (485 A1, 719 A2, 603 B1, 446 B2), gloses en
   espagnol (Espagne|Amérique) et en anglais ; genres trompeurs (84), être/avoir (93), partitifs (91), passé (90),
   subjonctif (100), faux amis (83, dont 61 pour hispanophones), 200 pseudo-mots. Rédigés pour le jeu, vérifiés
   automatiquement (`tools/data/fr/check_fr.py` : nature, genre, *h* aspiré, gloses contre les traductions des deux
   Wiktionnaires, formes contre le conjugueur), puis relus par un second rédacteur. Les questions qui demandent un
   locuteur natif sont dans le « cahier des doutes » (document partagé avec Guillaume).
5. **Écoute sans homophones** : les mots qui se prononcent comme un autre mot du lexique (*verre, vert, vers*) ne
   sont jamais joués à l'oreille (`assets/fr/homophones.txt`, d'après l'API du Wiktionnaire français).
6. **Notes dans la langue de l'apprenant** : « irrégulier » devient *irregular*, « pluriel » *plural* ; une mise en
   garde pour l'autre langue de base (`≠ EN: …` pour un hispanophone) est masquée.
7. **Voix** : enregistrements Lingua Libre de locuteurs de France (hors Midi d'abord, puis Midi), puis de Suisse et
   de Belgique ; jamais du Québec ni « non-native ». L'origine d'un locuteur vient du Wiktionnaire français, où le
   robot de Lingua Libre range chaque enregistrement avec le lieu du locuteur. 2 267 mots (99 %), 26 voix ; chaque
   fichier passé dans une reconnaissance vocale hors ligne (Vosk), 26 remplacés par un enregistrement mieux reconnu.

## Conséquences

- Pour un hispanophone, l'interface entière est en espagnol, les gloses en espagnol d'Amérique latine si l'appareil
  est réglé sur un pays d'Amérique.
- Le test de niveau s'arrête à B2 comme pour l'espagnol.
