# CDN release and update configuration

This project distributes installers and update metadata from your own CDN (for example Cloudflare R2 + CDN) without depending on GitHub release APIs at runtime.

## Stable short filenames (always latest stable)

Production releases upload **extra objects** with fixed names so users never need version strings in the URL:

| R2 object | Meaning |
|-----------|---------|
| `Tonet-Setup.exe` | Inno installer — latest **stable** |
| `Tonet-x64.msi` | MSI — latest **stable** |
| `tonet_amd64.deb` | Debian package — latest **stable** |
| `Tonet-macos.tar.gz` | macOS tarball — latest **stable** |
| `Tonet-windows-x64-portable.zip` | Portable zip — latest **stable** |

Versioned filenames (e.g. `Tonet-Setup-0.2.1-x64.exe`) are always uploaded too for archival installs.

## Preview / development channel

When a release is published with **`TONET_RELEASE_PRODUCTION=false`** (repository variable), CI uploads **preview aliases** instead of stable ones:

| R2 object | Meaning |
|-----------|---------|
| `Tonet-Setup-Preview.exe` | Latest **non-production** (preview) Inno build |
| `Tonet-x64-preview.msi` | Latest preview MSI |
| `tonet_amd64-preview.deb` | Latest preview `.deb` |
| … | Same pattern for portable + mac tarball |

Set **`TONET_RELEASE_PRODUCTION`** to `true` (default) for normal stable releases. Set to `false` only for preview builds that must not move `Tonet-Setup.exe`.

## Variables for landing build (`web/landing`)

Create `web/landing/.env.production` (or `.env.local`) with:

```dotenv
TONET_CDN_BASE_URL=https://dl.usetonet.com
TONET_CDN_VERSION_PATH=/version.json
TONET_SITE_DOWNLOADS_PAGE=https://dl.usetonet.com/
```

Optional for local `npm run build`:

```dotenv
TONET_RELEASE_PRODUCTION=true
```

## Variables for Tonet app update checks (`crates/tonet`)

Set these at **build time** (CI or shell before `cargo build`):

```dotenv
TONET_UPDATE_MANIFEST_URL=https://dl.usetonet.com/version.json
TONET_DOWNLOADS_PAGE_URL=https://dl.usetonet.com/
```

The app compares semver against **`channels.stable.version`** when present, otherwise **`version`**.

## Manifest (`version.json`) schema (v2)

Published by the release workflow merge script (`scripts/cdn-merge-manifest.mjs`). Summary:

- **`version`** — latest stable version string (for update checks).
- **`channels.stable`** — recommended production channel; `download` uses **short filenames**.
- **`channels.development`** — present when at least one non-production release exists; `download` uses **`-Preview`** aliases.
- **`releases`** — full history for the landing “Specific version” picker; each entry has `production` and **versioned** `download` URLs.

Minimal shape:

```json
{
  "schemaVersion": 2,
  "version": "0.2.1",
  "channels": {
    "stable": {
      "version": "0.2.1",
      "production": true,
      "recommended": true,
      "download": {
        "windowsSetup": "https://dl.usetonet.com/Tonet-Setup.exe"
      }
    }
  },
  "releases": []
}
```

## GitHub repository variables / secrets

In **Settings → Secrets and variables → Actions**:

- **Secrets:** `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`
- **Variables:** `CLOUDFLARE_R2_BUCKET`, `TONET_CDN_BASE_URL`, `TONET_CDN_VERSION_PATH`, `TONET_SITE_DOWNLOADS_PAGE`, `TONET_UPDATE_MANIFEST_URL`, `TONET_DOWNLOADS_PAGE_URL`
- **Optional:** `TONET_RELEASE_PRODUCTION` — default `true`; set to `false` so a workflow run publishes **preview** aliases only (does not overwrite stable short names).

When `CLOUDFLARE_R2_BUCKET` is set, `publish-cdn` uploads versioned assets, alias objects, and merges `version.json` into R2.
