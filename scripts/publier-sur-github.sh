#!/bin/bash
# Publie ¡Fuera! sur GitHub : crée le dépôt public Guiwald/fuera (la première fois), envoie le code et les étiquettes,
# crée la clé de signature Android du projet (une seule fois, gardée dans ./signature-android), enregistre les secrets
# (clé Android, mot de passe SFTP d'OVH pour la version web), puis lance la publication de la version.
#
# Usage, depuis le dossier qui contient fuera.bundle :   bash publier-sur-github.sh
# La connexion à GitHub se fait dans le navigateur, avec ton compte (aucun jeton à copier).
set -euo pipefail
trap 'echo "Arrêt : la commande de la ligne $LINENO a échoué. Relance le script après correction (il reprend où il en est)." >&2' ERR
ICI="$(cd "$(dirname "$0")" && pwd)"
cd "$ICI"
DEPOT="fuera"
OVH_HOTE="ftp.cluster103.hosting.ovh.net"
OVH_UTILISATEUR="librejeu-fue"

echo "== 1/6 Outil GitHub (gh)"
if ! command -v gh >/dev/null 2>&1; then
  if command -v brew >/dev/null 2>&1; then brew install gh; else
    echo "Installe gh depuis https://cli.github.com puis relance ce script."; exit 1
  fi
fi
echo "gh : $(gh --version | head -1)"

echo "== 2/6 Connexion à GitHub"
if ! gh auth status -h github.com >/dev/null 2>&1; then
  echo "gh va afficher un code de 8 caractères puis ouvrir GitHub dans ton navigateur :"
  echo "appuie sur Entrée, colle le code dans la page, puis clique sur « Authorize »."
  gh auth login -h github.com -p https -w -s workflow
elif ! gh auth status -h github.com 2>&1 | grep -q "workflow"; then
  gh auth refresh -h github.com -s workflow
fi
gh auth setup-git -h github.com
MOI="$(gh api user -q .login)"
echo "Connecté en tant que $MOI"

echo "== 3/6 Code source"
if [ -d fuera/.git ]; then
  git -C fuera fetch -q "$ICI/fuera.bundle" main
  git -C fuera checkout -q -f -B main FETCH_HEAD
else
  git clone -q -b main "$ICI/fuera.bundle" fuera
fi
git -C fuera fetch -q -f "$ICI/fuera.bundle" '+refs/tags/*:refs/tags/*' || true
VERSION="$(sed -n '/\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' fuera/Cargo.toml)"
TAG="v$VERSION"
echo "¡Fuera! $VERSION"

echo "== 4/6 Dépôt https://github.com/$MOI/$DEPOT"
if ! gh repo view "$MOI/$DEPOT" >/dev/null 2>&1; then
  gh repo create "$MOI/$DEPOT" --public \
    --description "¡Fuera! — jeu de blocs à faire sortir par la bonne porte, et pour apprendre l'espagnol. Rust : Mac, Windows, Android, web." \
    --homepage "https://fuera.librejeu.fr"
fi

echo "== 5/6 Secrets (clé Android, OVH)"
SIG="$ICI/signature-android"
if [ ! -f "$SIG/fuera.p12" ]; then
  mkdir -p "$SIG"
  # openssl est livré avec macOS (pas besoin de Java) ; pas de tube ici : avec « pipefail », un tube coupé
  # arrêtait le script sans un mot.
  MDP="$(openssl rand -hex 16)"
  TMP="$(mktemp -d)"
  openssl req -x509 -newkey rsa:4096 -nodes -days 36500 -subj "/CN=Guiwald/O=fuera" \
    -keyout "$TMP/cle.pem" -out "$TMP/cert.pem" 2>/dev/null
  openssl pkcs12 -export -name fuera -inkey "$TMP/cle.pem" -in "$TMP/cert.pem" \
    -out "$SIG/fuera.p12" -passout "pass:$MDP"
  rm -rf "$TMP"
  printf %s "$MDP" > "$SIG/mot-de-passe.txt"
  unset MDP
  chmod 600 "$SIG"/*
  echo "Clé de signature Android créée dans $SIG (garde une copie de ce dossier : sans elle, une mise à jour"
  echo "de l'appli impose de la désinstaller)."
fi
base64 < "$SIG/fuera.p12" | gh secret set FUERA_KEYSTORE_B64 -R "$MOI/$DEPOT"
gh secret set FUERA_KEYSTORE_PASS -R "$MOI/$DEPOT" < "$SIG/mot-de-passe.txt"
if ! gh secret list -R "$MOI/$DEPOT" | grep -q OVH_SFTP_PASSWORD; then
  echo "Mot de passe de l'utilisateur FTP « $OVH_UTILISATEUR » d'OVH (librejeu.fr → FTP-SSH) ; laisse vide pour plus tard :"
  read -r -s OVH_MDP
  echo
  if [ -n "$OVH_MDP" ]; then
    printf %s "$OVH_HOTE" | gh secret set OVH_SFTP_HOST -R "$MOI/$DEPOT"
    printf %s "$OVH_UTILISATEUR" | gh secret set OVH_SFTP_USER -R "$MOI/$DEPOT"
    printf %s "$OVH_MDP" | gh secret set OVH_SFTP_PASSWORD -R "$MOI/$DEPOT"
    unset OVH_MDP
    echo "Secrets OVH enregistrés : la version web sera mise en ligne sur https://fuera.librejeu.fr à chaque version."
  fi
fi

echo "== 6/6 Envoi du code et publication de $TAG"
cd fuera
git remote remove origin >/dev/null 2>&1 || true
git remote add origin "https://github.com/$MOI/$DEPOT.git"
git push -u origin main
if ! git rev-parse "$TAG" >/dev/null 2>&1; then git tag -a "$TAG" -m "¡Fuera! $VERSION"; fi
git push origin --tags
cd "$ICI"
echo
echo "Terminé ! GitHub fabrique maintenant les fichiers de la version (environ 20 minutes) :"
echo "  La compilation :      https://github.com/$MOI/$DEPOT/actions"
echo "  Les téléchargements : https://github.com/$MOI/$DEPOT/releases/latest"
echo "  La version web :      https://fuera.librejeu.fr"
open "https://github.com/$MOI/$DEPOT/actions" >/dev/null 2>&1 || true
