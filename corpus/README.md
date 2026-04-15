# Tonet corpus (frozen fixtures)

Versioned HTML/CSS (and later JS) snippets and **frozen** page snapshots used to measure compatibility and regressions. See [`TONET_VISION.md`](../TONET_VISION.md) for gates and targets.

| Path | Role |
|------|------|
| `fixtures/minimal.html` | Tiny valid document; loaded by `tonet-engine` corpus smoke tests in CI. |
| `fixtures/with_links.html` | Same, plus `<a href>` pairs for future link-resolution tests. |

Add new fixtures with a short note here when they are introduced.
