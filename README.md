# ¡Fuera!

**Fais sortir chaque bloc par la porte de sa couleur — et apprends l'espagnol en jouant.**

Un bloc qu'on lance glisse jusqu'à ce qu'il bute ; s'il arrive face à une porte de sa couleur, il sort. Le plateau est
encombré : tout l'art est dans l'ordre des coups. Le mode espagnol garde la même mécanique, mais les blocs portent des
mots et les portes des sens, des personnes, *ser* ou *estar*… et une mauvaise porte renvoie le bloc en montrant la
bonne réponse.

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

## Le mode espagnol

- **Test « ¿Existe? »** (3 minutes) : des vrais mots et des mots inventés ; il place ton niveau (A1 à B2) et marque
  les mots plus faciles comme connus.
- **Séance du jour** : 5 manches qui alternent vocabulaire et grammaire. Une série de jours, avec un jour manqué
  pardonné.
- **Modes** : Palabras (mot ↔ sens), Escucha (à l'oreille), El o la, Verbos (sept temps, *vosotros* compris), Ser o
  estar, Por o para, Pasado (*pretérito* ou *imperfecto*), Subjuntivo, Falsos amigos.
- **Mi cuaderno** : les mots ratés, à réécouter.
- Les révisions sont espacées (algorithme FSRS) : un mot revient juste avant que tu l'oublies. Sens en français ou en
  anglais. Prononciation : voix espagnole du système (sur Mac, Réglages → Accessibilité → Contenu énoncé pour en
  télécharger une plus belle).

## Pour les développeurs

```
cargo run                                   # le jeu (bureau)
cargo test --workspace --release            # tests (règles, solveur, conjugueur, FSRS, planificateur)
cargo run --release -p fuera-cli -- check assets/levels/campaign.txt   # chaque niveau est résolu
cargo run --release -p fuera-cli -- campaign assets/levels/campaign.txt   # régénère la campagne
cargo xtask web | apk | bundle-mac <bin> | dist-windows | icons
```

Architecture et décisions : `docs/adr/`. Licences : code MIT, polices OFL, contenus d'espagnol CC BY-SA 4.0
(`THIRD_PARTY.md`).

---

## English

**Slide every block out through the door of its colour — and learn Spanish while you play.**

A block you swipe slides until it hits something; if it reaches a door of its colour, it leaves. The board is
crowded, so the order of your moves is the puzzle. The Spanish mode keeps the same mechanic: blocks carry words, doors
carry meanings, persons, *ser* or *estar*…, and a wrong door bounces the block back and shows the right answer.

Free, no ads, no account, offline. Mac, Windows, Linux, Android, and in the browser at **https://fuera.librejeu.fr**.
Downloads are on the [releases page](https://github.com/Guiwald/fuera/releases/latest).

Spanish mode: a 3-minute "¿Existe?" placement test (A1–B2), a 5-round daily session, nine modes (vocabulary,
listening, gender, verbs in seven tenses, ser/estar, por/para, preterite/imperfect, subjunctive, false friends), a
notebook of missed words, spaced repetition (FSRS), meanings in English or French, system voices for pronunciation.

Code: MIT. Fonts: SIL OFL. Spanish content: CC BY-SA 4.0. See `THIRD_PARTY.md`.
