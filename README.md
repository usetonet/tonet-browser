# Tonet

Tonet is a **from-scratch** desktop browser: its own networking, parsing, and rendering path—**not** Chromium, Firefox, WebKit, or CEF. The goal is a **small, fast, privacy-minded** surface for essential reading and navigation, with room to grow security and features over time.

- **Repository language:** English (issues, docs, comments).
- **Product UI:** English by default; the app follows **Settings → Language** (`auto` uses the system locale). The marketing site picks **English** as the HTML default and adapts to the visitor’s language when possible (same idea as mainstream browsers).

## Status

Early MVP: HTTP(S) fetch with strict limits, minimal HTML extraction, and a lightweight UI (toolbar, history, settings). See the [landing](https://usetonet.com) and [documentation](web/landing/docs.html) for downloads and packaging notes.

## Build (desktop)

Requires a recent **stable Rust** toolchain.

```bash
cargo build --release -p tonet
```

The binary is at `target/release/tonet` (or `tonet.exe` on Windows).

## Repository layout

| Path | Purpose |
|------|---------|
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

Contributions are welcome: open issues and pull requests on GitHub. Please keep user-facing copy and maintainer docs in **English** unless the change is explicitly locale-specific (e.g. translated landing strings).
