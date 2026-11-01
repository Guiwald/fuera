# Construire le paquet de voix / Building the voice pack

Scripts Python (bibliothèque standard + numpy, ffmpeg) qui ont servi à fabriquer `assets/es/voces/voces-es.fvox` :

1. `titles.py` : liste des enregistrements espagnols de Lingua Libre sur Commons (`Category:Lingua Libre pronunciation-spa`).
2. `select.py` : rapproche les titres des mots du jeu (`targets.txt` : colonne `es` des lexiques et des listes de
   grammaire), écarte les enregistrements « Non-native » et les licences non libres, puis choisit une voix par mot
   (Espagne d'abord, autre accent seulement pour les mots sans *z*, *ce/ci*, *ll*, *y* consonne).
3. `fetch.py` : télécharge chaque fichier choisi (une requête à la fois : Commons limite le débit), le ramène en mono
   16 kHz, coupe les silences et harmonise le volume.
4. `check.py` : écarte les fichiers trop courts, trop longs ou trop bruités.

Puis `fuera-cli voices assets/es/voces/creditos.tsv <dossier des WAV> assets/es/voces/voces-es.fvox`.

## 1.1 : français et espagnol d'Amérique latine

Les mêmes étapes, généralisées ; tout ce qui touche Commons tourne sur la machine de Guillaume (le nuage reçoit des
429), une requête à la fois :

1. `titles_chunk.py` (dans `~/Code/fuera/.voces-fr/`) : liste par morceaux de 160 s des catégories Commons
   `Lingua Libre pronunciation-fra` (439 592 fichiers) et `-spa`.
2. `ll_locations.py` : lieux des locuteurs, extraits du Wiktionnaire français (wiktextract, `raw-wiktextract-data.jsonl.gz`), où le
   robot de Lingua Libre range chaque enregistrement avec le lieu du locuteur → `ll_locations.json`.
3. `fuera-cli words fr|es-419` : les mots à prononcer ; `select_fr.py` / `select_419.py` : candidats par mot, rangés
   par origine du locuteur ; `cats.py` : catégories Commons (licence, « Non-native ») ; `plan.py` : plan de
   téléchargement.
4. `fetch_seq.py` : téléchargement et conversion, reprend où il s'est arrêté (`FUERA_PAUSE` règle la pause entre deux
   fichiers ; 0,5 s tient sans 429) ; `check_clips.py` : durée et bruit.
5. `manifest.py` : crédits (`credits.tsv`, `creditos-419.tsv` ; pour l'Amérique latine, reprise des voix d'Espagne
   neutres depuis les FLAC de `voces-es-sources.tar`), puis `fuera-cli voices`.
6. Contrôle par reconnaissance vocale (Vosk, modèle `small-fr`, grammaire limitée aux mots du jeu) : les mots mal
   reconnus (hors homophones) ont été remplacés par l'enregistrement suivant quand il était mieux reconnu.
