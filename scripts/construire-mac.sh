#!/bin/bash
# Construit ¡Fuera! pour ce Mac (Apple Silicon ou Intel) à partir de fuera.bundle, placé à côté de ce script.
# Usage : bash construire-mac.sh            → dist/Fuera-X.Y.Z.dmg (et Fuera.app)
#         bash construire-mac.sh --jouer    → lance le jeu directement après la compilation
set -euo pipefail
ICI="$(cd "$(dirname "$0")" && pwd)"
cd "$ICI"

echo "== 1/4 Outils d'Apple"
if ! xcode-select -p >/dev/null 2>&1; then
  xcode-select --install || true
  echo "Une fenêtre propose d'installer les outils de ligne de commande : accepte, attends la fin, puis relance ce script."
  exit 1
fi

echo "== 2/4 Rust"
if [ -f "$HOME/.cargo/env" ]; then . "$HOME/.cargo/env"; fi
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "$HOME/.cargo/env"
fi
rustup update stable --no-self-update >/dev/null 2>&1 || true

echo "== 3/4 Code source"
if [ -d fuera/.git ]; then
  git -C fuera fetch -q "$ICI/fuera.bundle" main
  git -C fuera checkout -q -f -B main FETCH_HEAD
else
  git clone -q -b main "$ICI/fuera.bundle" fuera
fi
git -C fuera fetch -q "$ICI/fuera.bundle" 'refs/tags/*:refs/tags/*' || true
cd fuera
VERSION="$(sed -n '/\[workspace.package\]/,/^\[/s/^version = "\(.*\)"/\1/p' Cargo.toml)"
echo "¡Fuera! $VERSION"

echo "== 4/4 Compilation (3 à 6 minutes la première fois)"
cargo build --release -p fuera --bin fuera
if [ "${1:-}" = "--jouer" ]; then
  exec ./target/release/fuera
fi
mkdir -p "$ICI/dist"
cargo run --release -q -p xtask -- bundle-mac target/release/fuera "$ICI/dist"
echo
echo "Terminé : $ICI/dist/Fuera-$VERSION.dmg"
echo "Ouvre le .dmg et glisse ¡Fuera! dans Applications."
open "$ICI/dist" || true
