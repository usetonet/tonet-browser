# Servo engine integration and Tonet shell quality checklist

Living backlog for **Servo ↔ Tonet** and **post-Servo browser polish**. Status: `[x]` done, `[~]` partial / best-effort, `[ ]` not started. See also `TONET_VISION.md` §13.

**Reality check:** items like Linux embed, IME, multi-`WebView`, and full screen-reader coverage are **multi-week** engineering tracks. This file tracks them explicitly so “done” means **verified in tree**, not “checkbox clicked without implementation.”

### Servo crate pin and upgrade playbook

- **Pin:** `crates/tonet/Cargo.toml` keeps `servo = "0.1"` on the **0.1.x** line; optional deps (`http`, `rfd`, …) are listed next to the `servo-engine` feature.
- **Before bumping** the `servo` version: read release notes (mozjs, surfman, embedder API), run `cargo check -p tonet` on Windows with `LIBCLANG_PATH` and EGL DLLs, run `cargo test -p tonet`, and re-run **§ Manual smoke** below.
- **CI:** `.github/workflows/servo-engine-windows.yml` should stay aligned with the LLVM/bindgen expectations documented in **§ Build prerequisites**.

### Current policy: `http(s)` engine (Windows)

- **Default Windows build:** `http://` / `https://` and internal `tonet://` pages use **Servo only** for page content when the viewport is active (default **in-process** surfman readback → egui; optional legacy **owned Win32 popup** with **`TONET_SERVO_WIN32_POPUP=1`**; no `tonet-engine` DOM paint on those URLs). Opt-out for debugging: **`TONET_SERVO_VIEWPORT=0`**.
- **Non-Windows** (or without `servo-engine`): `http(s)` stays on the Tonet engine unless a future native embed exists; optional opt-in remains **Settings → System** (non-Windows) and/or **`TONET_SERVO_VIEWPORT=1`** where implemented.

---

## A. Servo ↔ Tonet integration

### A1. Build, CI, policy

| Status | Task |
|--------|------|
| [x] | Document build prerequisites (Windows, `servo-engine`, LLVM, EGL DLLs) in this file and vision §13. |
| [x] | CI: `.github/workflows/servo-engine-windows.yml` runs `cargo check -p tonet` on `windows-latest` with LLVM 18 (`continue-on-error: true` so default CI is not blocked). |
| [x] | Pin / document supported `servo` crate versions and upgrade playbook (`Cargo.toml` comment + **§ Servo crate pin and upgrade playbook** above). |
| [x] | Product decision: **Servo-by-default for `http(s)` on Windows** when built with `servo-engine` — documented above; opt-out via `TONET_SERVO_VIEWPORT=0`. |
| [x] | **`css/`**, **`html/`**, **`document_url.rs`**, **`limits.rs`**, and **`policy.rs`** live under `crates/tonet/src/` (canonical). `crates/tonet` does **not** depend on the `tonet-engine` package. **`tonet-engine`** keeps small-only modules (`js`, `navigation`) and re-exports shared sources from `tonet` via `#[path]` so corpus/unit tests can use `tonet_engine::` without duplicating files. |

### A2. Platforms

| Status | Task |
|--------|------|
| [x] | Windows: Win32 popup + surfman + `WindowRenderingContext` + shell snapshot sync. |
| [~] | Linux: native surface + event loop + input — **`ServoViewportRuntime::tick` is a no-op** today; links + API surface documented in **§ Linux / macOS Servo embed**. |
| [~] | macOS: embed parity — same no-op + portability plan as Linux until a native surface lands. |
| [x] | Feature matrix (OS × viewport × input) — see **§ “Platform feature matrix”** below. |

### A3. Tabs and lifecycle

| Status | Task |
|--------|------|
| [x] | Document single shared `WebView` and limits (tab switch, BF state) — see **§ “Single WebView model”** below. |
| [~] | One `WebView` per tab **or** explicit recycle + session restore rules — **§ A3 design backlog** (options table; no implementation chosen). |
| [~] | Suspend / throttle Servo embedder when the **active** tab is not `http(s)` (experimental viewport still on): `about:blank` instead of a remote demo page + **1/4** `spin_event_loop` cadence in `ServoWinHost::tick`. True per-tab CPU/GPU suspend for **other** `http(s)` tabs still depends on multi-`WebView` / A3 backlog. |

### A4. Shell sync

| Status | Task |
|--------|------|
| [x] | URL, title, loading, back / forward / reload wired from `WebView` into `Tab`. |
| [x] | `BrowserLog::record_visit` for Servo navigations (`http`/`https` only, deduped across frames). |
| [x] | Favicon: `WebView::favicon` → PNG → `egui` `include_bytes` + `favicon_uri` (when `servo-engine` + Windows). |
| [~] | Page errors: `LoadStatus` has no failure variant upstream; map embedder errors when API exists. |
| [x] | **HTTP auth:** `WebViewDelegate::request_authentication` → egui modal (username + masked password, **Sign in** / cancel / close); `for_proxy` note in i18n; ordered after script dialog, before site permission; `Drop` drops pending request (Servo treats as cancel). |
| [~] | Stop loading: no public `WebView::stop`; toolbar **Stop** is **disabled** while loading on Servo-superseded tabs (tooltip explains embedder limitation); Tonet-engine tabs still cancel in-flight fetch. Upstream API still wanted for real cancel. |

### A5. Input and focus

| Status | Task |
|--------|------|
| [x] | Pointer + wheel via popup `WNDPROC` → `notify_input_event`. |
| [x] | Keyboard forward path + release on omnibox / outside / Escape; skip `modifiers.command` shortcuts. |
| [~] | IME / composition: egui `Event::Ime` → Servo `InputEvent::Ime` while **page captures keyboard** (CJK/IME on Windows needs validation on real hardware). |
| [x] | Cursor: `WebView::cursor` each tick + `WM_SETCURSOR` on the Win32 popup (`LoadCursorW` / `SetCursor`, IDC mapping). |
| [~] | AccessKit / OS a11y bridge for embedded content — **blocked on upstream**; track API availability; shell-side keyboard audit in **§ Accessibility audit templates**. |

### A6. Chrome-specific UX

| Status | Task |
|--------|------|
| [~] | Right gutter scrollbar strip (decorative); real scroll metrics from Servo optional. |
| [~] | Context menu: `EmbedderControl::ContextMenu` → egui window + `select` / `dismiss`; `hide_embedder_control` wired. **Tonet:** “Open link in new Tonet tab” for `http(s)` hit-test links (`element_info().link_url`); Servo’s own `OpenLinkInNewWebView` entries unchanged. |
| [x] | Script dialogs (`alert` / `confirm` / `prompt`): `EmbedderControl::SimpleDialog` → egui window (`runtime_win` + `ServoViewportRuntime::show_embedder_modals`). |
| [x] | **Site permissions:** `request_permission` → disk-backed map (`servo_permissions.json` under Tonet config) + egui modal; origin+feature keys; localized names in `i18n`. **Settings → System** (internal tab) and the **floating preferences window** (Windows + `servo-engine`) offer “Clear saved Servo site permissions” without wiping visit history; clearing visit history also removes the file. |
| [~] | **Web notifications:** `WebViewDelegate::show_notification` → egui toast (top chrome, TTL + dismiss); not the OS notification center; cleared on `teardown_pending_embedder_controls`. |
| [~] | **Page console:** `WebViewDelegate::show_console_message` → bounded queue on the host, drained into `Tab::servo_console` each `sync_into_tab`; bottom strip (monospace scroll + Clear) when the Servo viewport is active; lines cleared when leaving the Servo URL gate / viewport. |
| [~] | **Downloads:** `WebViewDelegate::load_web_resource` — **best-effort** main-frame `GET` interception for a fixed extension list (`.zip`, `.pdf`, …): respond `204`, `reqwest` + `rfd` save-as on a worker thread, then `BrowserLog::record_page_fetch` with `saved_path`. **No** Servo cookie jar / auth on that fetch. **Narrow `HEAD` probe:** last path segment `download` / `export` / `attachment` with **no** dotted URL extension → `HEAD` (4s) inspects `Content-Disposition` filename + a small `Content-Type` allowlist; still no generic “any URL + CD” without URL hint. Navigations + `record_page_fetch` without interception unchanged. Internal `tonet://downloads` now also supports a Servo-side **Clear all** action link. `<select>` / `<input type=color>` / `<input type=file>` / IME as documented. |
| [~] | **Internal `tonet://` via Servo:** HTML synthesis for `settings`, `history`, `downloads` is live in `servo_engine/tonet_scheme_html.rs`; action links queue shell operations (**clear history**, **clear downloads**, **clear Servo site permissions**). Remaining parity with egui internal pages is tracked in B1/B3. |

### A7. Quality

| Status | Task |
|--------|------|
| [~] | Performance budgets (TTFP, RSS) with Servo on reference machine — **template** in **§ Performance budget template** below; fill numbers when measured on the reference machine (`TONET_VISION.md` §4 / §9). |
| [x] | `cancel_in_flight` no longer wipes Servo-derived tab fields (avoids broken chrome after cancel paths). |
| [~] | Clean shutdown / toggle experimental mode: `ServoWinHost::Drop` calls **`teardown_pending_embedder_controls`** (dialogs, HTTP auth, permission, menus, pickers, in-flight file pick, web-notification toast, in-memory console queue, queued download-log completions) then **`spin_event_loop`**; further hardening TBD. |
| [~] | Corpus comparison Tonet engine vs Servo — **procedure** in **§ Corpus comparison** below; fill a spreadsheet or issue table when you run paired passes on the reference machine. |

### A8. Tests

| Status | Task |
|--------|------|
| [x] | Unit tests for visit / URL policy (`servo_engine::visit_policy`, no Servo runtime). |
| [~] | More pure-logic tests: **`visit_policy`** URL gate + trim + scheme casing + `javascript`/`data`/`blob`/`file`/`ws`/`wss`/`chrome` + incomplete/empty committed URL (`is_http_or_https_history_url`, `should_record_visit`); **`servo_supersedes_dom_paint`** (Windows + `servo-engine`: trim, lowercase `http`/`https` only); **`download_heuristic`** (schemes incl. `ws`/`wss`/`ftp`/`file`, `GET` only vs `POST`/`HEAD`/`OPTIONS`/`PUT`/`DELETE`/`PATCH`, IPv6 `https`, redirects, multi-dot paths, trailing `/`, `*.tar.gz` → final `gz` suffix); **`url_path`** (`last_non_empty_path_segment`, single segment, IPv6 host); **`background_download`** (`sanitize_filename` incl. `char::is_control()`, Win32 trailing `.`/` ` strip, reserved DOS device stem `_*`, `suggested_save_name_for_download` incl. IPv6 URL path); **`content_disposition`** (`parse_filename_value`, leading UTF-8 BOM trim, `;` split + closing quote with `\"` rule, quoted `\\`/`\"` unescape then `%HH`, `notfilename=` guard, unquoted / `filename*`, empty `filename*` fallback, `inline`); favicon PNG path still relies on `servo::Image` (no extra unit tests yet). |
| [~] | Manual smoke checklist — §“Manual smoke” below expanded for recent embedder paths (incl. step **17b** extensionless `HEAD` download); still add login / media / PDF spot-checks as you find stable test URLs. |

---

## B. Full browser polish (Tonet shell, all engines)

### B1. Chrome and utilities

| Status | Task |
|--------|------|
| [~] | Tabs, omnibox, nav buttons, new tab, session restore — iterate on bugs as found. |
| [~] | Omnibox visit-history autocomplete (focus omnibox → scrollable `http(s)` rows from `visit_history.json`, substring match + recent when empty; **click** or **↑/↓ + Enter** navigates; **Escape** clears keyboard row highlight; selection resets when the suggestion list changes). Separate **product chips** / richer autocomplete remains spec-driven. |
| [~] | “Clear browsing data”: clearing **Downloads** (internal page) also clears Servo **ephemeral** queues (console / notification toast / pending download-log rows) and **`servo_console` on all tabs** when `servo-engine` + Windows; persisted `servo_permissions.json` still only cleared with **Clear history** or explicit **Clear saved Servo site permissions**. |

### B2. Accessibility

| Status | Task |
|--------|------|
| [~] | Stable `egui::Id` sources: **`chrome/ids.rs`** (toolbar + tabs + new tab); **`servo_engine/embedder_ids.rs`** (Servo embedder windows/toast on Windows + regression tests); **`ui.rs`** (`omnibox_id`, `settings_modal_id`, `settings_internal_form_id`). Screen-reader labels / full keyboard order still TBD. |
| [~] | Full keyboard order and screen-reader labels per control — **procedure** in **§ Accessibility audit templates**; egui limits still TBD per control. |
| [~] | Contrast / scaling audit vs system settings — same **§ Accessibility audit templates** (spot-check matrix). |

### B3. i18n

| Status | Task |
|--------|------|
| [~] | Servo-related strings in `i18n` for supported locales. |
| [~] | Review all new user-facing strings on every locale before “stable” Servo — **§ i18n coverage matrix** (track “verified” per locale × area). |

---

## Platform feature matrix

| OS | `servo-engine` build | Native Servo viewport | Pointer / wheel | Keyboard / IME forward | Cursor from engine |
|----|----------------------|-------------------------|-----------------|-------------------------|----------------------|
| **Windows** | Yes | Yes (default in-process embed; optional `TONET_SERVO_WIN32_POPUP=1` owned popup) | Yes (`WNDPROC` on popup **or** egui-fed embed per mode) | Yes (capture mode + IME mapping) | Yes (`WM_SETCURSOR` when popup) |
| **Linux** | Yes (links) | No | N/A | N/A | N/A |
| **macOS** | Yes (links) | No | N/A | N/A | N/A |

## Single WebView model

Tonet keeps **one** `WebView` inside `ServoWinHost` today. Implications:

- **Active tab only:** the popup shows the **active** tab’s `http(s)` URL; switching tabs calls `WebView::load` with the new URL (history in Servo is per document, not mirrored to Tonet’s `Tab::history` for that path).
- **BF chrome:** back/forward buttons read Servo’s `can_go_back` / `can_go_forward` for that single view, not independent per-tab stacks.
- **Background tabs:** other tabs’ Servo-specific fields (`servo_document_title`, etc.) can be **stale** until you switch back and sync runs again.
- **Idle embedder (experimental on, non-`http(s)` active tab):** the hidden `WebView` is driven to `about:blank` and Servo’s event loop is **throttled** (see A3 row) so settings / new-tab / internal pages do not keep a heavy remote page hot.

Multi-WebView-per-tab remains a follow-up (see A3 table).

### Linux / macOS Servo embed (A2 status, `servo-engine`)

- **Linux / macOS with `--features servo-engine`:** Tonet **links** the `servo` crate (`link_servo_when_enabled` in `Cargo.toml`). **`ServoViewportRuntime::tick`** is currently a **no-op** on non-Windows builds: there is **no** native `WindowRenderingContext` surface, popup, or input subclass yet.
- **Goal:** keep the **same** `ServoViewportRuntime` entry points as Windows (`tick`, `sync_active_tab_from_servo`, …) so the shell stays cross-platform; add Linux embed first unless platform constraints dictate otherwise.
- **macOS:** schedule **after** Linux for parity unless product priorities change.

### A3 design backlog (multi-WebView, suspend)

| Option | Pros | Cons |
|--------|------|------|
| **One `WebView` per tab** | Independent back/forward and cache per tab | Memory / GPU; lifecycle wiring |
| **Recycle a single `WebView`** | Lower RAM | Complex rules when switching tabs; session restore |
| **Suspend background tabs** | Saves CPU with a single view | Needs a clear “paused” state + Servo API support |

No code path is chosen yet; behavior today is the **single-WebView** model above.

### Accessibility audit templates (B2)

- **Keyboard / focus:** tab through chrome (tabs, omnibox, nav buttons, settings entry points) using **only** the keyboard; note tab-order skips, missing focus rings, and traps (document egui defaults and gaps).
- **Contrast / scaling:** compare Tonet chrome (`theme` + `UiTheme`) against the host OS accessibility recommendations (contrast, text size); file issues for controls that fail spot checks.

### i18n coverage matrix (B3)

| Area | Locales (Es, De, Fr, En) | Notes |
|------|--------------------------|-------|
| Servo embedder modals / toasts (`servo_*`, related keys) | *TBD per release* | Open each path once per locale |
| Internal pages + chrome | *TBD per release* | grep / manual pass |

Mark the B3 row **fully verified** when every cell has a dated “OK” in your QA tracker (issue or spreadsheet).

---

## Build prerequisites (Servo on Windows)

- Toolchain: **MSVC**, Windows SDK.
- **LLVM** / `libclang` for bindgen (e.g. `LIBCLANG_PATH`).
- **EGL**: `mozangle` DLLs next to `tonet.exe` (see crate `Cargo.toml` / build scripts and `scripts/tonet-servo-windows.ps1` if present).
- Cargo: `cargo build -p tonet`.

---

## Manual smoke (Servo viewport)

1. Build with default features on Windows (`cargo build -p tonet` or `.\scripts\tonet-servo-windows.ps1 run -p tonet`); optional **`TONET_SERVO_VIEWPORT=0`** to verify fallback.
2. Open `https://example.com` — check paint, URL bar, title, back, reload, favicon in tab strip.
3. Click page → type in a form; Escape returns focus to chrome.
4. Resize window and change display scale — popup tracks central rect.
5. Switch tabs — note single-`WebView` limitations until A3 is done.
6. With an **East Asian IME**, focus the page (click content), type in a form: preedit/commit should reach Servo (verify on a real `http(s)` page).
7. Hover links and text fields on the Servo surface: cursor should switch (e.g. hand, I-beam) per page CSS.
8. Open a page that calls `alert` / `confirm` / `prompt` — modal should appear over Tonet chrome; OK/Cancel (and prompt text) should resume script as expected.
9. Open an **`http(s)` URL protected with HTTP Basic auth** (or trigger **proxy** auth) — “Website login” modal with username/password; **Sign in** should continue the load; Cancel, Escape, or closing the window should cancel auth (page may show an error per Servo).
10. Right-click on page content — “Page menu” should appear **near the click** (mapped from the Servo popup into the egui window); choose an action or Escape / close; page should continue normally. On an **`http(s)` link**, **“Open link in new Tonet tab”** should add a shell tab and navigate there (remember **single `WebView`**: Servo stays on the old tab until you switch).
11. Open a page with a `<select>` / dropdown — “Choose option” should list entries; confirm updates the page; Cancel or closing the window keeps prior behavior.
12. Open a page with `<input type=color>` — “Color” window + sRGB picker; OK applies; Cancel keeps previous color.
13. Open a page with `<input type=file>` — native OS picker; choosing file(s) or cancel should return to the page without freezing Tonet.
14. Open a page that requests a **DOM permission** (e.g. geolocation) — “Site permission” modal with **Allow** / **Deny**; Escape or closing the window should deny; Allow should let the feature proceed per Servo. Trigger the same permission again (same session or after restart) — **no second modal** once a choice is stored in `servo_permissions.json` under the Tonet config folder.
15. **Web notifications:** on a page that calls the **Notification API** (after user gesture + permission if required), a **toast** should appear under the top chrome (localized title/body, dismiss, TTL); it should not rely on the OS notification center.
16. **Page console:** open a page that uses `console.log` / `console.warn` / `console.error` — lines should appear in the **bottom “Page console (Servo)” strip** when Servo owns the tab; **Clear** empties the strip; leaving the `http(s)` Servo path clears it (see `sync_active_tab_from_servo`).
17. **Heuristic download:** navigate (same tab, main frame) to a **public** `https://…/file.zip` (or another URL whose path ends in an allowlisted extension per `download_heuristic.rs`) — Tonet should offer **Save as**, write the file, and append an entry in **internal Downloads** with `saved_path`. Cancel the dialog: no new row (or no path). Remember: this path does **not** use Servo’s cookie jar (cookie-auth downloads may fail).
17b. **Heuristic download (extensionless path + `HEAD`):** same tab, main frame, to a public `https` URL whose **last path segment** is **`download`**, **`export`**, or **`attachment`** with **no** dotted extension (e.g. `https://…/api/download` or `…/EXPORT`), where the server’s **`HEAD`** response includes **`Content-Disposition`** with an allowlisted filename (e.g. `…filename="report.pdf"`) or a matching **`Content-Type`** (see `background_download::head_suggests_intercept_binary_get`) — Tonet should still intercept, **Save as**, and log **Downloads** when you save (same cookie/auth caveats as step 17). If `HEAD` is not supported (405) or headers do not indicate a binary, navigation should proceed normally.
18. **Settings → System** (internal): **Clear saved Servo site permissions** — should remove `servo_permissions.json` and reset in-memory cache without clearing visit history; a permission prompt should reappear after clear.
19. **Toolbar stop** on a Servo-superseded tab while loading: **Stop** is **disabled** with a tooltip (no upstream `WebView::stop` yet; see A4); reload and navigation should still work.
20. **Login / media / PDF** (spot-check when you have URLs): OAuth or form login over `https`, `<video>` / `<audio>` playback, and in-page or navigated **PDF** — note success/failures for corpus work (A7) and file bugs.

### Performance budget template (Servo, Windows + experimental viewport)

Fill after measuring on the **reference machine** from `TONET_VISION.md` §9 (same build flags and viewport size each run).

| Metric | Procedure (sketch) | Baseline | Target / note |
|--------|--------------------|----------|-----------------|
| Cold start → first paint (`example.com`) | Time from process start to stable `LoadStatus::Complete` + visible frame | *TBD* | Track regression only until product sets a budget |
| Resident set after 5 min browsing | OS RSS for `tonet.exe` | *TBD* | Compare Tonet-engine vs Servo-superseded same session |
| Navigation (cached) | Same-origin click latency feel / optional tracing | *TBD* | |

### Corpus comparison (Tonet engine vs Servo viewport)

Use the **same** URL list, **same** machine, and record **build flags** (`servo-engine` on/off, experimental setting, env), **window size**, and **viewport** (egui central rect).

| Column | Notes |
|--------|--------|
| URL | `https://…` |
| Tonet engine | Default path: DOM read view result (pass / fail / notes) |
| Servo experimental | Same URL with Servo superseding paint (pass / fail / notes) |
| Delta | Screenshot or short description of divergence |

Store results where the project tracks QA (issue, spreadsheet, or appendix to this file).

---

## Revision log

| Date | Change |
|------|--------|
| 2026-04-29 | `crates/tonet` removed direct Cargo dependency on `tonet-engine`; current bridge uses source-level `#[path]` shims for `css/html/policy/limits/document_url` to keep behavior stable during in-crate extraction. |
| 2026-04-29 | Servo default policy/docs refresh: commands use default features (`cargo check/test/build -p tonet`), A1 CI row updated; policy now includes `tonet://` on Windows Servo path. Added A6 row for Servo-served internal `tonet://` pages (`settings/history/downloads`) and action queue wiring (clear history/downloads/permissions). |
| 2026-05-24 | Initial checklist file; visit policy tests; Servo favicon sync; Windows CI workflow; stable toolbar widget ids (`ui::push_id`); `cancel_in_flight` no longer clears Servo tab fields; stop button skips `cancel_in_flight` when Servo supersedes DOM paint. |
| 2026-05-25 | IME: egui → Servo `InputEvent::Ime`; platform matrix + single-WebView doc; `Cargo.toml` note on `servo` 0.1.x pin. |
| 2026-05-26 | Win32 popup: `WebView::cursor` → `WM_SETCURSOR` + stock IDC cursors; matrix column “Cursor from engine”. |
| 2026-05-27 | Script dialogs: `TonetServoWebViewDelegate::show_embedder_control` queues `SimpleDialog`; egui modal + `spin_event_loop` after response; `Drop` on `ServoWinHost` pumps once; A6 checklist split (dialogs vs downloads/permissions). |
| 2026-05-28 | Context menu: queue `ContextMenu`, `hide_embedder_control`, egui window (`servo_context_menu_title`); script dialog pre-empts menu; `show_embedder_modals` runs dialog UI then menu UI with one `spin` if needed. |
| 2026-05-29 | Context menu placement: `ClientToScreen` + `ScreenToClient` map popup → owner egui points; prefer last `WM_RBUTTONUP` pixel clamped to Servo’s hit rect; stable `Window` id; `constrain` + fixed position. |
| 2026-05-30 | Unsupported embedder controls: classify in `TonetServoWebViewDelegate`, `take_unsupported_embedder_notice`, i18n toasts (7s) under top chrome; `servo_dialog_ok` / `servo_context_menu_title` gated to `servo-engine`+Windows in i18n. |
| 2026-05-31 | Native `<select>`: queue `SelectElement`, `show_select_element_if_pending` (optgroups + radios, OK/Cancel), pre-empt / `hide_embedder_control` aligned with dialog + context menu. |
| 2026-06-01 | `<input type=color>`: `ColorPicker` queue + `show_color_picker_if_pending` (sRGB edit + OK/Cancel); context menu defers if color open; `ServoUnsupportedEmbedderKind` no longer includes color. |
| 2026-04-14 | File: `FilePicker` + `rfd` / `poll_file_picker_completion`; IME: remove toast / `ServoUnsupportedEmbedderKind`, `InputMethod` no-op (egui → Servo composition unchanged). |
| 2026-04-14 | Permissions: egui Allow/Deny + i18n + `servo_permissions.json` (`permission_store`); fast path + modal save; pre-empt / defer; `Drop` teardown denies pending permission without writing the file. |
| 2026-04-14 | Context menu: “Open link in new Tonet tab” (`element_info().link_url`, `http`/`https` only) → `open_new_tab_with_url` + smoke step 10. |
| 2026-04-14 | HTTP auth: `request_authentication` → egui modal + i18n; modal order + `Drop`; smoke step 9. |
| 2026-04-14 | `visit_policy` tests: `is_http_or_https_history_url` + trim + uppercase scheme; A8 / A7 checklist tweaks; manual smoke renumbered 9–14. |
| 2026-04-14 | Manual smoke: steps 15–20 (notifications, console, heuristic download, clear permissions, stop no-op, login/media/PDF spot-check); A8 manual row → `[~]`. |
| 2026-04-14 | Servo pin playbook + default-engine **policy** sections; A1 pin → `[x]`, product row → `[~]`; A7 perf template → `[~]`; clear **Downloads** clears Servo ephemeral queues + all-tab `servo_console`; B1 clear-data row → `[~]`; `show_clear_confirm_modal` takes `FnMut(ClearTarget)`. |
| 2026-04-14 | `content_disposition::parse_filename_value` + unit tests; **§ Corpus comparison** procedure; A7 corpus row → `[~]`; A8 tests row mentions `content_disposition`. |
| 2026-04-14 | `parse_filename_value`: light `%HH` percent-decode; tests; checklist **§ Linux / macOS Servo embed**, **§ A3 design backlog**, **§ Accessibility audit templates**, **§ i18n coverage matrix**; A2/A3/A5/B2/B3 rows → `[~]` where templates apply; `ServoViewportRuntime::tick` doc (non-Windows no-op). |
| 2026-04-14 | Unit tests: **`download_heuristic`** (PDF, query string, `ftp`/`file`, no extension, multi-dot, `.tar.gz`/`gz`); **`background_download`** `sanitize_filename` (forbidden chars, length cap + trim order). |
| 2026-04-14 | Unit tests: **`visit_policy`** (incomplete load, empty/whitespace committed URL); **`content_disposition`** (invalid `%` escapes, unclosed quoted `filename`). |
| 2026-04-14 | **`background_download`:** `suggested_save_name_for_download` (header vs URL fallback, bad URL, trailing `/`, whitespace-only `filename`); `suggested_save_name` delegates to it. |
| 2026-04-14 | Save-as default from URL: **last non-empty** path segment (e.g. `…/releases/` → `releases`); bare origin `/` still `download`; tests updated. |
| 2026-04-14 | **`download_heuristic`:** extension gate uses **last non-empty** path segment (aligns with save-as; e.g. `…/app.zip/` intercepts); root `/` still no intercept. |
| 2026-04-14 | **`servo_engine::url_path`:** `last_non_empty_path_segment` shared by **`download_heuristic`** + **`background_download`**; A8 row + unit tests. |
| 2026-04-14 | **`content_disposition`:** tests + doc for `filename*` without `''` (fallback to `filename=`), `inline`, `filename*`-only broken form. |
| 2026-04-14 | **`parse_filename_value`:** `;`-split token parse (avoids `notfilename=`); empty `filename*=` after `''` falls back to plain `filename=`; A8 row + tests. |
| 2026-04-14 | **`content_disposition`:** `disposition_params` — split on `;` only outside ASCII quoted spans; semicolon inside quoted `filename`; unquoted `rest` no longer uses inner `;` cut (param is already one slice). |
| 2026-04-14 | **`content_disposition`:** `\"` handling for `;` splits + quoted `filename=` closing (`is_backslash_escaped_dquote` / `slice_before_unescaped_dquote`). |
| 2026-04-14 | **`content_disposition`:** `unescape_quoted_filename_body` for quoted `filename=` (`\\`, `\"`) before `%HH`; A8 row + doubled-backslash test. |
| 2026-04-14 | **`background_download`:** `sanitize_filename` maps `char::is_control()` to `_`; test; §13.5 code map row for `background_download.rs`. |
| 2026-04-14 | **`sanitize_filename`:** strip trailing **`.`** and ASCII **space** (Win32 save-as); A8 row + tests. |
| 2026-04-14 | **`sanitize_filename`:** leading **`_`** when stem is reserved DOS device (`CON`, `COM1`–`9`, `LPT1`–`9`, …); `COM10` unchanged; A8 row + tests. |
| 2026-04-14 | **`visit_policy`:** tests for `javascript:` / `data:` / `blob:` / `file:` (not `http(s)` history); A8 row. |
| 2026-04-14 | **`download_heuristic`:** tests + module note for `ws` / `wss` (no intercept); A8 row. |
| 2026-04-14 | **`parse_filename_value`:** trim leading ASCII space + strip UTF-8 BOM (`U+FEFF`); test; A8 row. |
| 2026-04-14 | **`visit_policy`:** `ws` / `wss` / `chrome` scheme tests; **`download_heuristic`:** `HEAD` does not intercept; A8 row. |
| 2026-04-14 | **`url_path`** + **`visit_policy`:** tests (single path segment, IPv6 URL / host literal); A8 row. |
| 2026-04-14 | **`download_heuristic`** / **`background_download`:** IPv6 `https` intercept + save-as name from path; A8 row. |
| 2026-04-14 | **`download_heuristic`:** `OPTIONS` / `PUT` / `DELETE` do not intercept; module doc notes **GET**-only; A8 row. |
| 2026-04-14 | **`background_download`:** `sanitize_filename` tests `COM0` (no prefix), `PRN` (prefix); **`visit_policy`:** `should_record_visit` for IPv6 `https`; A8 row. |
| 2026-04-14 | **`visit_policy`:** `last_recorded_url` vs trimmed committed (whitespace mismatch records again); **`servo_supersedes_dom_paint`** unit tests (Windows + `servo-engine`); A8 row. |
| 2026-04-14 | **`download_heuristic`:** `PATCH` does not intercept; **`url_path`:** double-`/` path segments; **`content_disposition`:** ASCII case-insensitive `filename` / `filename*` tokens; A8 row. |
| 2026-04-14 | **A3:** Servo idle path `about:blank` + throttled `spin_event_loop` when active tab is not `http(s)`; **B1:** omnibox visit-history autocomplete (`browser_log` + `chrome/toolbar`); A3/B1 rows → `[~]`; §Single WebView idle bullet. |
| 2026-04-14 | **B1:** omnibox history list **keyboard** (↑/↓ highlight, Enter open; Enter without row = navigate typed URL); B1 row. |
| 2026-04-17 | **Downloads (A6):** narrow **`HEAD`** before intercept for extensionless paths ending in `download` / `export` / `attachment` (`download_heuristic` + `background_download::head_suggests_intercept_binary_get`); A6 row + unit tests. |
| 2026-04-17 | **B1:** omnibox history **Escape** clears keyboard-highlighted row (without leaving the address field). |
| 2026-04-17 | **Manual smoke:** step **17b** (extensionless `…/download|export|attachment` + `HEAD` / `Content-Disposition` / MIME heuristic download). |
| 2026-04-17 | **B1 / B3:** omnibox history **keyboard hint** (`i18n::omnibox_history_keyboard_hint`); **`background_download`** unit tests for `HEAD` **MIME** allowlist (`mime_*`). |
| 2026-05-02 | **Policy + matrix:** Windows default = **in-process** Servo embed (readback → egui); optional **`TONET_SERVO_WIN32_POPUP=1`** legacy owned popup. Aligns with `TONET_VISION.md` §13 and `runtime_win.rs` module docs. |
| 2026-05-02 | **B2:** `chrome/ids.rs` centralizes stable `egui::Id` values for toolbar, tab cells, and **+** new-tab; unit test locks string names. |
| 2026-05-02 | **A8:** `download_heuristic` tests — `HEAD` probe excluded on redirect / subresource / `POST`; `attachment` path segment (case-insensitive). |
| 2026-05-02 | **A8:** `servo_engine::url_path` — deep trailing `/`, `..` normalization, percent-encoded last segment (raw `path_segments` string). |
| 2026-05-02 | **A8:** `content_disposition::parse_filename_value` — empty quoted `filename`, param order, unquoted spaces, `filename*` UTF-8 `%` octets. |
| 2026-05-02 | **A8:** `visit_policy` — `mailto:` / `magnet:` excluded from history URL gate + `should_record_visit`; `.cursor/rules/pr-workflow.mdc` — batch related PRs when reviewable. |
| 2026-05-02 | **B2:** `servo_engine/embedder_ids.rs` — stable `egui::Id` for Servo embedder UI (`runtime_win` + page console strip in `app.rs`); script dialogs use `tonet_servo_simple_dialog`; unit test locks names. |
| 2026-05-02 | **A8:** `visit_policy` — `view-source:` and `gopher:` excluded from `http(s)` history URL gate. |
| 2026-05-02 | **B2:** `chrome/ids` — `omnibox_history_scroll()` for visit-history popup scroll area. |
| 2026-05-02 | **A8:** `visit_policy` — `tel:` / `sms:` excluded from history URL gate. |
| 2026-05-02 | **B2:** `chrome/ids` — `omnibox_history_popup_layer(omnibox_id)` for visit-history overlay Area. |
| 2026-05-02 | **A8:** `download_heuristic` — `CONNECT` / `TRACE` do not intercept main-frame GET heuristic. |
| 2026-05-02 | **A8:** `visit_policy` — `rtsp:` / `irc:` excluded from history URL gate. |
| 2026-05-02 | **B2:** `ui::settings_modal_id` / `settings_internal_form_id`; **`visit_policy`** — `about:` excluded from history URL gate. |
| 2026-05-02 | **A4 / UX:** toolbar **Stop** disabled while loading on Servo-superseded tabs + localized tooltip (`stop_loading_unavailable_servo_tooltip`); **`servo_supersedes_dom_paint`** tests (empty URL, `javascript:`, `ftp:`); manual smoke step 19 + A4 row updated. |
| 2026-05-02 | **A1:** Engine HTML/CSS + `document_url` / `limits` / `policy` canonical in `crates/tonet/src/`; `tonet-engine` is a thin re-export for tests; removed `tonet-engine/src/css|html` and duplicate policy files. |
