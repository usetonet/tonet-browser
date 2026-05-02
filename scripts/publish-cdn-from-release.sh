#!/usr/bin/env bash
# One-off: republish R2 + version.json from an existing GitHub Release (same logic as
# .github/workflows/cdn-republish.yml). Requires Node 20+, gh, wrangler, and .env with
# CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID, CLOUDFLARE_R2_BUCKET, TONET_CDN_BASE_URL.
#
# Usage: ./scripts/publish-cdn-from-release.sh <semver> [production]
# Example: ./scripts/publish-cdn-from-release.sh 0.2.1 true
set -euo pipefail

VERSION="${1:?usage: $0 <semver> [production:true|false]}"
PRODUCTION="${2:-true}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

: "${CLOUDFLARE_API_TOKEN:?missing CLOUDFLARE_API_TOKEN}"
: "${CLOUDFLARE_ACCOUNT_ID:?missing CLOUDFLARE_ACCOUNT_ID}"
: "${CLOUDFLARE_R2_BUCKET:?missing CLOUDFLARE_R2_BUCKET}"
: "${TONET_CDN_BASE_URL:?missing TONET_CDN_BASE_URL}"

R2_BUCKET="$CLOUDFLARE_R2_BUCKET"
CDN_BASE="${TONET_CDN_BASE_URL%/}"
export VERSION
export TONET_RELEASE_PRODUCTION="$PRODUCTION"

mkdir -p assets
gh release download "v${VERSION}" --dir assets --clobber

if ! command -v wrangler >/dev/null 2>&1; then
  npm i -g wrangler@3
fi

for f in assets/*; do
  name="$(basename "$f")"
  wrangler r2 object put "${R2_BUCKET}/${name}" --file "$f"
done

V="${VERSION}"
if [ "${TONET_RELEASE_PRODUCTION}" = "false" ]; then
  wrangler r2 object put "${R2_BUCKET}/Tonet-Setup-Preview.exe" --file "assets/Tonet-Setup-${V}-x64.exe"
  wrangler r2 object put "${R2_BUCKET}/Tonet-x64-preview.msi" --file "assets/Tonet-${V}-x64.msi"
  wrangler r2 object put "${R2_BUCKET}/tonet_amd64-preview.deb" --file "assets/tonet_${V}_amd64.deb"
  wrangler r2 object put "${R2_BUCKET}/Tonet-macos-preview.tar.gz" --file "assets/Tonet-${V}-macos.tar.gz"
  wrangler r2 object put "${R2_BUCKET}/Tonet-windows-x64-portable-preview.zip" --file "assets/Tonet-${V}-windows-x64-portable.zip"
else
  wrangler r2 object put "${R2_BUCKET}/Tonet-Setup.exe" --file "assets/Tonet-Setup-${V}-x64.exe"
  wrangler r2 object put "${R2_BUCKET}/Tonet-x64.msi" --file "assets/Tonet-${V}-x64.msi"
  wrangler r2 object put "${R2_BUCKET}/tonet_amd64.deb" --file "assets/tonet_${V}_amd64.deb"
  wrangler r2 object put "${R2_BUCKET}/Tonet-macos.tar.gz" --file "assets/Tonet-${V}-macos.tar.gz"
  wrangler r2 object put "${R2_BUCKET}/Tonet-windows-x64-portable.zip" --file "assets/Tonet-${V}-windows-x64-portable.zip"
fi

export CDN_BASE
PREV_PATH=prev.json
curl -sfL "${CDN_BASE}/version.json" -o prev.json || echo '{}' > prev.json
node scripts/cdn-merge-manifest.mjs > version.json
wrangler r2 object put "${R2_BUCKET}/version.json" --file version.json

echo "CDN republish complete for v${VERSION}."
