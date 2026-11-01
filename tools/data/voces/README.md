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
