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
- [x] Refine Home density, split Paste, menu affordances, inline style previews and
  common font-size stepping; preserve mixed runs/history and 640-DIP overflow.
  Alpha.4 native macOS keyboard/paste, formatting, Save/Open and Cancel/Discard
  smoke passed; automated non-ASCII character typing remains unresolved.
- [x] Give Font its own draft dialog and one shared undo transaction; preserve mixed
  formatting, cancellation and Unicode save/reopen. Split Find primary/options.
  Alpha.5: 48 full / 26 core-only local tests, native macOS keyboard/save smoke and
  wide/minimum-window Home/Font captures; Windows comparison remains open.
- [x] Add tabs-only temporary expansion, dismissal/pinning, Home command/group/menu
  keyboard routing and visible-region F6. Alpha.6: 54 full / 26 core-only tests,
  nine additional overlay DPI cases, native macOS keyboard/save smoke and three
  inspected captures. No canvas reflow or lost selection in the covered scenarios.
- [x] Add Home/File sequential KeyTips, bare-Alt modifier-transition entry, F10,
  prefix/Escape/focus handling and responsive More routes. Alpha.7: 61 full / 26
  core-only tests, nine badge scale cases, four inspected native render captures
  and macOS F10/Alt+H/menu/size/Save/Open smoke. Bare Alt has event-test evidence only.
- [x] Extend KeyTips to the fixed Quick Access Toolbar, all tab entries, narrow
  More tabs and real View commands; add a validated view-only Zoom draft.
  Alpha.8: 68 full / 26 core-only tests, 27 badge width/scale and three modal-scale
  cases, four inspected captures; native macOS QAT/history, View/Zoom, Draw/Help
  and Unicode Save/Open passed. Reference mappings remain provisional.
- [x] Correct Home inter-group separator overlap reported by the owner. Alpha.11:
  8-DIP gaps and divider/control separation pass 54 width/scale/state cases;
  four native macOS captures and normal-binary pointer/keyboard checks inspected.
  Matched Windows/reference acceptance remains separate.
- [x] Alpha.14: refine the existing Navigation pane with one grapheme-safe matcher,
  active result count, previous/next keyboard routing, virtual rows beyond 40,
  bounded highlighted excerpts and document scrolling. Preserve alpha.13 palette
  and icons; [validation](docs/VALIDATION.md#alpha14-navigation-validation) records
  local evidence. This improves the R1 prototype, not the full D1/D2 exit gates.
- [x] Alpha.15: add Match case/Whole words to the shared literal query, with original
  Unicode ranges, keyboard toggles/Escape, bounded errors and replacement/history
  regressions. Preserve palette/icons; [validation](docs/VALIDATION.md#alpha15-search-options-validation)
  records 110/44 local tests, two captures and native macOS Save/Open evidence.
  This depends on alpha.14 Navigation; full linguistic search/D2 remains open.
- [x] Alpha.16: add a read-only Replace All preview and one shared-engine undo
  transaction using alpha.15 query options and original ranges. Validate Unicode,
  mixed formatting, no-op/stale/oversized plans and keyboard cancellation/apply.
  [Validation](docs/VALIDATION.md#alpha16-replace-all-validation) records 121/51
  local tests, two native captures and macOS Save/Open; [ADR 0005](docs/adr/0005-replace-all-preview.md)
  defines style/paragraph rules. Native Windows acceptance and full D2 remain open.
- [ ] Verify native bare Alt/AltGr and rapid modifier taps, contextual/future command
  hints, and compare all mappings against the named Windows reference build. Depends on the native Windows/Word environment; no full parity claimed.
- [ ] Capture Word 2024 Windows and Docs at matched build/language/theme/window,
  document zoom and Windows 100/125/150%; record measured deltas in UI_SPEC.
- [ ] Verify native Windows keyboard, IME, dialogs, font fallback, clipping and focus.
- [ ] Review context tabs when document objects exist; full KeyTips and paginated vertical-ruler behavior.

**Exit:** matched screenshots with recorded measurements, no unintended Home/workspace overlap,
all active commands change real state, native D01–D09 reviewed, no fake pagination.
Rust selection is complete; this gate evaluates GUI behavior and visual fidelity.

### D2 — Everyday document structure

- [x] Add direct font size and underline through shared commands and Home controls;
  preserve mixed selections, pending input, undo/redo and v1-read/v2-save content.
  [ADR 0003](docs/adr/0003-character-size-and-underline.md) and
  [validation](docs/VALIDATION.md) record the narrow alpha.3 increment; this does not
  complete D2 or the native-platform R1 gate.
- [x] Alpha.17: add the supported character-format reset before extending named
  styles. Home, Ctrl+Space, KeyTips and command search use one validated command;
  text/alignment, pending input, no-op redo and exact history/native roundtrip are
  preserved. [ADR 0003](docs/adr/0003-character-size-and-underline.md#alpha17-direct-character-reset)
  defines this narrow rule. This does not complete inherited styles or D2.
- [x] Alpha.18: built-in Normal/No Spacing/Heading 1 inheritance, distinct direct
  overrides, real Home gallery and spacing, one-operation undo and strict v3
  roundtrip with v1/v2 appearance preservation. Depends on alpha.17 reset and
  alpha.16 replacement snapshots; [ADR 0006](docs/adr/0006-built-in-paragraph-styles.md).
- [x] Alpha.19: Modify Style draft for the three built-in definitions, parent
  selection, cycle/range rejection, Cancel and one undo. Changing Normal updates
  inherited properties only; definitions survive v4 save/reopen, text editing and
  batch replacement. Depends on alpha.18; [ADR 0007](docs/adr/0007-editable-style-definitions.md).
- [x] Alpha.20: create named custom paragraph styles with validated IDs/names,
  built-in/custom parent selection, optional application and one undo. Home menu,
  gallery, search, Unicode, v5 save/reopen, v1–v4 reading and failed-write retention
  are covered. Depends on alpha.19; [ADR 0008](docs/adr/0008-custom-named-styles.md).
- [x] Alpha.21: rename custom styles without changing IDs, paragraph/descendant
  references or layout; validate names and combined definition edits in one undo.
  Save/reopen, failed-write retention, unchanged redo and unused-style targeting
  have regressions. Depends on alpha.20; [ADR 0008](docs/adr/0008-custom-named-styles.md#alpha21-renaming).
- [x] Alpha.22: delete custom styles with separate paragraph/direct-child-parent
  replacements, cycle validation, one undo and exact v5 roundtrip. Direct overrides,
  selection, unused survivors and failed-write retention are covered. Depends on
  alpha.21; [ADR 0009](docs/adr/0009-delete-custom-styles.md).
- [x] Alpha.23: following-style selection and transactional Enter, independent of
  inheritance, with exact undo/redo and native v6 migration. Shared engine and widget
  regressions cover references and keyboard input; native-platform gates stay open.
  [ADR 0010](docs/adr/0010-following-paragraph-styles.md).
- [x] Alpha.24: built-in Strong/Emphasis character styles, semantic run references,
  paragraph → character → direct cascade, selection/pending Undo/Redo and native v7
  roundtrip. Home/KeyTips use the shared engine; Unicode and failed saves are tested.
  Depends on alpha.23; [ADR 0011](docs/adr/0011-character-style-precedence.md).
- [x] Alpha.25: document-owned named character styles, optional properties and
  creation/application in one undo. Validated names/IDs, v8 save/reopen, invalid
  files/failed saves and native GUI creation are covered; palette/icons retained.
  [ADR 0012](docs/adr/0012-named-character-styles.md); depends on alpha.24.
- [x] Alpha.26: modify custom character-style properties with stable IDs/references,
  one Undo/Redo, exact v8 roundtrip, cancellation and invalid/no-op preservation.
  Depends on alpha.25; native Windows acceptance remains open.
- [x] Alpha.27: rename custom character styles with stable references, validated
  names, combined property edits in one Undo/Redo and exact v8 roundtrip.
  Depends on alpha.26; native Windows acceptance remains open.
- [x] Alpha.28: delete custom character styles with explicit replacement/removal,
  all-run and pending-input remapping, one Undo/Redo and native v8 roundtrip.
  Depends on alpha.27; native Windows acceptance remains open.
- [x] Alpha.29: character-style parents, cycle/missing-reference rejection,
  shared cascade/preview, combined one-step history and native v9 migration.
  Deletion remaps direct children with cycle validation; depends on alpha.28.
- [x] Alpha.30: read-only character-property provenance in Formatting, named inheritance sources, independent mixed values/sources and future-typing inspection.
- [x] Alpha.31: alignment/line-spacing/space-after provenance, independent mixed values/sources and half-open selection including empty paragraphs; read-only history/native roundtrip tests.
- [x] Alpha.32: Styles name/scope filtering with reset, empty results, keyboard routing and view-only state.
- [x] Alpha.33: Used in document filter with transitive parent closure, empty paragraphs, exact pending/next-style exclusions, Reset and read-only history/roundtrip checks.
- [x] Alpha.34: label direct use, parent dependencies and unused styles in menus; direct assignment wins for shared parents. Core/widget tests and capture evidence are recorded in validation.
- [x] Alpha.35: show directly assigned consumers of parent dependencies, including transitive inheritance and direct-use parents.
- [x] Alpha.36: navigate from Required by names to focused/scrolled style rows, clearing name/category filters without applying formatting.
- [x] Alpha.37: restore the original parent and exact filters after dependency navigation; pointer/keyboard return, chained jumps and menu-close lifecycle tested.
- [x] Alpha.38: expose the return destination name in the Back control, with bounded width and a full-name tooltip.
- [x] Alpha.39: test a 32-style reverse-order chain and repeated keyboard jumps; handle renamed/deleted/unused return destinations without stale focus or content/history changes.
- [x] Alpha.40: add an 18-case synthetic dependency-keyboard/scale matrix, roundtripped 32-style fixture and reproducible Windows acceptance checklist/review-artifact workflow.
- [x] Alpha.41: direct line-height/space-after overrides, partial/reset dialog, one undo, shared layout/provenance, Enter/Replace All preservation and strict v10 roundtrip; depends on alpha.31.
- [ ] Execute native Windows dependency keyboard and 100/125/150% display-scaling acceptance using docs/WINDOWS_ACCEPTANCE.md; deferred by the owner until a Windows desktop is available. Linked styles and first-line/hanging indents remain separate increments.
- [ ] Complete lists, first-line/hanging indents, tabs and links; search/replace, intervals and left/right indents have incremental implementations above.
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

### Next small toolbar increment

- [x] Alpha.9: window-local command visibility, above/below placement, dynamic numeric hints, keyboard focus and reset; real View 100%/width fitting. Local event/native evidence: [validation](docs/VALIDATION.md#alpha9-toolbar-and-view-validation).
- [x] Alpha.10: persist ordered toolbar commands and placement in a versioned user profile; confirmed reset/backup, unknown-version preservation and failed-write/stale-window handling. Alpha.9 approved; restart/roundtrip, failure, unchanged-document tests and native macOS checks passed. [ADR 0004](docs/adr/0004-local-toolbar-preferences.md), [validation](docs/VALIDATION.md#alpha10-preferences-validation).
- [x] Alpha.12: explicit command reordering through More Commands, keyboard-accessible move controls, draft OK/Cancel, unique ordered commands, hint remapping and stable focus after moves. Uses the alpha.10 settings boundary; five widget regressions, 18 geometry cases and native macOS restart evidence: [validation](docs/VALIDATION.md#alpha12-toolbar-order-validation). Alpha.11 approved; alpha.12 awaits version/review approval.
- [x] Alpha.13: refine icon color roles and light surfaces using owner-supplied macOS visual evidence, retaining original vector artwork, disabled command semantics, geometry and editing. Alpha.13 version approved in chat. Three native captures and the existing 95/36 tests verified; [palette evidence](docs/UI_SPEC.md#alpha13-icon-palette). This does not complete the Windows comparison gate.
- [ ] Compare both toolbar positions and View against a named Word 2024 Windows build at matched sizes/theme/language and native 100/125/150%; attach paired captures and measured differences. Depends on an available reference environment; no similarity percentage.

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
| 2026-10-16 | Owner approved alpha.3; alpha.4 tightens Home geometry/menus/icons and implements shared font-size stepping | 44 full / 24 core-only local tests, three native widths and macOS keyboard/dialog smoke; Windows reference/native DPI and IME gates remain open |
| 2026-10-16 | Owner approved alpha.4; alpha.5 adds transactional Font dialog and split Find | 48 full / 26 core-only local tests; native macOS dialog/history/save/reopen and four inspected captures; Windows/Word/IME gates remain open |
| 2026-10-17 | Owner approved alpha.5; alpha.6 adds temporary ribbon and Home keyboard navigation | 54 full / 26 core-only local tests; native macOS focus/menu/undo/save checks and three captures; Windows/Word/IME and full KeyTips gates remain open |
| 2026-10-17 | Owner approved alpha.6; alpha.7 adds Alt/F10 and Home/File KeyTips | 61 full / 26 core-only local tests, native macOS sequence/Save/Open smoke and four captures; native bare Alt/AltGr, Windows/Word and remaining KeyTips gates stay open |
| 2026-10-17 | Owner approved alpha.7; alpha.8 extends QAT/all-tab/View KeyTips and adds a view-only Zoom draft | 68 full / 26 core-only local tests, four captures, native macOS keyboard/Unicode Save/Open; Windows/Word/AltGr/IME and exact mapping review remain open |
| 2026-10-17 | Owner approved alpha.8; alpha.9 adds window-local QAT customization/keyboard focus and compact View with 100%/Canvas Width | 74 full / 26 core-only local tests, four native captures and macOS keyboard/Unicode Save/Open; toolbar persistence/reordering and Windows/Word acceptance remain open |
| 2026-10-17 | Owner approved alpha.9; alpha.10 persists QAT settings with reset/backup, conflict detection and retry | 89 full / 36 core-only tests, three native captures and macOS restart/error/Unicode save checks; explicit ordering controls and Windows/Word acceptance remain open |
| 2026-10-17 | Alpha.11 fixes Home separators overlapping Bold/font-family controls | 90 full / 36 core-only tests, 54 gutter cases and four native captures; native macOS formatting/history/temporary Home passed. Alpha.10 and alpha.11 approvals remain pending |
| 2026-10-17 | Owner approved alpha.11; alpha.12 adds keyboard-accessible QAT ordering | 95 full / 36 core-only tests, 18 dialog cases and two native captures; native macOS cancellation/apply/KeyTips/restart passed. Windows/Word acceptance and alpha.12 approval remain open |
| 2026-10-17 | Alpha.13 refines icon accents and neutral shell/File colors from owner-supplied macOS captures | 95 full / 36 core-only tests and three native captures; native formatting/history/File/KeyTips verified. Alpha.12/13 approval and Windows/Word comparison remain open |

- [x] Alpha.42: inherited/direct space-before, partial/reset dialog, provenance, one Undo/Redo, v11 persistence and wrapped-line layout; depends on alpha.41.
- [x] Alpha.43: left/right paragraph indents with style/direct resolution, transactional dialog/reset, provenance, v12 persistence and indented wrapping/alignment.
- [x] Alpha.44: first-line/hanging indents with inherited values, explicit zero/reset, one history operation, distinct row widths/hit-testing and strict v13 persistence.
- [x] Alpha.45: 108 long/mixed-format indent layout cases; logical whitespace caret fix, zoom/edit/history/reopen coverage and compact font-metric regression checks.
- [x] Prototype interactive ruler handles through validated paragraph geometry commands, with one undo per completed drag and Escape/focus/transform cancellation (alpha.46 local engine/widget tests).
- [x] Alpha.47: keyboard entry, marker traversal, half-point/coarse preview, transactional commit/cancel and exact resolved values (local widget tests).
- [x] Alpha.48: per-property mixed resolved amounts, marker-origin indicators, exact ranges and half-open selection semantics; preview/cancel/history verified locally.
- [x] Alpha.50: reset selected paragraph indents to each style chain through ruler context menu/keyboard, with preview/cancel and one Undo/Redo (local core/widget checks). Native Windows acceptance remains deferred.

- [x] Alpha.49: compact horizontal and vertical continuous-canvas rulers, contextual values, shared visibility and scrolling/zoom alignment (local widget validation).
- [ ] Matched Word 2024 Windows ruler comparison; editable margins/tab stops depend on the corresponding layout model.

- [x] Alpha.51: individual left/right/first-line-hanging reset commands, focused-marker Shift+Delete, preview/cancel and one Undo/Redo; unrelated overrides preserved (local tests).
- [x] Alpha.52: inherited/direct/default indent sources and mixed-source counts in marker tooltips/accessibility labels; preview/history and read-only inspection verified locally.
- [x] Alpha.53: F1 opens read-only marker provenance, dismissal restores marker focus without accepting/cancelling preview; keyboard/focus/compact bounds verified locally.
- [x] Alpha.54: complete ruler source lists, bounded scrolling with arrows/Page Up/Down/Home/End, preserved drafts and reopening at the top (local core/widget tests).
- [x] Alpha.55: pointer-wheel source inspection, viewport scoping, residual-scroll isolation and draft cancellation/commit regressions (local widget tests).
- [x] Alpha.56: ruler context-menu source inspection for all four markers, focus restoration, read-only behavior and unchanged reset/cancellation policy (local widget tests).
- [x] Alpha.57: Shift+F10 marker context menu, explicit keyboard navigation/activation, preview-preserving dismissal and cleanup on focus loss/resize (local widget tests).
- [ ] Final accessibility review: inspect and navigate ruler source panels with assistive technology; deferred by the owner in alpha.59.

- [x] Alpha.58: AccessKit marker activation/full-source/preview descriptions, disabled states and menu focus/action regressions; native macOS AX and keyboard smoke verified.
- [ ] Sustained screen-reader speech/cursor acceptance, including long source lists; deferred to the final review by owner instruction in alpha.59. Observed deactivation was manual, not a confirmed system failure.

- [x] Alpha.60: functional Home left-indent stepping through a shared command, mixed/inherited values, clamping, one Undo/Redo and v13 roundtrip tests.
- [ ] Next D2 increment: define tab-stop storage/layout semantics before adding editable ruler tab stops; list structure remains a separate feature.

- [x] Alpha.61: literal Tab storage/input, validated paragraph grid interval, native v14 migration and left-stop layout with Unicode/cursor/history tests.
- [x] Alpha.62: transactional interval dialog, mixed selection, validation/default reset, cancel and one Undo/Redo through the shared command.
- [x] Alpha.63: bounded explicit left positions, native v15 storage, shared one-step history and layout with interval fallback; local migration/editing/Unicode/indent/zoom/cursor checks.
- [x] Alpha.64: transactional left-position dialog with add/update/remove/clear, mixed selection, validation, cancel and one-step history; local keyboard/bounds and save/reopen tests.
- [x] Alpha.65: visible existing left-stop markers with isolated drag preview, cancellation and one shared commit; common-list requirement and indent priority tested locally.
- [x] Alpha.66: keyboard ruler tab selection, exact preview, fine/coarse movement, cancellation and one shared Undo/Redo (local widget checks).
- [x] Alpha.67: ruler click creation, marker-menu deletion and keyboard deletion preview, cancellation and one shared Undo/Redo (local checks).
- [x] Alpha.68: union markers and coverage, mixed-list add/move/remove preserving unrelated positions, conflict rejection and one Undo/Redo (local checks).
- [x] Alpha.69: keyboard ruler tab creation, correction, cancel and one Undo/Redo (local checks).
- [x] Alpha.70: exact numeric position input in keyboard tab previews, strict validation and one Undo/Redo; local core/widget validation.
- [x] Alpha.71: typed center/right stop storage, v16 migration, transactional API and field layout; local core/render tests.
- [x] Alpha.72 step 1: unified Left/Center/Right dialog with staged changes, cancel and one Undo/Redo.
- [x] Alpha.72 step 2: type-specific ruler shapes and explicit mixed-type/coverage labels.
- [x] Alpha.72 step 3: pointer/keyboard/F2 movement and deletion of all types, preserving mixed owners and one transaction (local tests).
- [x] Alpha.73 step 1: window-local creation-type selector, pointer/context-menu and keyboard access.
- [x] Alpha.73 step 2: typed pointer creation with drag preview, cancellation and one transaction.
- [x] Alpha.73 step 3: selected-type keyboard creation and exact F2 input, preserving conflicts, mixed lists and Undo/Redo.
- [x] Review extended typed-tab interaction cases before leaders (alpha.74 local model/widget tests); physical input acceptance remains open.

- [x] Alpha.74: extended typed-tab recovery tests; adjacent collision traversal, mixed-owner atomicity, numeric/deletion recovery and cancellation/re-entry.
- [x] Alpha.75: per-stop leader storage/v17 and bounded dot/dash/line rendering; shared transactional command and ruler retention.
- [x] Alpha.76 step 1: staged leader choice and unified one-operation dialog commit.
- [x] Alpha.76 step 2: illustrative live sample through the native tab renderer.
- [x] Alpha.76 step 3: mixed-leader detection and explicit full-list replacement.
- [x] Alpha.76 step 4: ruler context-menu leader set/reset, preserving nonowners.
- [x] Alpha.76 step 5: keyboard dialog traversal and compact widget validation.
- [x] Alpha.77 step 1: leader coverage in tooltips/widget labels; absent stops distinguished.
- [x] Alpha.77 step 2: context-menu counts and common-value selection.
- [x] Alpha.77 step 3: read-only F1 details in keyboard preview.
- [x] Alpha.77 step 4: transactional F3 leader cycling.
- [x] Alpha.77 step 5: combined position/leader preview through arrows/F2, cancel and one Undo/Redo.
- [x] Alpha.78 step 1: direct 0–3 leader selection in marker preview.
- [x] Alpha.78 step 2: reverse Shift+F3 cycling.
- [x] Alpha.78 step 3: restore original per-paragraph leaders without losing position.
- [x] Alpha.78 step 4: F4 returns from valid numeric input without committing.
- [x] Alpha.78 step 5: explicit choice feedback separate from mixed coverage.
- [x] Alpha.79 step 1: context-menu entry to the shared marker preview.
- [x] Alpha.79 step 2: pointer leader choices with selected-state feedback.
- [x] Alpha.79 step 3: restore original leaders using the same draft.
- [x] Alpha.79 step 4: exact-position entry and return without committing.
- [x] Alpha.79 step 5: Apply/Cancel with invalid-input protection and one Undo/Redo.
- [x] Alpha.80 step 1: F6 entry/return and wrapping Tab/Shift+Tab control traversal.
- [x] Alpha.80 step 2: Enter/Space activation with repeat suppression.
- [x] Alpha.80 step 3: visible accent outline and widget focus.
- [x] Alpha.80 step 4: pointer/keyboard handoff without an intermediate commit.
- [x] Alpha.80 step 5: disabled-control skipping and invalid-input cancellation.
- [x] Alpha.81 step 1: read-only focused-control descriptions.
- [x] Alpha.81 step 2: mode-specific shortcut hints.
- [x] Alpha.81 step 3: shared pointer hover descriptions.
- [x] Alpha.81 step 4: state-specific disabled-control explanations.
- [x] Alpha.81 step 5: detailed accessibility labels preserving selection/disabled state.
- [x] Alpha.82 step 1: F7 help visibility without moving focus or repeated toggles.
- [x] Alpha.82 step 2: pointer Show/Hide help retaining control focus.
- [x] Alpha.82 step 3: window-local visibility across preview re-entry.
- [x] Alpha.82 step 4: numeric caret/selection preservation, including invalid drafts.
- [x] Alpha.82 step 5: accessible toggle state and retained descriptions when hidden.
- [ ] Next: extended preview focus recovery across mode switches and cancellation. Style inheritance remains separate.

- [x] Alpha.83: owner-requested video-guided shell refinement; denser title/Home,
  neutral surfaces, semantic icon accents, sample/name style cards and View refit.
  Local geometry/functional checks and native captures are recorded in UI_SPEC
  and VALIDATION; Windows/reference equivalence is not claimed.

- [x] Alpha.84: reduce permanent instructions in previews/search/styles, retain
  optional keyboard/hover help and real mixed-state information, and refine icon
  color/details from the supplied reference. Local evidence in UI_SPEC/VALIDATION.
