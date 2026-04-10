#!/usr/bin/env bash
# Instalación de usuario (sin root): copia el binario release y el .desktop en XDG.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${ROOT}/target/release/tonet"
DEST_BIN="${HOME}/.local/bin"
DEST_APP="${HOME}/.local/share/applications"
DESKTOP="${ROOT}/packaging/linux/tonet.desktop"

if [[ ! -f "${BIN}" ]]; then
  echo "Falta ${BIN}. Ejecuta desde la raíz del repo: cargo build --release" >&2
  exit 1
fi

mkdir -p "${DEST_BIN}" "${DEST_APP}"
install -m 0755 "${BIN}" "${DEST_BIN}/tonet"
install -m 0644 "${DESKTOP}" "${DEST_APP}/tonet.desktop"
echo "Tonet instalado en ${DEST_BIN}/tonet y ${DEST_APP}/tonet.desktop"
echo "Asegúrate de que ~/.local/bin está en tu PATH."
