# Changelog

## Unreleased

## 0.2.3

Alpha release: Chrome-style **TonetSetup** online installer, browser DevTools, and landing alpha messaging.

### Added
- **TonetSetup** (`tonet-setup`): downloads portable zip from GitHub with CDN fallback; per-user install; desktop and Start menu shortcuts; uninstall registry entry; blocks re-install when already on latest version; auto-updates older installs; frameless UI; no console window.
- **Tonet**: optional DevTools (Elements, Network), dock right/bottom, F12 toggle; shared `Servo` per process on Windows.
- Landing **Alpha** badge and copy; TonetSetup documented as primary Windows installer.
- `scripts/generate-app-ico.mjs` to regenerate `app.ico` from `tonet.svg`.

### Changed
- `crates/tonet` and `crates/tonet-setup` version **0.2.3** (`Cargo.lock` updated).
- Regenerated Windows icon (replaces cyan placeholder); `app.ico` included in portable zip CI artifacts.
- `web/landing/public/version.json`: `channels.development` → **0.2.3** (alpha / preview); stable channel unchanged at **0.2.1**.

## 0.2.2

Preview / development channel bump; stable CDN short names and manifest top-level `version` remain **0.2.1** until you promote a release with `TONET_RELEASE_PRODUCTION=true`.

### Added
- CDN manifest schema v2 with `channels.stable`, optional `channels.development`, and `releases` history for version picker.
- Stable short filenames on R2 (`Tonet-Setup.exe`, `tonet_amd64.deb`, …) overwritten each stable release; preview aliases (`Tonet-Setup-Preview.exe`, …) when `TONET_RELEASE_PRODUCTION=false`.
- Landing download section: channel selector (stable / preview / specific version) wired to manifest URLs.
- `scripts/cdn-merge-manifest.mjs` used by CI and local landing embed step.

### Changed
- `crates/tonet` and `crates/tonet-setup` version **0.2.2** (`Cargo.lock` updated).
- `web/landing/public/version.json`: `channels.development` points at **0.2.2** (`production: false`); `channels.stable` unchanged.
- Release `publish-cdn` job uploads alias objects, fetches previous `version.json`, merges history, and uploads the merged manifest.
- Landing embed / `gen:manifest` merge from committed `public/version.json` when present so release history is preserved; deploy workflow passes `TONET_RELEASE_PRODUCTION` from repository variables (set to `false` while publishing preview builds only).
- Project license is now **GNU GPL v3 or later** (`GPL-3.0-or-later`), replacing PolyForm Noncommercial; see `LICENSE`, `README.md`, and `CONTRIBUTING.md`.

## 0.2.1

### Added
- Landing redesign with richer visual sections and new public subpages: `compare`, `roadmap`, and `handbook`.
- Structured documentation expansion with operations, use cases, comparisons, and roadmap content.
- MkDocs-style docs source scaffold under `web/landing/docs-site`.
- CDN configuration documentation for release artifacts and update manifests (`docs/CDN_RELEASES.md`).

### Changed
- Landing download links now generate from CDN environment variables instead of hardcoded GitHub release paths.
- Tonet update checker now reads a manifest URL and opens a configurable downloads page URL.
- Update-related user-facing copy now references manifest/CDN flow rather than GitHub releases.
- Project version bumped from `0.2.0` to `0.2.1` in `tonet`, `tonet-setup`, and lock metadata.

### Removed
- Direct runtime dependency on GitHub Releases endpoint for update checks and download-page opening.
