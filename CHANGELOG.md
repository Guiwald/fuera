# Journal des versions

## 1.0.0

Première version.

- **Mode classique** : 100 niveaux (trois tutoriels faits main, 97 fabriqués par le générateur puis vérifiés par le
  solveur), avec murs intérieurs, gros blocs, glace, blocs sur rail, blocs à deux couches, chaînes et clés, volets.
  Annuler sans limite, recommencer, indice (le prochain coup d'une solution optimale), étoiles selon le nombre de
  coups.
- **Mode espagnol** : les blocs portent des mots, les portes des sens ou des catégories ; une mauvaise porte renvoie
  le bloc et montre la bonne réponse. Palabras (sens ↔ mot), Escucha (à l'oreille), El o la, Verbos (sept temps),
  Ser o estar, Por o para, Pasado (pretérito/imperfecto), Subjuntivo, Falsos amigos. Test de niveau « ¿Existe? »,
  séance du jour (5 manches), série de jours, « Mi cuaderno » (les mots ratés), sens en français ou en anglais.
  Révisions espacées (FSRS-6), voix du système pour la prononciation.
- **Plateaux du mode espagnol prouvés sans impasse** : une seule porte par catégorie, toujours au même endroit
  (« por » en haut, « para » en bas…) ; trois ou quatre portes dans les coins avec une colonne de parking ; pour
  chaque disposition utilisée, toutes les positions possibles ont été vérifiées gagnables (`fuera-cli prove`, voir
  `docs/adr/0006-modele-mathematique.md`). Le casse-tête reste simple au début et se corse avec les manches jouées.
- Mode classique : le jeu signale une impasse (« plus de solution d'ici ») dès qu'il peut la prouver.
- Mac, Windows, Linux, Android (Pixel 8), navigateur. Interface en français, anglais et espagnol.
