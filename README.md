# ¡Fuera!

**Fais sortir chaque bloc par la porte de sa couleur — et apprends l'espagnol ou le français en jouant.**

Un bloc qu'on lance glisse jusqu'à ce qu'il bute ; s'il arrive face à une porte de sa couleur, il sort. Le plateau est
encombré : tout l'art est dans l'ordre des coups. Le mode langue garde la même mécanique, mais les blocs portent des
mots et les portes des sens, des personnes, *ser* ou *estar*, *un* ou *une*… et une mauvaise porte renvoie le bloc en
montrant la bonne réponse. Trois cours : espagnol d'Espagne, espagnol d'Amérique latine, français.

Gratuit, sans pub, sans compte, hors ligne. Mac, Windows, Linux, Android, et dans le navigateur :
**https://fuera.librejeu.fr**.

*English speakers: see [the English section](#english).*

## Jouer

| Où | Quoi |
|---|---|
| Navigateur | https://fuera.librejeu.fr |
| Mac | `Fuera-X.Y.Z.dmg` sur la [page des versions](https://github.com/Guiwald/fuera/releases/latest) ; ouvrir, glisser dans Applications. La première fois : clic droit → Ouvrir (l'appli n'est pas signée par Apple). |
| Windows | `Fuera-X.Y.Z-windows-x64.zip` ; dézipper, lancer `Fuera.exe` (« Informations complémentaires » → « Exécuter quand même »). |
| Android | `Fuera-X.Y.Z.apk` ; l'ouvrir sur le téléphone et autoriser l'installation. |
| Linux | `Fuera-X.Y.Z-linux-x86_64.tar.gz`. |

**Commandes** : glisser un bloc (doigt ou souris). Au clavier : Tab choisit un bloc, les flèches le lancent, Z annule,
R recommence, H donne un indice, Échap revient en arrière.

## Les cours de langue

« Apprendre une langue » propose trois cours ; chacun garde ses progrès.

- **Espagnol d'Espagne** (sens en français ou en anglais) et **espagnol d'Amérique latine** (*ustedes* sans
  *vosotros*, *carro*, *computadora*…). Modes : Palabras (mot ↔ sens), Escucha (à l'oreille), El o la, Verbos (sept
  temps), Ser o estar, Por o para, Pasado (*pretérito* ou *imperfecto*), Subjuntivo, Falsos amigos.
- **Français** (sens en espagnol ou en anglais) : Mots, Écoute, Un ou une, Verbes (six temps), Du, de la, des, Être
  ou avoir, Passé (passé composé ou imparfait), Subjonctif, Faux amis.
- Pour chaque cours : test de niveau (« ¿Existe? », « Ça existe ? », A1 à B2), séance du jour de 5 manches, série
  de jours (un jour manqué pardonné), cahier des mots ratés, révisions espacées (algorithme FSRS).
- **Prononciation** : voix humaines enregistrées par les bénévoles de Lingua Libre pour les mots isolés (Espagne,
  Amérique latine ou France selon le cours), voix du système pour le reste.
- **Ton avis** : le bouton bulle ouvre le menu de partage du téléphone avec un message déjà rempli ; le drapeau
  signale un mot mal prononcé ou mal traduit. Sur ordinateur : fuera@librejeu.fr.

## Pour les développeurs

```
cargo run                                   # le jeu (bureau)
cargo test --workspace --release            # tests (règles, solveur, conjugueur, FSRS, planificateur)
cargo run --release -p fuera-cli -- check assets/levels/campaign.txt   # chaque niveau est résolu
cargo run --release -p fuera-cli -- campaign assets/levels/campaign.txt   # régénère la campagne
cargo run --release -p fuera-cli -- prove     # preuve : aucun plateau du mode espagnol n'a d'impasse
cargo run --release -p fuera-cli -- deadends assets/levels/campaign.txt   # impasses de chaque niveau
cargo xtask web | apk | bundle-mac <bin> | dist-windows | icons
```

Architecture et décisions : `docs/adr/`. Licences : code MIT, polices OFL, contenus d'espagnol et de français
CC BY-SA 4.0, voix CC BY-SA, CC BY ou CC0 selon l'enregistrement (`THIRD_PARTY.md`).

---

## English

**Slide every block out through the door of its colour — and learn Spanish or French while you play.**

A block you swipe slides until it hits something; if it reaches a door of its colour, it leaves. The board is
crowded, so the order of your moves is the puzzle. The language mode keeps the same mechanic: blocks carry words, doors
carry meanings, persons, *ser* or *estar*, *un* or *une*…, and a wrong door bounces the block back and shows the right
answer. Three courses: Spanish from Spain, Latin-American Spanish, and French.

Free, no ads, no account, offline. Mac, Windows, Linux, Android, and in the browser at **https://fuera.librejeu.fr**.
Downloads are on the [releases page](https://github.com/Guiwald/fuera/releases/latest).

Courses: Spanish (Spain or Latin America, meanings in English or French) and French (meanings in English or Spanish),
each with a 3-minute placement test (A1–B2), a 5-round daily session, nine modes (vocabulary, listening, gender, verbs,
and each language's own traps: ser/estar, por/para, preterite/imperfect, subjunctive; partitives, être/avoir, passé
composé/imparfait, subjonctif; false friends), a notebook of missed words and spaced repetition (FSRS). Single words
are spoken by Lingua Libre volunteers, the rest by your system's voice. A feedback button opens your phone's share
menu with a prefilled message (or a mail to fuera@librejeu.fr on a computer).

Code: MIT. Fonts: SIL OFL. Spanish and French content: CC BY-SA 4.0. Voices: CC BY-SA, CC BY or CC0 per recording. See `THIRD_PARTY.md`.
