# Journal des versions

## 1.1.0

- **Apprendre le français**, depuis l'espagnol ou l'anglais : 2 253 mots du A1 au B2 (sens en espagnol d'Espagne
  ou d'Amérique latine, et en anglais), et neuf modes nommés en français : Mots, Écoute, Un ou une, Verbes (six
  temps, accord du participe noté *allé(e)s*), Du, de la, des, Être ou avoir, Passé (passé composé ou imparfait),
  Subjonctif, Faux amis (pièges pour hispanophones et anglophones), plus le test « Ça existe ? ». Conjugueur français
  vérifié sur 25 600 formes du Wiktionnaire. 2 267 mots (99 %) prononcés par des bénévoles de Lingua Libre de France
  (puis de Suisse et de Belgique), chaque enregistrement contrôlé par une reconnaissance vocale hors ligne ; voix
  française du système pour le reste.
- **Espagnol d'Amérique latine** : *ustedes* sans *vosotros*, *carro*, *computadora*, *celular*…, phrases propres à
  l'Espagne écartées, 1 769 mots (86 %) prononcés par des voix d'Amérique latine d'abord (Mexique, Amérique
  centrale, puis Andes et Caraïbes). Les progrès sont partagés avec l'espagnol d'Espagne.
- **Choisir sa langue** : « Apprendre une langue » ouvre un sélecteur (le cours qui correspond à la langue de
  l'appareil est proposé en premier). Chaque langue garde ses progrès ; une sauvegarde 1.0 reste en espagnol
  d'Espagne. Réglage « Espagnol : Espagne / Amérique latine » (variante enseignée et gloses).
- **Donner son avis** : un bouton ouvre le menu de partage du téléphone (WhatsApp, mail…) avec un message déjà
  rempli ; on peut parler avec le micro de sa messagerie. Un drapeau signale un mot (« la voix sonne mal »,
  « traduction bizarre ») sur la carte de correction, le résumé de manche et le cahier. Sur ordinateur, le message
  part par mail à fuera@librejeu.fr.
- Phrases longues sur deux lignes dans les tuiles ; notes traduites dans la langue de l'apprenant.
- Voix d'Espagne : les enregistrements de deux locuteurs probablement non natifs sont retirés (1 470 mots, 70 %).
- Les paquets de voix (Espagne, Amérique latine, France) sont tous dans les applis ; sur le web, seul celui du cours
  joué est téléchargé.

## 1.0.1

- **Voix humaines** : 1 516 mots du mode espagnol (72 %) sont prononcés par des bénévoles de Lingua Libre (Wikimedia
  Commons, licences libres), au lieu de la voix de synthèse. Une voix d'Espagne est prise en priorité ; un autre
  accent n'est retenu que pour les mots qui se prononcent pareil des deux côtés de l'Atlantique (sans *z*, *ce/ci*,
  *ll* ni *y* consonne). Les phrases et les autres mots restent lus par la voix du système. Auteurs et fichiers
  d'origine : `assets/es/voces/creditos.tsv`, et dans l'écran Crédits.

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
- Toucher une tuile la lit à voix haute **sans donner la réponse** (le mot sans son article, la forme sans sa
  personne, la phrase avec un blanc). Réglage « Toucher lit la réponse » pour l'entendre en entier : la tuile compte
  alors comme aidée, comme avec l'ampoule.
- Mode classique : le jeu signale une impasse (« plus de solution d'ici ») dès qu'il peut la prouver.
- Mac, Windows, Linux, Android (Pixel 8), navigateur. Interface en français, anglais et espagnol.
