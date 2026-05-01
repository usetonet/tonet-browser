# CDN release and update configuration

This project can distribute installers and update metadata from your own CDN (for example Cloudflare R2 + CDN) without depending on GitHub release endpoints at runtime.

## Variables for landing build (`web/landing`)

Create `web/landing/.env.production` (or `.env.local`) with:

```dotenv
TONET_CDN_BASE_URL=https://downloads.example.com
TONET_CDN_VERSION_PATH=/version.json
TONET_SITE_DOWNLOADS_PAGE=https://downloads.example.com/
```

## Variables for Tonet app update checks (`crates/tonet`)

Set these at build time (CI or local shell before `cargo build`):

```dotenv
TONET_UPDATE_MANIFEST_URL=https://downloads.example.com/version.json
TONET_DOWNLOADS_PAGE_URL=https://downloads.example.com/
```

## Expected manifest

`version.json` must include at minimum:

```json
{
  "version": "0.2.1"
}
```

Optional download fields (used by landing buttons):

- `download.windowsSetup`
- `download.windowsMsi`
- `download.windowsExe`
- `download.linuxSetup`
- `download.deb`
- `download.macSetup`
- `download.macTarball`

## GitHub repository variables/secrets

Set these in **Settings → Secrets and variables → Actions**:

- **Secrets**
  - `CLOUDFLARE_API_TOKEN`
  - `CLOUDFLARE_ACCOUNT_ID`
- **Variables**
  - `CLOUDFLARE_R2_BUCKET` (bucket name)
  - `TONET_CDN_BASE_URL`
  - `TONET_CDN_VERSION_PATH`
  - `TONET_SITE_DOWNLOADS_PAGE`
  - `TONET_UPDATE_MANIFEST_URL`
  - `TONET_DOWNLOADS_PAGE_URL`

When `CLOUDFLARE_R2_BUCKET` is set, release workflow uploads assets + `version.json` to R2 automatically.
