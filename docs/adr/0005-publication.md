# ADR 0005 — Dépôt, publication, hébergement

- Statut : acceptée (31/10/2026)

## Décision

- Dépôt public **Guiwald/fuera** (MIT), créé et alimenté par `scripts/publier-sur-github.sh` depuis le Mac.
- **Publication** (`.github/workflows/release.yml`) à chaque étiquette `vX.Y.Z` : `.dmg` universel, `.zip` Windows,
  `.apk` signé (clé RSA 4096 propre au projet, créée par le script, copie à garder), `.tar.gz` Linux, version web.
- **Web** : `fuera.librejeu.fr` (hébergement OVH Pro de librejeu.fr, dossier `fuera`, ajouté le 31/10/2026), mis en
  ligne par SFTP avec l'utilisateur FTP `librejeu-fue` limité à ce dossier ; mot de passe uniquement dans les secrets
  GitHub.
- Essais : la version web publiée en page privée claude.ai ; Android via l'émulateur Pixel 8 et le pont adb d'AlpIRC
  ou `scripts/essai-android.sh`.
