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
5. **CSS (author)** — **in progress**: minimal **syntax tokenizer** in `tonet-engine` (`css::syntax`); the desktop shell **fetches** linked stylesheets after navigation and **tokenizes** them into `Tab.loaded_stylesheet_tokens`—**not** applied to layout yet. Next: cascade + box model per §5.
6. **Appearance (light/dark)** — **partially met**: `tonet` uses a thread-local `UiTheme` and `theme.rs` so chrome, settings, and page chrome colors track the same palette; extend when layout needs author-driven constraints.
7. **Next (gates / measurement):** **cookie/cache** persistence design (**Gate C**); grow HTML/CSS corpora; fill §4 **TBD** budgets on the reference machine (§9).

---

## 13. Revision

| Date | Change |
|------|--------|
| (initial) | Created vision + gates template. |
| 2026-04-14 | Marked engine + corpus CI steps done; noted redirect cap; next steps for Appearance / conformance / storage. |
| 2026-04-15 | Documented HTML read path + stylesheet discovery + first CSS syntax tokenizer; clarified Appearance status and Gate C / metrics next steps. |
| 2026-04-16 | Stylesheet GET after navigation (capped per `EngineLimits`); vision note on fetch vs apply. |
| 2026-04-17 | Tokenize fetched stylesheet bodies on the tab (`tokenize_stylesheet_bundle`). |

Update this file when phases complete, budgets change, or the reference machine changes.
