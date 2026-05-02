# Tonet — product vision and quality gates

This document turns the long-term browser ambition into **verifiable phases**. It is the contract for what “ready” means before investing in large refactors (e.g. standalone engine crate, corpus CI). **Numbers marked TBD** must be filled from measured baselines on the **reference machine** (see below); until then they are placeholders, not promises.

**Repository language:** English (this file). User-facing UI may follow Settings → Language.

---

## 1. Strategic intent

- **Own stack:** networking, parsing, layout/rendering path owned by Tonet—not Chromium, Firefox, WebKit, or CEF.
- **Compete seriously, but measure:** progress is judged against **frozen corpora**, **conformance subsets**, and **hard metrics**, not slogans.
- **Security and updates are non-optional:** they run in parallel with features, not after “feature complete.”
- **Servo on Windows (default build):** with **`servo-engine`** (enabled by default in `crates/tonet`), real **`http://` / `https://`** pages render through the upstream **Servo** embed while Tonet keeps tabs, chrome, and settings in **egui** (see §13 and [`docs/SERVO_INTEGRATION_CHECKLIST.md`](docs/SERVO_INTEGRATION_CHECKLIST.md)). The in-process **`tonet-engine`** stack remains for **`tonet://`**, the new-tab page, and for **`http(s)`** when Servo is unavailable (build without `servo-engine`) or **opted out** via **`TONET_SERVO_VIEWPORT=0`**. **Non-Windows** builds still use Tonet’s engine for `http(s)` unless and until a native Servo surface ships (checklist **§ Linux / macOS Servo embed**).

---

## 2. JavaScript — phased

| Phase | Scope | Gate (summary) |
|-------|--------|----------------|
| **Phase 1 (no JS)** | Ship-quality **static / no-JS** experience: DOM, CSS, layout, networking, security, user data/cache foundations. | Corpus + stability + performance budgets for **no-JS** path; JS disabled or ignored by design. |
| **Phase 2 (own JS engine)** | Incremental milestones: event loop → timers → Promises → DOM bindings → selective **test262**. | Documented JS subset + interactive metric + crash budget on extended corpus. |
| **Phase 3 (breadth + speed)** | Wider compatibility and “near-instant” feel: profiling, incremental layout/paint, large corpus. | Public compatibility + performance targets; security gates still hold. |

---

## 3. Compatibility and measurement

- **Frozen site corpus (v0 → vN):** versioned snapshots or fixed URLs with recorded hashes; scripts evaluate pass/fail and (where defined) visual severity.
- **Conformance suites:** HTML/CSS/JS tests in **incremental** slices (e.g. html5lib/WPT/test262 **selected** directories—not “all green day one”).
- **Published targets (examples—replace TBD with measured values):**
  - **Visual:** ≥ **TBD %** of corpus without **serious visual failure** (definition §8).
  - **Stability:** ≥ **TBD %** of automated navigation sessions **without crash** on corpus **vN**.
- **Regression rule:** what is green **stays** green unless an explicit, versioned exception is recorded.

---

## 4. Performance and memory (product priority)

**Ambition:** navigation that feels **almost instant** and **bounded RAM** with **5 tabs** on the reference machine.

Track per release (same build type, e.g. `release`, same flags):

| Metric | Meaning | Budget (v1) |
|--------|---------|-------------|
| Time to first paint (TTFP) | First meaningful pixels for active tab | **TBD ms** (baseline first) |
| Time to interactive (TTI) | Relevant when JS exists | **TBD ms** / N/A in Phase 1 |
| RSS (5 tabs, steady state) | After warm navigation | **TBD MiB** cap target |
| HTTP cache / disk | Size + eviction behavior | **TBD** + policy in §6 |

**Process:** set **TBD** from median of **N ≥ 10** runs on the reference machine; adjust budgets only with a changelog entry.

---

## 5. HTML / CSS — incremental conformance

Aspirational **high** conformance, achieved by **layers**, each with **zero regression** on prior layers:

1. HTML5 parsing (including error recovery) + stable DOM.
2. Cascade, specificity, inheritance (heavy automated tests).
3. Selectors.
4. Box model + normal flow.
5. Flexbox.
6. Grid.
7. Advanced typography (font loading, shaping, fallbacks).

“Perfect” is not a single milestone: it is **monotone improvement** on agreed suites + corpus.

---

## 6. Cache and user data (professional bar)

Design **early**, even if the first implementation is minimal:

- **Cookies**, **storage**, **sessions**, **history**, **HTTP cache**: clear **persistence model**, **schema versioning**, **migrations**, **disk quotas**, **eviction**, **privacy boundaries**.
- **Policies:** documented (e.g. third-party rules when applicable, clearing vs “delete profile”).
- **Gate:** reproducible behavior after cache clear, profile migration, and crash recovery tests.

---

## 7. Security (enterprise-grade ambition)

Ongoing program, not a late phase:

- **Threat model** (living document): STRIDE-style summary, updated when surface grows.
- **Origin model** and isolation roadmap (logical first, stronger isolation later).
- **Hardening:** parsers, URL handling, network stack; **fuzzing** (HTML/CSS/URL; JS when present).
- **Updates:** signed artifacts, secure channel, vulnerability reporting process.
- **Release policy:** e.g. fuzz or crash-rate **regression blocks release** (thresholds TBD once baselines exist).

---

## 8. Definition: serious visual failure

Used for corpus scoring. A run counts as **serious visual failure** if **any** of:

- **Layout break:** main content column unreadable (overlap/collapse) or navigation completely unusable on a **viewport size fixed** for that test.
- **Missing critical content:** visible blank where the reference snapshot shows primary article/body text (not ads).
- **Corrupt typography:** illegible text (wrong encoding handled as engine bug if the page declares charset).

**Not** serious: minor font metric differences, subpixel antialiasing, 1–2px spacing drift—unless they break readability.

Refine this list as the diff tooling improves (pixel diff thresholds may be added later).

---

## 9. Reference machine (for comparable metrics)

Record once and reuse; update when hardware changes.

| Field | Value |
|-------|--------|
| OS | TBD |
| CPU | TBD |
| RAM | TBD |
| GPU / display | TBD |
| Scale factor | 100% / TBD |
| Build | `cargo build --release -p tonet` |

All **TBD** performance numbers in §4 must be measured on this profile unless a row explicitly says “other.”

---

## 10. Extensions — future goal, architecture now

**Not** required for early public milestones, but:

- Keep an **internal API** mindset: stable seams (tabs, navigation, storage hooks, events).
- Plan **permissions**, **isolation**, and **backward compatibility** before shipping an extension host.
- **Gate F (late):** internal “hello extension” proves the model—no store required.

---

## 11. Quality gates (summary)

| Gate | Theme | Must pass before… |
|------|--------|-------------------|
| **A** | Foundations: spec, measurable pipeline, corpus v0, CI smoke, extension **design** only | Large engine refactor or major CSS milestone |
| **B** | Phase 1 (no JS): corpus targets, crash budget, TTFP/RAM budgets (5 tabs) | Declaring “static web ready” for a defined audience |
| **C** | User data platform: persistence, migrations, quotas | Calling storage/cookies “production” |
| **D** | Phase 2 JS: interactive metric + selective test262 growth | Broad JS marketing |
| **E** | Phase 3: large corpus + performance bar | “Competitive feel” claims |
| **F** | Extensions prototype | Public extension ecosystem promises |

---

## 12. Next implementation steps (repo)

Suggested order (already agreed in planning):

1. **This document** (`TONET_VISION.md`) — **done** when merged; fill TBDs from baselines.
2. **`tonet-engine`** crate — **done**: limits (`EngineLimits::STANDARD`), `policy::check_document_size`, navigation helpers; `tonet` uses it for HTTP/fetch budgets and **explicit redirect policy** (`max_http_redirects`, default 10).
3. **Corpus CI** — **done**: `corpus/fixtures/*.html` + `tonet-engine` integration tests; **`.github/workflows/ci.yml`** runs `cargo test --workspace` on Ubuntu and Windows; `tonet-setup` built separately.
4. **HTML static read path** — **in progress**: tokenizer + tree builder; `DomNode` flattening; rawtext `script`/`style`; `<base href>` for link resolution; **`<link rel=stylesheet>` URL discovery** (`extract_stylesheet_candidates`) for a future fetch path. Full HTML5 tree construction remains a §5 milestone.
5. **CSS (author)** — **in progress**: **tokenizer** + **top-level rule split** + **declarations** + **simple selector cascade** (`css::author_cascade`: type / `.class` / `#id` / `#id.class` (one extra class) / `tag.class` / `tag#id` / `tag#id.class` or `tag.class#id` (one class), specificity + order); **`html` / `body` type rules** supply document-wide defaults for typography-like properties (including **`text-decoration: underline`**, **`text-align`** (`left` / `start`, `center`, `right` / `end`; not `justify` yet), **`line-height`** as `normal`, unitless number, `px` / `em` / `rem`, `%`, **`letter-spacing`** as `normal` or `px` / `em` / `rem`, and **`text-transform`** (`none`, `uppercase`, `lowercase`, `capitalize` via Unicode UAX \#29 + CSS Text semantics for `capitalize`; locale-aware casing when document `lang` is wired), **`text-indent`** (`px` / `em` / `rem` / `%`, merged with `html`/`body`; `%` vs read-area width at paint; first line via egui `LayoutJob` `leading_space`), **`opacity`** (`0`…`1` or `%`, merged with `html`/`body`; scales resolved text color in gamma space), **`visibility`** (`visible` / `hidden` / `collapse` ≡ `hidden` until tables; merged with `html`/`body`; egui `add_visible` keeps layout space), **`display`** (`none` skips the matched `DomNode` with no margins; if **`html`/`body` type defaults** resolve to `none`, the whole read subtree is skipped; other root `display` values are **not** copied onto children—non-inheritance), **`white-space`** (`nowrap` → no soft wrap at read width, `LayoutJob` + infinite wrap width; `normal` / `pre-wrap` / `pre-line` merged with `html`/`body`; `pre` / `break-spaces` not yet), **`word-break`** (`break-all` → mid-word soft breaks via egui `break_anywhere`), **`overflow-wrap`** / legacy **`word-wrap`** (`anywhere` / `break-word`; `overflow-wrap` preferred when both names appear), **`max-width`** (`none`, `px` / `em` / `rem` / `%`; **not** inherited—only matching rules; narrows read `Ui` when resolved width is below the read area), **`padding-left` / `padding-right` / `padding-top` / `padding-bottom`** (`px` / `em` / `rem` / `%`; **not** inherited; horizontal inset before `max-width` / `text-align`, vertical strip before/after that row), **`padding`** shorthand (1–4 tokens; all sides from top/right/bottom/left expansion; longhands override shorthand when both appear in the same cascade map) before per-node rules; **`margin`** shorthand plus **`margin-top` / `margin-bottom` / `margin-left` / `margin-right`** longhands (length units): vertical `add_space`, horizontal outer strip before padding / `max-width`; **do not** inherit from `html`/`body`. **`background-color`** and **`background`** shorthand when it is a **single** `<color>` token (same grammar as `color`; **not** inherited; if both properties appear in the author map, the `background-color` key wins); fill behind padding + text; author `opacity` tints the fill like the text). **`border-radius`** (uniform: first `px` / `em` / `rem` / `%` token, or `0`; **not** inherited; `egui::Frame` rounding; clamped vs read width). **`border-width`** / **`border-color`** (uniform width, first token; color as `color`; **not** inherited; `currentColor` when width set but color omitted; `Frame` stroke). The shell **fetches** sheets and maps to egui. **No** full box model, combinators, true DOM inheritance tree, or pseudo-classes yet. Next: broader selectors + layout per §5.
6. **Appearance (light/dark)** — **partially met**: `tonet` uses a thread-local `UiTheme` and `theme.rs` so chrome, settings, and page chrome colors track the same palette; extend when layout needs author-driven constraints.
7. **Next (gates / measurement):** **cookie/cache** persistence design (**Gate C**); grow HTML/CSS corpora; fill §4 **TBD** budgets on the reference machine (§9).

### When does author CSS paint the page?

**Partially, for a deliberate subset.** The read path still uses Tonet’s layout model, but **author** `color`, `font-size`, **`line-height`**, **`letter-spacing`**, `font-weight`, `font-style`, **`text-decoration`** (underline / none), **`text-align`** (`left`/`start`, `center`, `right`/`end`; LTR), and **`text-transform`** (`none`, `uppercase`, `lowercase`, `capitalize` per CSS Text + UAX \#29; document `lang` not yet wired for locale-specific casing), **`text-indent`** (`px` / `em` / `rem` / `%`, first line in read view), **`opacity`** (number or `%`, tints resolved text color and any `background-color` fill in read view), **`visibility`** (`visible` / `hidden` / `collapse`), **`display: none`** (matching rule on that node, or winning `html`/`body` type default `none` for the whole read list), **`white-space: nowrap`**, **`word-break: break-all`**, and **`overflow-wrap`** / **`word-wrap`** (`anywhere` / `break-word`, merged with `html`/`body`), **`max-width`**, **`padding`** shorthand, **`padding-left` / `padding-right` / `padding-top` / `padding-bottom`**, **`background-color`**, **`background`** (single-color shorthand only), **`border-radius`**, and **`border-width` / `border-color`** (uniform width; per-node, not from `html`/`body` defaults) from **simple selectors**—one prelude token: type (`p`), class (`.lead`), id (`#main`), compound `#main.lead` (id + one class), compound `p.lead`, compound `p#main`, or compound `p#main.lead` / `p.lead#main` (tag + id + one class)—are resolved after fetch + parse and applied when drawing `DomNode` text in egui. **`html` / `body` type rules** fill typography-like properties when a node does not declare them (not a full inheritance engine). **`margin`** shorthand (1–4 lengths) feeds **`margin-top` / `margin-bottom` / `margin-left` / `margin-right`** when longhands are absent; longhands win over shorthand in the same rule block. Margins are **not** inherited from `body`. Specificity is **`tag#id.class` / `tag.class#id` > `#id.class` > `tag#id` > `#id` > `tag.class` > `.class` > `type`**; ties use source order (including duplicate properties in one block). There are **no** combinators, pseudo-classes, or a general box model yet. Broader “CSS drives layout” still needs **§5 layers 3–4** (full cascade + box model).

---

## 13. Servo engine in Tonet (experimental embed)

This section documents **how** Servo is wired today, **when** it is active, and how it relates to the rest of the vision (especially §1 “own stack”).

### 13.1 Purpose and scope

- **Goal:** run real **`http://` / `https://`** documents through the **`servo`** crates.io embed (`Servo` + `WebView` + `WindowRenderingContext`) while keeping **Tonet’s shell** (tabs, chrome, omnibox, i18n, settings) in **egui/eframe**.
- **Windows + `servo-engine`:** **`http://` / `https://`** use **Servo** as the only page engine when the viewport is active (default **in-process** surfman readback → egui; optional legacy **owned Win32 popup** with **`TONET_SERVO_WIN32_POPUP=1`**). The in-process **`tonet-engine` + `render_nodes`** path remains for **`tonet://`**, new tab, and for **`http(s)`** only when Servo is **opted out** via **`TONET_SERVO_VIEWPORT=0`** or when the binary was built **without** `servo-engine`.
- **Platform:** **Native Servo viewport is Windows-only today** (ANGLE / surfman; see §13.3). With `--features servo-engine` on **Linux/macOS**, the `servo` crate may **link**, but **`ServoViewportRuntime::tick`** is a **no-op** there until embed work lands — goals and status are in the checklist (**§ Linux / macOS Servo embed**), not “cancelled.”
- **Checklist:** living integration backlog and smoke steps live in [`docs/SERVO_INTEGRATION_CHECKLIST.md`](docs/SERVO_INTEGRATION_CHECKLIST.md).

### 13.2 Build and activation

| Mechanism | Detail |
|-----------|--------|
| **Cargo** | `cargo build -p tonet --features servo-engine` (or `cargo run …`). Without the feature, no Servo types are linked. |
| **User setting** | **Non-Windows:** Settings → System toggle (or `TONET_SERVO_VIEWPORT=1`) where the native viewport is not yet wired. **Windows + `servo-engine`:** the setting does not gate `http(s)`; Servo runs by default. |
| **Environment** | **`TONET_SERVO_VIEWPORT=0`** disables the Servo viewport on **Windows** (built with `servo-engine`) so `http(s)` falls back to Tonet’s in-process engine. **`TONET_SERVO_VIEWPORT=1`** still opts in on **non-Windows** `servo-engine` builds for future embeds. |
| **URL gate** | Only tabs whose address resolves to **`http://` or `https://`** use Servo when the viewport runtime is active. Internal `tonet://` pages and the new-tab flow stay on the Tonet engine + egui stack. |

**Prerequisites (Windows):** MSVC, Windows SDK, ANGLE/mozangle toolchain expectations as documented in the repo (e.g. `mozangle` DLLs next to the binary, `LIBCLANG_PATH` where bindgen needs it). Treat as **developer / CI** requirements until install UX exists.

### 13.3 Architecture (high level)

1. **Shell:** `CentralPanel` still lays out chrome hints; for Servo-superseded tabs it reserves the **central content rect** (minus a small right gutter for a decorative scrollbar strip) and skips **`render_nodes`** for that tab’s web content.
2. **Rendering surface (Windows):** **default** path matches Slint-style **in-process** surfman: Servo paints into a **`GPURenderingContext`**, reads pixels for the reserved rect, and composites into egui (`CentralPanel`). **Optional** **`TONET_SERVO_WIN32_POPUP=1`:** a **borderless owned popup** (`HWND`) is positioned each frame with **`SetWindowPos`** to match that rect (screen coordinates via **`ClientToScreen`** on the winit-owned parent). **`WS_EX_NOACTIVATE`** avoids stealing keyboard focus from the main window so the omnibox keeps working.
3. **Rendering pipeline:** surfman **`WindowRenderingContext`** + **`WebView::paint`** + **`present`** (popup mode) or the in-process readback path (default); egui’s wgpu swapchain is unchanged.
4. **Input:** **default embed:** pointer and wheel are forwarded from **egui** over the content rect (`feed_egui_servo_embed_input` / `feed_servo_slint_egui_pointer` in `runtime_win`). **Popup mode (`TONET_SERVO_WIN32_POPUP=1`):** mouse and wheel use a **`WNDPROC` subclass** on the owned **`HWND`** into **`WebView::notify_input_event`**. Keyboard (both modes): after a click on the page, captured keys are forwarded from **egui** into Servo; **Escape**, **omnibox focus**, or a **click outside** the content rect releases capture. Chrome shortcuts using **Ctrl/⌘** are not consumed by the forwarder. **IME:** while capture is active, egui **`Event::Ime`** is mapped to Servo **`InputEvent::Ime`** (composition start/update/end and dismiss); validate on real hardware for CJK. **Cursor:** each frame reads **`WebView::cursor`**; in popup mode the popup’s **`WM_SETCURSOR`** maps Servo’s cursor to stock Win32 cursors (`LoadCursorW` / `SetCursor`).
5. **Shell sync (Phase 5 embed):** each frame the embed reads **`WebView::url`**, **`page_title`**, **`load_status`**, **`can_go_back` / `can_go_forward`** and updates the active **tab** (omnibox URL when not editing, window/tab titles, loading spinner, back/forward/reload). Navigating from the omnibox does **not** spawn the Tonet HTML fetch thread for those tabs; **`WebView::load` / `reload`** drive the document.

### 13.4 Quality and vision alignment

- **Gates:** Servo does **not** satisfy **Gate A/B** for “Tonet static web ready” by itself; corpus scores for the **Tonet** pipeline remain authoritative until a deliberate policy change.
- **Measurement:** when comparing Servo vs Tonet on the same URL, record **build flags**, **viewport size**, and **reference machine** (§9) so numbers stay comparable.
- **Roadmap (non-blocking):** favicon from `WebView` and `browser_log` visits for Servo navigations are implemented (see checklist). **Script dialogs** (`alert` / `confirm` / `prompt`) are shown as egui windows and wired to Servo’s `SimpleDialog` responses. **HTTP Basic / proxy auth** (`AuthenticationRequest`) uses an egui username/password window (localized); it runs in the embedder modal chain after script dialogs and before site-permission prompts. **Web Notification API** (`WebViewDelegate::show_notification`) surfaces as an **egui toast** anchored under the top chrome (TTL + dismiss; not the OS notification center; cleared with other embedder teardown). **Console** (`WebViewDelegate::show_console_message`): messages queue on the embed host and merge into the active tab; a **bottom strip** in the Servo content area shows a scrolling, bounded log with **Clear** (not persisted; cleared when the tab leaves the Servo `http(s)` + viewport gate). **`load_web_resource` (downloads):** for **main-frame GET** URLs whose path ends in a small allowlisted extension set, Tonet **intercepts** with `204`, runs a **separate** `reqwest` download + native save-as (`rfd`) on a worker thread, then appends **`record_page_fetch`** with `saved_path` (no Servo session cookies on that fetch; `Content-Disposition`-only cases without a matching extension still follow the normal Servo load). **Context menu** (right-click) is shown as an egui window with Servo-provided labels, positioned from the popup’s client coordinates into the Tonet window (near the click when available); `hide_embedder_control` dismisses it when the engine requests. Tonet adds **“Open link in new Tonet tab”** for `http(s)` link hit-tests (new shell tab + navigation; single shared `WebView` stays on the active tab until switched). **`<select>`** uses Servo’s `SelectElement` with an egui chooser (options / optgroups) and `submit`. **`<input type=color>`** uses Servo’s `ColorPicker` with an egui sRGB editor and `submit`. **`<input type=file>`** opens the OS dialog via **`rfd`** on a background thread; **`poll_file_picker_completion`** applies `FilePicker::select` / `submit` or `dismiss` without blocking egui. Servo’s **`InputMethod`** embedder hint is ignored (no separate panel); **IME** still flows as egui composition events into Servo when the page holds keyboard capture (`InputEvent::Ime`), with full CJK validation and edge cases still to prove out on hardware. **Permissions** (`PermissionRequest`) use an egui **Allow / Deny** modal (localized feature names); closing the window denies and **records deny**. Decisions are keyed by **page origin + feature** and kept in a map **loaded at Servo host startup** from **`servo_permissions.json`** under the Tonet config directory (`dirs::config_dir()/tonet/`), and **rewritten after each Allow/Deny** from the modal. On **`ServoWinHost`** teardown, pending embedder UI is dismissed before a final **`spin_event_loop`** (teardown does not write the file). **Clear visit history** (internal History page) also deletes `servo_permissions.json` and clears the in-memory map on the active Servo host. **`BrowserLog::record_page_fetch`** runs for Servo-completed `http(s)` navigations (same timing as visits; **no** `page-snapshots` HTML until we can read committed document bytes from the embed API). **Save-as / attachment:** a **best-effort** `load_web_resource` path intercepts some **main-frame GET** URLs by file extension (`download_heuristic`); cookie-auth and `Content-Disposition`-only cases without a matching extension remain gaps. Still open: multi-tab **multi-WebView** or explicit recycle policy, non-Windows embed, and documented **stop-load** when upstream exposes it.

### 13.5 Code map (for contributors)

| Area | Location (under `crates/tonet/`) |
|------|----------------------------------|
| Feature + URL gating + runtime shell API | `src/servo_engine/mod.rs` |
| Stable `egui::Id` for Servo embedder windows / toast (script dialog, HTTP auth, permission, context menu, `<select>` / color pickers, web notification area) | `src/servo_engine/embedder_ids.rs` |
| Win32 popup, surfman, input subclass, shell snapshot, `WM_SETCURSOR`, script dialogs, HTTP auth modal, site permissions + JSON store, context menu, `<select>` / color / native file (`rfd`), web notification toasts (`show_notification`), page console (`show_console_message` → `Tab::servo_console`), heuristic `load_web_resource` downloads, Drop teardown | `src/servo_engine/runtime_win.rs` |
| Servo permission persistence (`servo_permissions.json`) | `src/servo_engine/permission_store.rs` |
| Central panel, omnibox, navigation, sync after tick | `src/app.rs` |
| Tab fields for Servo title / chrome nav / page console lines | `src/tab.rs` |
| Visit policy tests (no runtime) | `src/servo_engine/visit_policy.rs` |
| Main-frame download URL heuristic (unit tests) | `src/servo_engine/download_heuristic.rs` |
| Last non-empty URL path segment (shared by download heuristic + save-as fallback) | `src/servo_engine/url_path.rs` |
| Blocking `reqwest` download + save-as (`rfd`), suggested filename from `Content-Disposition` / URL | `src/servo_engine/background_download.rs` |
| `Content-Disposition` filename parsing (unit tests) | `src/servo_engine/content_disposition.rs` |
| Favicon PNG encoding | `src/servo_engine/servo_favicon.rs` |

---

## 14. Revision

| Date | Change |
|------|--------|
| (initial) | Created vision + gates template. |
| 2026-04-14 | Marked engine + corpus CI steps done; noted redirect cap; next steps for Appearance / conformance / storage. |
| 2026-04-15 | Documented HTML read path + stylesheet discovery + first CSS syntax tokenizer; clarified Appearance status and Gate C / metrics next steps. |
| 2026-04-16 | Stylesheet GET after navigation (capped per `EngineLimits`); vision note on fetch vs apply. |
| 2026-04-17 | Tokenize fetched stylesheet bodies on the tab (`tokenize_stylesheet_bundle`). |
| 2026-04-18 | Engine: `css::simple_rules` — top-level qualified rules from token stream; `@` skip fix. |
| 2026-04-19 | Desktop: `parse_stylesheet_bundle_to_rules` wired after stylesheet fetch. |
| 2026-04-20 | `css::declarations` for `property: value`; vision FAQ on when author CSS reaches pixels. |
| 2026-04-21 | `ParsedQualifiedRule` + `Tab.loaded_stylesheet_parsed` filled after fetch (parallel to qualified rules). |
| 2026-04-22 | `css::author_cascade` (simple type selectors, last-wins); desktop applies author `color` / `font-size` in `render_nodes`; `DomNodeType::tag_name` public. |
| 2026-04-23 | `DomNode` carries `class`/`id`; cascade supports `.class` / `#id` with id > class > type specificity. |
| 2026-04-24 | Author `font-weight` / `font-style` → `RichText` strong/italics; headings default to weight 700 unless overridden. |
| 2026-04-25 | `cascade_document_defaults` (`html`/`body` type rules) merged into per-node paint hints. |
| 2026-04-26 | Author `margin-top` / `margin-bottom` → vertical `add_space` (not inherited from `html`/`body`). |
| 2026-04-27 | `margin` shorthand; `text-decoration: underline` (+ `none`) merged with `html`/`body`; `RichText::underline`. |
| 2026-04-28 | `text-align` (keywords above) merged with `html`/`body`; read view uses egui row alignment. |
| 2026-04-29 | `line-height` merged with `html`/`body`; resolved vs used `font-size`; `RichText::line_height`. |
| 2026-04-30 | `letter-spacing` merged with `html`/`body`; `RichText::extra_letter_spacing`. |
| 2026-05-01 | `text-transform` merged with `html`/`body`; applied to string before `RichText::new`. |
| 2026-05-02 | `text-transform: capitalize` aligned with CSS Text + UAX \#29 (`unicode-segmentation`); tail of word no longer forced lowercase. |
| 2026-05-03 | `text-indent` merged with `html`/`body`; `LayoutJob` first-line `leading_space`; `%` vs available read width. |
| 2026-05-04 | `opacity` merged with `html`/`body`; `Color32::gamma_multiply` on resolved text color. |
| 2026-05-05 | `visibility` merged with `html`/`body`; `Ui::add_visible` for hidden text + links (layout preserved). |
| 2026-05-06 | `display: none` on matching simple selectors skips the node in read layout; `display` not merged from `html`/`body`. |
| 2026-05-07 | `html`/`body` type default `display: none` skips all read `DomNode`s; other root `display` values still not inherited per-node. |
| 2026-05-08 | `white-space: nowrap` merged with `html`/`body`; read paint uses `LayoutJob` with no max wrap width. |
| 2026-05-09 | `word-break: break-all` and `overflow-wrap` / `word-wrap` (`anywhere`, `break-word`) merged with `html`/`body`; `LayoutJob` sets `break_anywhere` when soft-wrapping. |
| 2026-05-10 | `max-width` (`none`, lengths, `%`) from matching rules only; read `Ui` width cap before `text-align`. |
| 2026-05-11 | `padding-left` / `padding-right` (lengths, `%`) per-node; horizontal strip then existing read layout. |
| 2026-05-12 | `padding` shorthand (1–4) feeds horizontal insets; longhands override shorthand in hint merge. |
| 2026-05-13 | `margin-left` / `margin-right` + `margin` shorthand horizontal sides; outer horizontal strip before padding. |
| 2026-05-14 | `padding-top` / `padding-bottom` + shorthand vertical sides; vertical strip inside horizontal margins. |
| 2026-05-15 | `background-color` per matching node (not from `body`); `Frame` fill behind padding + text; `opacity` tints fill. |
| 2026-05-16 | `background` shorthand → fill when value is one color token; longhand key wins when both appear in the map. |
| 2026-05-17 | Uniform `border-radius` (first length token); `Frame` rounding with or without fill. |
| 2026-05-18 | `border-width` / `border-color` → `Frame` stroke; `currentColor` when color omitted. |
| 2026-05-19 | `author_cascade`: compound `tag.class` prelude; specificity above plain class. |
| 2026-05-20 | `author_cascade`: compound `tag#id` prelude; specificity above `#id`. |
| 2026-05-21 | `author_cascade`: compound `#id.class` prelude (one class); specificity above `tag#id`. |
| 2026-05-22 | `author_cascade`: `tag#id.class` / `tag.class#id` prelude; specificity `(1,1,1)` above `#id.class`. |
| 2026-05-23 | **§13 Servo engine:** documented optional `servo-engine` feature, Windows embed (popup, input, shell sync), activation, vision alignment, and code map; §1 bullet on optional Servo path; **§14** revision table (renumbered from §13). |
| 2026-05-24 | Servo: `docs/SERVO_INTEGRATION_CHECKLIST.md`; `visit_policy` / `servo_favicon`; favicon + visit sync in embed; stable toolbar widget ids; stop does not call `cancel_in_flight` on Servo-superseded tabs; optional Windows `servo-engine` CI job; §13 checklist link and code map rows. |
| 2026-05-25 | Servo: egui IME → `InputEvent::Ime`; checklist platform matrix + single-WebView section; `Cargo.toml` comment on `servo` 0.1.x pin; §13 input bullet (IME). |
| 2026-05-26 | Servo: Win32 `WM_SETCURSOR` + `WebView::cursor` → stock IDC cursors on the popup; §13 input bullet (cursor). |
| 2026-05-27 | Servo: `alert` / `confirm` / `prompt` via `WebViewDelegate::show_embedder_control` + egui modal (`show_embedder_modals`); `ServoWinHost` `Drop` pumps `spin_event_loop` once; §13 roadmap + code map. |
| 2026-05-28 | Servo: context menu (`ContextMenu`) + `hide_embedder_control` in `TonetServoWebViewDelegate`; egui “Page menu” window; §13 roadmap + code map. |
| 2026-05-29 | Servo: context menu anchored via Win32 `ClientToScreen` / `ScreenToClient` + last right-up pixel; §13 roadmap. |
| 2026-05-30 | Servo: toast when color / file / IME embedder controls are dropped (defaults); §13 roadmap. |
| 2026-05-31 | Servo: native `<select>` via `SelectElement` + egui chooser; §13 roadmap. |
| 2026-06-01 | Servo: `<input type=color>` via `ColorPicker` + egui sRGB; toasts for file / IME only; §13 roadmap. |
| 2026-04-14 | Servo: `<input type=file>` via `rfd` + `poll_file_picker_completion`; removed embedder IME toast / `ServoUnsupportedEmbedderKind` (`InputMethod` no-op; composition via egui); §13 roadmap + code map. |
| 2026-04-14 | Servo: `PermissionRequest` egui Allow/Deny + i18n + disk-backed origin+feature map (`tonet/servo_permissions.json`); `ServoWinHost::Drop` still tears down pending UI before `spin_event_loop`; clear visit history also clears permission file + memory; §13 roadmap + code map. |
| 2026-04-14 | Servo: `record_page_fetch` on completed `http(s)` loads (`sync_into_tab`) so the internal Downloads log matches the Tonet-engine path (snapshots TBD); §13 roadmap. |
| 2026-04-14 | Servo: context menu entry **Open link in new Tonet tab** (`link_url`, `http`/`https`); §13 roadmap. |
| 2026-04-14 | Servo: **HTTP / proxy authentication** modal (`AuthenticationRequest`); §13 roadmap + code map. |
| 2026-04-14 | Servo: **Web Notification API** (`show_notification`) → egui top-chrome toast + i18n; cleared on `teardown_pending_embedder_controls`; §13 roadmap + code map. |
| 2026-04-14 | Servo: **Page console** (`show_console_message`) → `Tab::servo_console` + bottom strip in the Servo viewport rect; §13 roadmap + code map. |
| 2026-04-14 | Servo: **`load_web_resource`** heuristic main-frame binary GET → `reqwest` + save-as + `record_page_fetch` (`download_heuristic` / `background_download`); §13 roadmap + code map. |
| 2026-04-14 | Servo: documented **default vs experimental** `http(s)` policy in checklist; **clear Downloads** clears Servo ephemeral embedder queues + all-tab page console; `Cargo.toml` servo upgrade steps; checklist perf budget template. |
| 2026-04-14 | Servo: `content_disposition` tests + checklist corpus §; code map row. |
| 2026-04-14 | Servo: more **`visit_policy`** / **`content_disposition`** unit tests (edge cases for history + filename parsing). |
| 2026-04-14 | Servo: **`background_download`** `suggested_save_name_for_download` + tests (save-as default name from `Content-Disposition` vs URL). |
| 2026-04-14 | Servo: save-as URL fallback uses **last non-empty** path segment (trailing-slash downloads). |
| 2026-04-14 | Servo: **`download_heuristic`** uses the same last-segment rule for extension detection (`…/file.zip/`). |
| 2026-04-14 | Servo: **`servo_engine::url_path`** — shared `last_non_empty_path_segment` for `download_heuristic` + `background_download`; §13.5 code map. |
| 2026-04-14 | Servo: **`content_disposition`** more unit tests (`filename*` without `''`, `inline`). |
| 2026-04-14 | Servo: **`content_disposition`** `parse_filename_value` token-boundary parse + empty-`filename*` fallback. |
| 2026-04-14 | Servo: **`content_disposition`** quoted `filename` may contain `;` (split params outside `"` only). |
| 2026-04-14 | Servo: **`content_disposition`** `\"` before `"` for param split + quoted `filename=` end. |
| 2026-04-14 | Servo: **`content_disposition`** quoted `filename=` `\\` / `\"` unescape before percent-decode. |
| 2026-04-14 | Servo: **`background_download`** `sanitize_filename` strips control chars; §13.5 code map. |
| 2026-04-14 | Servo: **`sanitize_filename`** strips trailing `.` / ASCII space for Win32 save-as. |
| 2026-04-14 | Servo: **`sanitize_filename`** avoids reserved DOS device stems (`COM1`, `NUL`, …) for save-as. |
| 2026-04-14 | Servo: **`visit_policy`** tests for `javascript` / `data` / `blob` / `file` schemes (visit gate). |
| 2026-04-14 | Servo: **`download_heuristic`** documents + tests `ws` / `wss` excluded from intercept. |
| 2026-04-14 | Servo: **`content_disposition`** strips leading UTF-8 BOM before parsing `Content-Disposition`. |
| 2026-04-14 | Servo: **`visit_policy`** / **`download_heuristic`** tests (`ws`/`wss`/`chrome`, `HEAD` vs intercept). |
| 2026-04-14 | Servo: **`url_path`** / **`visit_policy`** tests (IPv6 `https` URLs). |
| 2026-04-14 | Servo: **`download_heuristic`** / **`background_download`** IPv6 `https` coverage tests. |
| 2026-04-14 | Servo: **`download_heuristic`** documents + tests **GET**-only (`OPTIONS`/`PUT`/`DELETE`). |
| 2026-04-14 | Servo: **`sanitize_filename`** `COM0`/`PRN` edge tests; **`visit_policy`** IPv6 `should_record_visit`. |
| 2026-04-14 | Servo: **`visit_policy`** + **`servo_supersedes_dom_paint`** unit tests (URL trim / scheme casing + `last_recorded` normalization note). |
| 2026-04-14 | Servo: **`download_heuristic`** / **`url_path`** / **`content_disposition`** small unit-test additions (`PATCH`, `//` segments, case-insensitive param names). |
| 2026-04-14 | Servo (Windows): idle embedder **`about:blank`** + throttled spins when experimental viewport on but active tab is not `http(s)`; shell **omnibox visit-history** suggestions from `BrowserLog`. |
| 2026-04-14 | Shell: omnibox history suggestions **↑/↓ + Enter** (keyboard parity with click). |
| 2026-04-17 | Servo downloads: optional **`HEAD`** probe for extensionless `…/download|export|attachment` URLs to honor `Content-Disposition` / MIME before intercept. |
| 2026-04-17 | Shell: omnibox history **Escape** clears keyboard row highlight. |
| 2026-04-17 | Checklist manual smoke: step **17b** for extensionless download + `HEAD` probe. |
| 2026-04-17 | Shell: omnibox history keyboard **tooltip** (i18n); **`background_download`** MIME allowlist tests for `HEAD` probe. |
| 2026-05-02 | **§1 / §13:** Servo-default policy on Windows vs `tonet-engine` fallback; **§13.3** documents default in-process embed vs optional Win32 popup (`TONET_SERVO_WIN32_POPUP`); Linux/macOS no-op embed cross-linked to checklist (embed tracked, not abandoned). |
| 2026-05-02 | **§13.5:** `servo_engine/embedder_ids.rs` in code map; stable `egui::Id` for Servo embedder modals + `tonet_servo_simple_dialog` for script `alert` / `confirm` / `prompt`. |

Update this file when phases complete, budgets change, or the reference machine changes.
