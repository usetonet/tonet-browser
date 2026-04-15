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

Tonet is licensed under the **PolyForm Noncommercial License 1.0.0**—see [`LICENSE`](LICENSE).

**In short:** you may use, study, modify, and share the project for **noncommercial** purposes. **Selling** the software or offering it as part of a paid product or service (other than as permitted in the license text) is **not** allowed for third parties. The **copyright holders** (Usetonet and Tonet contributors) retain the right to operate commercial offerings, services, and infrastructure around Tonet. For commercial licensing questions, contact the project maintainers via [usetonet.com](https://usetonet.com).

## Links

- Website: [usetonet.com](https://usetonet.com)
- Source: [github.com/usetonet/tonet-browser](https://github.com/usetonet/tonet-browser)

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines. Contributions are welcome: open issues and pull requests on GitHub. Please keep user-facing copy and maintainer docs in **English** unless the change is explicitly locale-specific (e.g. translated landing strings).
