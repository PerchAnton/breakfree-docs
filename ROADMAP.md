# BreakFree Docs — Development roadmap

**Stage: Rust D1 loop and native Home ribbon · Next gate: native Windows input/visual validation**
**Reviewed: 2026-10-16.** The first standalone writing loop is implemented as a
reversible experiment. D0 adoption, license, accessibility and cross-platform gates
remain open. The detailed long-term plan is preserved below. Milestone names are
planning identifiers, not published releases; no readiness percentage is claimed.

[Overview](README.md) · [Initial acceptance scenarios](docs/REQUIREMENTS.md) · [Suite plan](https://github.com/PerchAnton/breakfree-office/blob/main/ROADMAP.md) · [Evidence rules](https://github.com/PerchAnton/breakfree-office/blob/main/docs/DEVELOPMENT_STANDARD.md)

The owner-defined release cycle is `0.1.0-alpha.N` → beta → RC → `0.1.0`;
the `0.2.0` cycle starts after actual first stable publication. See
[VERSIONING](docs/VERSIONING.md) and [CHANGELOG](CHANGELOG.md). D0–D8/R1 remain
feature/evidence milestones; version numbers do not mark those gates complete.
Every delivered change set increments the current prerelease counter and updates
CHANGELOG, including documentation-only work. Every work report includes the
current application version for owner approval.

## 1. Product destination and current position

Build a local-first word processor that grows from everyday writing into structured, reviewable and interoperable documents. A visible feature is not complete until editing, undo, persistence and output work together.

| Dimension | Current evidence | Required evidence later |
| :--- | :--- | :--- |
| Feature coverage and depth | Basic D1 prototype; [acceptance evidence](docs/VALIDATION.md) | Versioned feature inventory and expanded workflow tests |
| Layout and typography | Rust galley wrapping/alignment and hit-test assertions; no pagination/fidelity measurements | Semantic assertions and controlled visual fixtures |
| DOCX fidelity | Not implemented | Separate import, edit, export and reopen reports |
| File safety and recovery | Write-then-replace and fault-injection tests; no crash recovery | Interrupted-process, power-loss and recovery tests |
| Performance | No baseline | Published hardware, corpus, versions and timings |
| Accessibility and localization | Rust widget event tests; assistive technology and localization pending | Native platform, assistive-technology and language reviews |
| Platforms and distribution | Rust native executable; local macOS build/render checks | Installation and workflow evidence per OS/architecture |
| Automation | Process launch/capability adapter v1, provisional Office baseline | Published Office contract reconciliation and launcher tests |

Readiness will be assessed separately for short everyday documents, long structured reports and review-heavy professional work. Do not publish a single percentage until the feature inventory, weighting and measurements exist.

## 2. Proposed architecture

**Approved language:** Rust for model, editing, storage, integration and GUI. No
further language approval is needed. The owner selected desktop Word 2024 for
Windows as the visual reference. The native eframe/egui ribbon is a replaceable
implementation under [ADR 0002](docs/adr/0002-rust-native-ribbon.md); toolkit,
accessibility, licensing and platform gates remain open. The superseded Python
experiment is retained in `prototypes/python/` as historical evidence.

| Boundary | Responsibility |
| :--- | :--- |
| Document model | Stable identifiers; paragraphs, runs, styles, sections, tables and references |
| Edit operations | Validated transactions, selection semantics, undo/redo and history grouping |
| Text and layout | Shaping, bidirectional text, font fallback, line breaking and pagination |
| File adapters | Versioned native persistence and separately scoped DOCX/ODT/RTF adapters |
| Application engine | Sessions, command registry, jobs, cancellation and document events |
| Clients | Desktop UI, CLI and later opt-in MCP; no independent copies of editing logic |

Keep unsupported imported data separate from editable content. Preserve it only when safe and valid after edits; otherwise warn, offer a new output file or refuse a destructive operation. A native save is not the same operation as a compatibility export.

**Dependency rules:** the model and edit engine do not depend on a window toolkit; layout must be testable without a window; file parsing must not require network access. Define document-format versions, text-index conventions, resource limits and migration behavior before committing to public APIs.

## 3. Feature map

Rows describe intended coverage; only the narrow D1 subset in the evidence report
is implemented. All wider coverage remains planned. Track read/write separately.

| Area | Intended coverage | First phase |
| :--- | :--- | :--- |
| Editing | Caret, selection, paragraphs, clipboard, undo/redo | D1 |
| Text formatting | Emphasis, fonts, paragraph alignment, spacing | D1–D2 |
| Structure | Styles, nested lists, tables, images and anchors | D2 |
| Layout | Pages, margins, sections, columns, headers and footers | D3 |
| Navigation | Search/replace, headings, bookmarks and outline | D2–D3 |
| Output | Print preview, physical printing, PDF export | D3 |
| Interoperability | Scoped DOCX exchange, later ODT/RTF investigations | D4 |
| Review | Comments, revisions, accept/reject and comparisons | D5 |
| Long documents | Footnotes, endnotes, contents, citations and cross-references | D5 |
| Specialist authoring | Equations, templates, forms and mail merge | D5 |
| Automation | CLI, batch work and optional MCP | D1 foundation; D6 expansion |
| Assistant features | Explicit, previewable and undoable suggestions | D8 research |

## 4. Milestones and exit criteria

### D0 — Decide and prove the foundation

- [x] Record the owner-approved Rust language and Word 2024 Windows visual target (ADR 0002).
- [ ] Review license, first supported target OS, toolkit and native format adoption.
- [ ] Prototype caret movement, input methods, clipboard, font fallback and accessibility on the first target.
- [x] Define prototype invariants, command parameters, undo boundaries and the tested failure model for saves (ADR 0002 and NATIVE_FORMAT.md).
- [x] Create a minimal standalone shell, headless test runner and reproducible build instructions (README and local validation).
- [x] Configure Rust formatting, Clippy, core/widget/CLI tests and binary build in CI.
- [ ] Review exact dependency licenses and enforce architectural boundaries beyond feature separation.

**Exit:** an owner-reviewed decision record and a shell another contributor can build. A toolkit demo alone does not pass the text-input investigation. No build commands are advertised before validation.

### D1 — A reliable writing loop

- [x] Implement prototype paragraphs, selections, typing, deletion, bold/italic and paragraph alignment.
- [x] Route edits through validated commands; expose the same engine to headless tests.
- [x] Add undo/redo, explicit save/discard/cancel choices and versioned local save/reopen, covered by local tests.
- [x] Handle tested write failures, malformed files and the tested keyboard workflow on the evaluation host.
- [ ] Run all D01–D09 scenarios in [REQUIREMENTS.md](docs/REQUIREMENTS.md).

The last gate remains open: automated cases cover the initial loop, but complete
native dialog/IME, accessibility and target-platform reviews are outstanding.

**Exit:** create → edit → save → reopen → continue preserves supported content, and every initial acceptance scenario has evidence. This is the first implementation milestone, not a DOCX milestone.

### R1 — Rust ribbon and native-platform proof (depends on D1 data-safety tests)

- [x] Migrate all runtime layers to Rust; reopen the native v1 fixture unchanged.
- [x] Implement custom title/search, File, Home groups, white continuous canvas,
  real statistics, navigation, formatting pane and adaptive ribbon overflow.
- [x] Cover shared editing, format/history, lifecycle, transient IME and synthetic
  100/125/150% geometry with actual Rust widget events.
- [x] Refine Home icon density, two-row Font/Paragraph, style previews, title/search,
  File Home/Options and status zoom; inspect native wide/narrow/File captures and
  test individual control containment and nonoverlap at responsive breakpoints.
- [ ] Capture Word 2024 Windows and Docs at matched build/language/theme/window,
  document zoom and Windows 100/125/150%; record measured deltas in UI_SPEC.
- [ ] Verify native Windows keyboard, IME, dialogs, font fallback, clipping and focus.
- [ ] Review context tabs when document objects exist; full KeyTips and vertical ruler.

**Exit:** matched screenshots with recorded measurements, no Home/workspace overlap,
all active commands change real state, native D01–D09 reviewed, no fake pagination.
Rust selection is complete; this gate evaluates GUI behavior and visual fidelity.

### D2 — Everyday document structure

- [x] Add direct font size and underline through shared commands and Home controls;
  preserve mixed selections, pending input, undo/redo and v1-read/v2-save content.
  [ADR 0003](docs/adr/0003-character-size-and-underline.md) and
  [validation](docs/VALIDATION.md) record the narrow alpha.3 increment; this does not
  complete D2 or the native-platform R1 gate.
- [ ] Add paragraph/character styles with inheritance and direct-formatting rules.
- [ ] Add lists, indentation, tabs, spacing, links and Unicode-aware search/replace.
- [ ] Add table editing and images with explicit sizing and anchoring semantics.
- [ ] Add navigation, command search and understandable tool states.
- [ ] Expand fixtures for English, Ukrainian, Russian, combining characters and mixed scripts; investigate CJK and right-to-left text explicitly.

**Exit:** a synthetic letter and a structured report survive editing, undo and native round trips. Features unsupported by the current text/layout stack are listed, not silently normalized.

### D3 — Pages, layout and output

- [ ] Add page size, margins, breaks, section behavior, headers, footers and numbering.
- [ ] Define pagination rules for tables, images, keep-with-next and paragraph splitting.
- [ ] Use a consistent document/layout model for screen, preview and export.
- [ ] Implement print preview, PDF export and tested physical-printer integration.
- [ ] Declare font substitution, fallback and embedding policies with redistribution checks.

**Exit:** a fixed-font test corpus has reviewed page counts, object positions and output comparisons. Test physical printing separately; a correct PDF is not printer-fidelity evidence.

### D4 — DOCX exchange and public alpha

- [ ] Implement bounded OOXML packaging and an explicit initial WordprocessingML subset.
- [ ] Cover text, runs, styles, lists, tables, images, relationships and section settings incrementally.
- [ ] Report unsupported content before a potentially lossy edit or export.
- [ ] Test native round trips, DOCX round trips and exported files reopened in a named external application/build.
- [ ] Publish per-feature import/write limitations and an alpha package only for tested targets.

**Exit:** D1–D3 core workflows pass, the declared DOCX subset exchanges successfully, and no known release-blocking data-loss issue remains. The alpha label does not imply broad Word equivalence.

### D5 — Review and professional documents

- [ ] Add threaded comments, tracked insertions/deletions and accept/reject behavior.
- [ ] Extend styles, references, contents, footnotes/endnotes and document fields.
- [ ] Investigate equations, cross-references, citation data and bibliography rendering.
- [ ] Add template, mail-merge and form workflows through separately scoped proposals.
- [ ] Expand the compatibility corpus for every newly supported object.

**Exit:** published professional-workflow cases demonstrate review state, pagination and references surviving save/reopen and the declared exchange path. Features in this phase can ship independently; unfinished work remains visibly partial.

### D6 — Automation and an accessible desktop

- [ ] Stabilize command IDs, typed parameters, structured results and versioning.
- [ ] Add CLI batch operations and an optional MCP adapter with the same command semantics.
- [ ] Require explicit file boundaries, write authorization, cancellation and review of destructive operations.
- [ ] Finish keyboard, assistive-technology, high-DPI, theme and localization reviews on declared platforms.
- [ ] Verify that UI and automation produce equivalent document outcomes for the same supported actions.

**Exit:** reproducible headless workflows and reviewed consent/path-boundary tests exist. No server starts automatically and no document is transmitted without an explicit user action.

### D7 — Beta, hardening and 1.0

- [ ] Run parser fuzzing, regression corpora, interrupted-save tests and migration tests.
- [ ] Measure long-document typing, pagination, opening/saving, memory and startup.
- [ ] Test installation, upgrade, uninstall and recovery on each promised platform.
- [ ] Establish private security reporting; audit dependencies, fonts and assets; publish checksums and signing status.
- [ ] Publish known limitations and a reproducible release evidence report.

**Beta gate:** supported everyday workflows pass, with no unresolved release-blocking corruption, recovery or calculation-of-fields defect. **1.0 gate:** every promised release capability and platform has repeatable acceptance evidence and support documentation. Neither gate requires claiming every feature of another word processor.

### D8 — Separately approved extensions

Investigate collaborative editing, browser delivery, sandboxed extensions and optional assistant features. Each requires its own privacy/security design, persistence/conflict semantics, resource budget and owner approval. Imported VBA execution and arbitrary document scripts are not implicitly authorized by this roadmap.

## 5. Testing and performance plan

Test pure model invariants, command/undo round trips, native persistence, malformed input and interchange independently. Use synthetic or licensed documents with known fonts; maintain semantic checks as well as image comparisons. Visual tolerances must be declared, not chosen after seeing failures.

Proposed benchmark fixtures: a short letter; a 100-page report; a 500-page stress document; mixed-script paragraphs; large tables and image-rich reports. These are **test designs, not supported capacity claims**. Establish reference hardware and baseline measurements in D0–D2 before adopting numeric performance budgets. Record cold/warm startup, typing latency, repagination, open/save time and peak memory.

## 6. Work order, evidence and ownership

Critical path: **D0 → D1 → D2 → D3 → D4**. Review features, accessibility and automation can progress in parallel once their model boundaries stabilize. A milestone is complete only with linked tests, fixtures, platform results and maintainer review. Name an accountable owner before starting each milestone; none is assigned by this plan.

### Optional Office integration and concrete dependencies

| Step | Depends on | Completion criterion / current evidence |
| :--- | :--- | :--- |
| I0: independent executable | D0 shell | `breakfree-docs` installs from this repo; headless commands work with the GUI and Office unavailable; tested by Rust subprocesses outside the checkout |
| I1: process contract v1 | I0, owner baseline | `--new`, absolute `--open`, `--capabilities`, `--version`; exact JSON/version, invalid args and failed-open tests; [pinned local snapshot](contracts/v1/BASELINE.md) |
| I2: Office reconciliation | Published Office `contracts/v1/` and ADR | Pin its source commit/version, compare every field/command and record differences; **pending**, no published contract found at inspected revision |
| I3: Office launcher smoke | I2, actual Office launcher, D1 editor | Office starts this same executable with new/open; standalone launch still passes with all sibling repositories absent; **pending** |
| D1 review gate | D0 owner review, data-safety tests, platform input/dialog evidence | D01–D09 evidence reviewed, limitations accepted, no known blocking data-loss bug; **pending** |
| D2 next increment | D1 review, stable command/native fixtures | Choose one small feature; retain exact undo and native round-trip tests rather than starting several adapters |

I2/I3 never block development or standalone builds. Office is an optional launcher,
not the host of the Docs engine. No shared document engine is planned without
demonstrated need in multiple applications. Build inputs must never track another
repository's live main branch.

Turn checklist items into small issues with a user outcome, dependencies, fixture, acceptance test and risk. Prioritize data loss, unsafe input handling and broken core workflows before feature breadth. Do not invent effort estimates from another project's pace; calibrate ranges from completed BreakFree work.

## 7. Specification inputs and revision log

Use public specifications and original test cases, not copied application code. Useful starting points: [ECMA-376 / OOXML](https://ecma-international.org/publications-and-standards/standards/ecma-376/) and [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html). Pin editions and relevant clauses in implementation decisions.

| Date | Change | Implementation impact |
| :--- | :--- | :--- |
| 2026-10-16 | Expanded the independent roadmap, quality gates and app identity | Documentation only; no feature completion claimed |
| 2026-10-16 | Added a reversible standalone D1 editor experiment, local evidence and optional Office I0–I3 dependencies | Narrow implemented subset checked above; D0 adoption, full D1 review and Office reconciliation remain open |
| 2026-10-16 | Owner approved Rust and Word 2024 Windows reference; replaced runtime with Rust and added R1 ribbon proof gate | Native v1/CLI preserved; toolkit adoption and matched Windows visual/DPI evidence pending |
| 2026-10-16 | Owner requested font size and underline as the next small increment; alpha.3 implements shared formatting and native content v2 | Strict v1 reading retained; protocol v1 unchanged. Local history/storage/widget checks and wide/narrow native captures recorded; Windows comparison remains pending |
