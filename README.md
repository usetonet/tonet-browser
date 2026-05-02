# Tonet

Tonet is a **from-scratch** desktop browser: its own networking, parsing, and rendering path—**not** Chromium, Firefox, WebKit, or CEF. The goal is a **small, fast, privacy-minded** surface for essential reading and navigation, with room to grow security and features over time.

- **Repository language:** English (issues, docs, comments).
- **Product UI:** English by default; the app follows **Settings → Language** (`auto` uses the system locale). The marketing site picks **English** as the HTML default and adapts to the visitor’s language when possible (same idea as mainstream browsers).

## Status

Early MVP: HTTP(S) fetch with strict limits, minimal HTML extraction, and a lightweight UI (toolbar, history, settings). See the [landing](https://usetonet.com) and [documentation](web/landing/docs.html) for downloads and packaging notes.

**Long-term direction, quality gates, and measurable phases:** [`TONET_VISION.md`](TONET_VISION.md) (fill in TBD baselines on the reference machine as you measure them).

## Build (desktop)

Requires a recent **stable Rust** toolchain.

```bash
cargo build --release -p tonet
```

Workspace tests (engine + browser + corpus smoke): `cargo test --workspace`. Avoid `cargo test --workspace --all-targets` on Windows: the installer binary (`tonet-setup`) is not meant as a test harness.

The binary is at `target/release/tonet` (or `tonet.exe` on Windows).

## GitHub Releases

When a commit lands on **`main`** and the version in [`crates/tonet/Cargo.toml`](crates/tonet/Cargo.toml) is **strictly greater** than on the merge’s **first parent** (`HEAD^1`), the workflow [`.github/workflows/release-on-version-bump.yml`](.github/workflows/release-on-version-bump.yml) builds Windows (MSI + EXE) and Linux (`.deb`), then publishes a **GitHub Release** tagged `vX.Y.Z` with those assets. Pushes that do **not** bump the crate version do **not** create a release. If tag `vX.Y.Z` already exists, the workflow skips publishing to avoid duplicates.

## Repository layout

| Path | Purpose |
|------|---------|
| [`TONET_VISION.md`](TONET_VISION.md) | Product vision, phases (incl. JS roadmap), quality gates, metrics placeholders. |
| `corpus/` | Frozen fixtures for corpus smoke tests and future conformance runs. |
| `crates/tonet-engine` | Engine contracts: limits, navigation policy, size checks (no UI). |
| `crates/tonet` | Desktop application (Rust, `eframe` / `egui`). |
| `web/landing` | Marketing site and docs (Vite + Cloudflare Workers). |
| `packaging/` | Debian and other packaging helpers. |
| `scripts/` | Install and utility scripts. |

## License

Tonet is **free software**: you may use, study, modify, and redistribute it under the terms of the [**GNU General Public License v3.0**](https://www.gnu.org/licenses/gpl-3.0.html) **or any later version** published by the Free Software Foundation—see [`LICENSE`](LICENSE) for the full license text.

When you distribute a modified version or a binary built from this source, the GPL requires you to preserve those freedoms for recipients (including making Corresponding Source available under the same license when you convey object code). Read the license for exact obligations.

## Links

- Website: [usetonet.com](https://usetonet.com)
- Source: [github.com/usetonet/tonet-browser](https://github.com/usetonet/tonet-browser)

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines. Contributions are welcome: open issues and pull requests on GitHub. Please keep user-facing copy and maintainer docs in **English** unless the change is explicitly locale-specific (e.g. translated landing strings).
