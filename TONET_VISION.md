# Tonet — product vision and quality gates

This document turns the long-term browser ambition into **verifiable phases**. It is the contract for what “ready” means before investing in large refactors (e.g. standalone engine crate, corpus CI). **Numbers marked TBD** must be filled from measured baselines on the **reference machine** (see below); until then they are placeholders, not promises.

**Repository language:** English (this file). User-facing UI may follow Settings → Language.

---

## 1. Strategic intent

- **Own stack:** networking, parsing, layout/rendering path owned by Tonet—not Chromium, Firefox, WebKit, or CEF.
- **Compete seriously, but measure:** progress is judged against **frozen corpora**, **conformance subsets**, and **hard metrics**, not slogans.
- **Security and updates are non-optional:** they run in parallel with features, not after “feature complete.”

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
5. **CSS (author)** — **in progress**: **tokenizer** + **top-level rule split** + **declarations** + **simple selector cascade** (`css::author_cascade`: type / `.class` / `#id`, specificity + order); **`html` / `body` type rules** supply document-wide defaults for typography-like properties (including **`text-decoration: underline`**, **`text-align`** (`left` / `start`, `center`, `right` / `end`; not `justify` yet), **`line-height`** as `normal`, unitless number, `px` / `em` / `rem`, `%`, **`letter-spacing`** as `normal` or `px` / `em` / `rem`, and **`text-transform`** (`none`, `uppercase`, `lowercase`, `capitalize` via Unicode UAX \#29 + CSS Text semantics for `capitalize`; locale-aware casing when document `lang` is wired), **`text-indent`** (`px` / `em` / `rem` / `%`, merged with `html`/`body`; `%` vs read-area width at paint; first line via egui `LayoutJob` `leading_space`), **`opacity`** (`0`…`1` or `%`, merged with `html`/`body`; scales resolved text color in gamma space), **`visibility`** (`visible` / `hidden` / `collapse` ≡ `hidden` until tables; merged with `html`/`body`; egui `add_visible` keeps layout space), **`display`** (`none` skips the matched `DomNode` with no margins; other supported keywords keep layout; **not** merged from `html`/`body`—non-inherited) before per-node rules; **`margin`** shorthand plus **`margin-top` / `margin-bottom`** longhands (length units) adjust vertical spacing and **do not** inherit from `html`/`body`. The shell **fetches** sheets and maps to egui. **No** full box model, combinators, true DOM inheritance tree, or pseudo-classes yet. Next: broader selectors + layout per §5.
6. **Appearance (light/dark)** — **partially met**: `tonet` uses a thread-local `UiTheme` and `theme.rs` so chrome, settings, and page chrome colors track the same palette; extend when layout needs author-driven constraints.
7. **Next (gates / measurement):** **cookie/cache** persistence design (**Gate C**); grow HTML/CSS corpora; fill §4 **TBD** budgets on the reference machine (§9).

### When does author CSS paint the page?

**Partially, for a deliberate subset.** The read path still uses Tonet’s layout model, but **author** `color`, `font-size`, **`line-height`**, **`letter-spacing`**, `font-weight`, `font-style`, **`text-decoration`** (underline / none), **`text-align`** (`left`/`start`, `center`, `right`/`end`; LTR), and **`text-transform`** (`none`, `uppercase`, `lowercase`, `capitalize` per CSS Text + UAX \#29; document `lang` not yet wired for locale-specific casing), **`text-indent`** (`px` / `em` / `rem` / `%`, first line in read view), **`opacity`** (number or `%`, text color only in read view), **`visibility`** (`visible` / `hidden` / `collapse`), and **`display: none`** (only when a matching rule sets it on that node—not from `html`/`body` defaults) from **simple selectors**—one token: type (`p`), class (`.lead`), or id (`#main`)—are resolved after fetch + parse and applied when drawing `DomNode` text in egui. **`html` / `body` type rules** fill typography-like properties when a node does not declare them (not a full inheritance engine). **`margin`** shorthand (1–4 lengths) feeds **`margin-top` / `margin-bottom`** when longhands are absent; longhands win over shorthand in the same rule block. Margins are **not** inherited from `body`. Specificity is **id > class > type**; ties use source order (including duplicate properties in one block). There are **no** combinators, pseudo-classes, or a general box model yet. Broader “CSS drives layout” still needs **§5 layers 3–4** (full cascade + box model).

---

## 13. Revision

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

Update this file when phases complete, budgets change, or the reference machine changes.
