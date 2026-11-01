# ADR 0002 — Pile technique : la base d'Alpagamp

- Statut : acceptée (31/10/2026)

## Décision

- **winit 0.30 + softbuffer** (fenêtre et image, partout), **tiny-skia** (dessin vectoriel lissé sur le processeur),
  **fontdue** (texte), **cpal** (son). Les briques sont dessinées une fois en sprites, le plateau en calque : le
  dessin n'a lieu que pendant les animations.
- **Android** : `NativeActivity`, aucune ligne de Java, APK construit sans Gradle (`cargo xtask apk`, repris
  d'Alpagamp) ; `targetSdk` 34 (pas d'affichage bord à bord imposé). Vibration et synthèse vocale par JNI.
- **Web** : la même appli compilée en WebAssembly (`cargo xtask web`), qui sert aussi de canal d'essai.
- **Sons** synthétisés par le code ; **voix** : celles du système (`say` sur Mac, System.Speech sur Windows,
  TextToSpeech sur Android, Web Speech dans le navigateur).
- Crates : `core` (règles, solveur, générateurs, sans dépendance), `learn` (espagnol), `ui` (écrans), `app`
  (plateformes), `tools/cli` (outils de niveaux), `xtask` (paquets).

## Conséquences

- APK d'environ 2 Mo, version web d'environ 2,4 Mo.
- Tout dessin passe par le processeur : pas d'effets plein écran coûteux.
