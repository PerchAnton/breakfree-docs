# BreakFree Docs — Development roadmap

**Stage: planning · Next gate: D0 — architecture and a buildable shell**  
**Reviewed: 2026-10-16.** This is an original implementation plan. Documentation and branding exist; application features do not. Milestone names below are planning identifiers, not published releases. No dates, delivery estimates or readiness percentages have been measured.

[Overview](README.md) · [Initial acceptance scenarios](docs/REQUIREMENTS.md) · [Suite plan](https://github.com/PerchAnton/breakfree-office/blob/main/ROADMAP.md) · [Evidence rules](https://github.com/PerchAnton/breakfree-office/blob/main/docs/DEVELOPMENT_STANDARD.md)

## 1. Product destination and current position

Build a local-first word processor that grows from everyday writing into structured, reviewable and interoperable documents. A visible feature is not complete until editing, undo, persistence and output work together.

| Dimension | Current evidence | Required evidence later |
| :--- | :--- | :--- |
| Feature coverage and depth | No implementation | Versioned feature inventory and workflow tests |
| Layout and typography | Not measured | Semantic assertions and controlled visual fixtures |
| DOCX fidelity | Not implemented | Separate import, edit, export and reopen reports |
| File safety and recovery | Design requirements only | Fault-injection and recovery tests |
| Performance | No baseline | Published hardware, corpus, versions and timings |
| Accessibility and localization | Requirements only | Keyboard, assistive-technology and language reviews |
| Platforms and distribution | No builds | Installation and workflow evidence per OS/architecture |
| Automation | Proposed | The same documented commands exercised through each client |

Readiness will be assessed separately for short everyday documents, long structured reports and review-heavy professional work. Do not publish a single percentage until the feature inventory, weighting and measurements exist.

## 2. Proposed architecture

**Proposal for D0 review, not a selected stack:** a Rust core organized into focused packages, with a replaceable desktop UI. Evaluate the toolkit against text input, layout, accessibility, clipboard and printing requirements before adopting it. A browser build is a later investigation, not a reason to weaken the desktop editor.

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

Every row is **planned**, not implemented. Read and write coverage must be tracked separately.

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

- [ ] Record architecture decisions: license approval, first target OS, language, toolkit and native format.
- [ ] Prototype caret movement, input methods, clipboard, font fallback and accessibility on the first target.
- [ ] Define document invariants, command parameters, undo boundaries and the failure model for saves.
- [ ] Create a minimal shell, headless test runner and reproducible build instructions.
- [ ] Add baseline formatting/linting, dependency review and layer checks to real CI.

**Exit:** an owner-reviewed decision record and a shell another contributor can build. A toolkit demo alone does not pass the text-input investigation. No build commands are advertised before validation.

### D1 — A reliable writing loop

- [ ] Implement paragraphs, selections, typing, deletion and basic formatting.
- [ ] Route edits through validated commands; expose a minimal headless command path for tests.
- [ ] Add undo/redo, explicit save/discard/cancel choices and versioned local save/reopen.
- [ ] Handle failed writes, malformed files and keyboard-only operation.
- [ ] Run all D01–D09 scenarios in [REQUIREMENTS.md](docs/REQUIREMENTS.md).

**Exit:** create → edit → save → reopen → continue preserves supported content, and every initial acceptance scenario has evidence. This is the first implementation milestone, not a DOCX milestone.

### D2 — Everyday document structure

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

Turn checklist items into small issues with a user outcome, dependencies, fixture, acceptance test and risk. Prioritize data loss, unsafe input handling and broken core workflows before feature breadth. Do not invent effort estimates from another project's pace; calibrate ranges from completed BreakFree work.

## 7. Specification inputs and revision log

Use public specifications and original test cases, not copied application code. Useful starting points: [ECMA-376 / OOXML](https://ecma-international.org/publications-and-standards/standards/ecma-376/) and, if the Rust proposal is adopted, [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html). Pin editions and relevant clauses in implementation decisions.

| Date | Change | Implementation impact |
| :--- | :--- | :--- |
| 2026-10-16 | Expanded the independent roadmap, quality gates and app identity | Documentation only; no feature completion claimed |
