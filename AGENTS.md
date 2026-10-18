# Working in BreakFree Docs

Read README.md, ROADMAP.md, docs/REQUIREMENTS.md, CONTRIBUTING.md, existing ADR and
docs/PROJECT_LOG.md before making changes. The owner has selected Rust for the
model, commands, persistence, integration and native GUI. Do not request language
approval again. ADR 0002 supersedes the archived Python experiment; GUI-library,
platform and license reviews remain open.

## Scope

Build a reliable writing and local-persistence loop before advanced document
compatibility. Follow D0–D8 and preserve D01–D09. The referenced shared development
standard is absent at the inspected Office revision; do not invent its contents.
Use local evidence reporting and the pinned `contracts/v1/` snapshot until it exists.

## Rules

- Preserve BreakFree Docs and its approved app-specific icon at `docs/brand/breakfree-docs.webp`. Do not replace it with the suite icon or a musical symbol.
- Keep public documentation in English and distinguish proposed, implemented and verified work.
- Rust is mandatory. Do not implement the editor in JavaScript/TypeScript, Electron
  or an HTML interface hosted in a WebView. Keep the GUI separate from the engine.
- Follow desktop Word 2024 for Windows as the approved visual reference; implement
  a custom office ribbon and components. Preserve BreakFree identity. Follow
  docs/UI_SPEC.md; disable unfinished commands with explanations and show real
  model statistics. Do not imply pagination or visual equivalence without evidence.
- Obtain owner review for remaining architecture, license changes and major dependencies.
  Rust selection is already approved.
- Write original project code and documentation. Do not copy another application's source or branding. Retain all legally required dependency and asset notices.
- Keep the document model, command engine and layout policy independent of the UI toolkit.
  Keep toolkit-specific text rendering/hit testing in a replaceable layout adapter.
- Do not execute imported macros/scripts or silently upload documents.
- Do not discard unsupported content silently or overwrite a valid file after a failed save.
- Test Unicode, editing, undo/redo, layout and persistence separately.
- Preserve semantic style inheritance and direct overrides through shared commands,
  layout and storage; follow ADR 0006, ADR 0007, ADR 0008, ADR 0009, ADR 0010, ADR 0011, ADR 0012 and ADR 0013. Do not flatten styles or infer inheritance
  from legacy resolved formatting.
- Add CLI and optional automation through the same validated command interface, with explicit file and consent boundaries.
- Never invent build commands, platform support, test results, dates or DOCX compatibility percentages.
- Update milestone status only when evidence exists; do not check implementation tasks off for documentation changes.
- Keep `docs/PROJECT_LOG.md` in this repository; record work, checks, comments,
  reviews, help requests/offers and next actions with Europe/Kyiv timestamps and
  source links. Update an event with the same link instead of duplicating it.
- Fetch safely and preserve local/remote work before log updates. Commit log changes
  separately on the review branch. Never force push, auto-merge, release, or send
  public replies/assignments without separate owner instruction.
- After the initial audit, inspect changes since the prior recorded check, including
  replies to old issues/PRs. Distinguish local tests, server CI, merge and release.

## Versioning and owner approval

Follow `docs/VERSIONING.md`. The first release cycle uses `0.1.0-alpha.N`,
`0.1.0-beta.N`, `0.1.0-rc.N`, then the first stable `0.1.0`. Only after that stable
release is actually published may development move to the `0.2.0` cycle.
`Cargo.toml` is the application-version source; keep Cargo.lock consistent.
Application, process-protocol and document-content versions are independent.

End **every work report**, including documentation/journal-only work, with the
current checked-in application version for the owner's approval. State whether
approval is pending or explicitly received; never infer approval from silence.
Every delivered change set must increment the current prerelease counter and add
a dated CHANGELOG.md entry, including code, design, documentation and journal-only
work. One coherent work iteration uses one new version; its implementation,
validation and separately committed journal share that version. A later change
set requires another increment. Read-only checks do not increment the version.
Obtain explicit owner approval for stage transitions and release numbers before
tagging/publishing. A version approval is not permission to merge, tag or publish.
Preserve historical version records and immutable tags/artifacts.

## Validation

Run `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
`cargo test --locked` and `cargo test --locked --no-default-features`. Use
`python3 tools/check_repository.py` for documentation, contract and brand checks;
Python is a developer check only, not an editor runtime. Build the native binary
with `cargo build --locked --release`. No sibling checkout or Office service may
be required. Record actual platform, native-input, screenshot and DPI evidence in
docs/VALIDATION.md, docs/UI_SPEC.md and the project journal. Synthetic egui DPI
tests do not establish Windows 100%/125%/150% behavior. Never merge or release
without separate owner instruction.
