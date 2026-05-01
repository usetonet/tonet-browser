# Changelog

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
