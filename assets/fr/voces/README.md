# Voix enregistrées / Recorded voices

`voces-fr.fvox` contient des prononciations de mots français enregistrées par les bénévoles de
[Lingua Libre](https://lingualibre.org) et publiées sur Wikimedia Commons : 2 267 mots, soit 99 % du lexique du cours
de français. Chaque enregistrement garde sa licence d'origine (CC BY-SA 4.0, CC BY 4.0 ou CC0). Le paquet est diffusé
sous CC BY-SA 4.0. `credits.tsv` donne, pour chaque mot, la personne qui l'a enregistré, la licence et le fichier
d'origine sur Commons.

**Modifications** : son ramené en mono 16 kHz, silences du début et de la fin coupés, volume harmonisé (EBU R128,
−18 LUFS), compression IMA ADPCM (format décrit dans `crates/learn/src/voice.rs`).

**Choix des voix** (français de France) :

- L'origine de chaque locuteur vient du Wiktionnaire français : le robot de Lingua Libre y range chaque
  enregistrement avec le lieu du locuteur (« France (Lyon) », « Suisse (Lausanne) », « Canada (Shawinigan) »).
- Ordre de préférence : France hors Midi, puis Midi de la France, puis Suisse et Belgique. Jamais le Québec ni un
  autre pays, ni un locuteur sans lieu connu, ni un enregistrement classé « Non-native ». Le Québec pourra venir
  plus tard dans un paquet à part (`fr-CA`) avec un réglage « Accent ».
- À rang égal, le locuteur qui a enregistré le plus de mots du jeu passe d'abord : peu de voix différentes, un son
  plus égal. Sartus85 (Vendée) prononce ainsi 1 900 mots ; LoquaxFR et Poslovitch (Vosges), WikiLucas00 et Lyokoï
  (Lyon), VictorDtmtc (Nancy) et 20 autres le reste.
- Chaque fichier a été vérifié (durée, rapport signal/bruit), puis passé dans une reconnaissance vocale hors ligne
  (Vosk) limitée aux mots du jeu : quand elle entendait autre chose qu'un homophone, l'enregistrement suivant a été
  écouté de la même façon et gardé s'il était mieux reconnu (26 mots).

Les mots sans enregistrement, les noms avec leur article et les phrases sont lus par la voix française du système.

Reconstruire le paquet : `fuera-cli voices credits.tsv <dossier des WAV> voces-fr.fvox` (WAV mono, 16 bits, 16 kHz ;
scripts dans `tools/data/voces/`).

The pack holds French word recordings by Lingua Libre volunteers from France (then Switzerland and Belgium; never
Québec), each under its own licence (CC BY-SA 4.0, CC BY 4.0 or CC0). The pack as a whole is CC BY-SA 4.0. Speakers
and source files: `credits.tsv`.
