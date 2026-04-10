#!/usr/bin/env bash
# Genera tonet_<versión>_amd64.deb desde target/release/tonet (requiere dpkg-deb).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN="${ROOT}/target/release/tonet"
VERSION="$(grep -m1 '^version = ' "${ROOT}/crates/tonet/Cargo.toml" | cut -d'"' -f2)"
STAGE="${ROOT}/target/tonet-deb-stage"
OUT="${ROOT}/dist"

if [[ ! -f "${BIN}" ]]; then
  echo "Falta ${BIN}. Ejecuta: cargo build --release -p tonet" >&2
  exit 1
fi

rm -rf "${STAGE}"
mkdir -p "${STAGE}/DEBIAN" "${STAGE}/usr/bin" "${STAGE}/usr/share/applications"

cat > "${STAGE}/DEBIAN/control" << EOF
Package: tonet
Version: ${VERSION}
Section: web
Priority: optional
Architecture: amd64
Maintainer: Tonet Contributors
Description: Navegador web minimalista Tonet
Depends: libc6, libgtk-3-0, libglib2.0-0, libfontconfig1, libfreetype6, libxcb1, libx11-6
EOF

install -m 0755 "${BIN}" "${STAGE}/usr/bin/tonet"
install -m 0644 "${ROOT}/packaging/linux/tonet.desktop" "${STAGE}/usr/share/applications/tonet.desktop"

mkdir -p "${OUT}"
dpkg-deb --root-owner-group --build "${STAGE}" "${OUT}/tonet_${VERSION}_amd64.deb"
echo "Generado: ${OUT}/tonet_${VERSION}_amd64.deb"
