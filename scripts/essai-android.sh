#!/bin/bash
# Installe l'APK d'essai (Fuera-X.Y.Z.apk, à côté de ce script) sur le téléphone branché en USB (débogage USB activé)
# ou sur l'émulateur ouvert, puis lance le jeu.
set -euo pipefail
ICI="$(cd "$(dirname "$0")" && pwd)"
ADB="${ADB:-$HOME/Library/Android/sdk/platform-tools/adb}"
command -v adb >/dev/null 2>&1 && ADB="$(command -v adb)"
APK="$(ls -t "$ICI"/Fuera-*.apk 2>/dev/null | head -1)"
[ -n "$APK" ] || { echo "Aucun Fuera-*.apk à côté de ce script."; exit 1; }
"$ADB" devices
"$ADB" install -r "$APK" || { echo "Si l'installation est refusée (signature différente), désinstalle d'abord : $ADB uninstall fr.guiwald.fuera"; exit 1; }
"$ADB" shell monkey -p fr.guiwald.fuera -c android.intent.category.LAUNCHER 1 >/dev/null
echo "¡Fuera! est lancé."
