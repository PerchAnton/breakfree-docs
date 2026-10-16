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
- Add CLI and optional automation through the same validated command interface, with explicit file and consent boundaries.
- Never invent build commands, platform support, test results, dates or DOCX compatibility percentages.
- Update milestone status only when evidence exists; do not check implementation tasks off for documentation changes.
- Keep `docs/PROJECT_LOG.md` in this repository; record work, checks, comments,
  reviews, help requests/offers and next actions with Europe/Kyiv timestamps and
  source links. Update an event with the same link instead of duplicating it.
  Follow the mandatory two-copy publication rule below.
- Fetch safely and preserve local/remote work before log updates. Commit log changes
  separately on the review branch. Never force push, auto-merge, release, or send
  public replies/assignments without separate owner instruction.
- After the initial audit, inspect changes since the prior recorded check, including
  replies to old issues/PRs. Distinguish local tests, server CI, merge and release.

## Журнал и приватное зеркало

The owner requires **two complete copies** after every significant stage and every
journal update. This rule replaces the former instruction not to duplicate the
project journal in gitwork. Publishing only the primary journal is incomplete.

- Primary repository: <https://github.com/PerchAnton/breakfree-docs>.
  Primary path: `docs/PROJECT_LOG.md`. Publish through the project's review
  branch/PR workflow, with journal updates in separate commits.
- Private repository: <https://github.com/PerchAnton/gitwork>, branch `main`.
  Exact mirror path: `breakfree/breakfree-docs/PROJECT_LOG.md`.
  This must be a full, text-identical copy of the published primary journal.
- Before writing, read the current
  [project prompt](https://github.com/PerchAnton/gitwork/blob/main/prompts/breakfree-docs.md),
  applicable repository instructions and both existing journals. Safely fetch/read
  current remote revisions and preserve other chats' and contributors' records.
  If the mirror has missing history, reconcile it into the primary journal first,
  publish that source, then copy its complete text. Do not publish private
  correspondence or secrets while reconciling history.
- Keep completed work, actual checks and limitations, comments/reviews with
  `@authors`, human/bot classification and direct links, help offers/requests and
  concrete next actions. Record Europe/Kyiv timestamps and the inspected period;
  include new replies on older issues/PRs. Preserve history and avoid duplicate
  events with the same link. An offer is not an assignment; stars/forks are not offers.
- After publishing the primary journal, read fresh `gitwork/main` and update only
  this project's mirror, `breakfree/breakfree-docs/SYNC.md` and its row in the
  Breakfree table of `README.md`. The row includes status, mirror/source links,
  check time and next step. Preserve all other projects and unrelated files.
- Keep mirror metadata out of the copied journal. `SYNC.md` records the source
  repository, source branch/path, full published source commit SHA, immutable
  source-file link and synchronization time with the Europe/Kyiv timezone.
  Commit the mirror/metadata update separately in `gitwork/main`.
- Never force push. On a concurrent update, read the new head, preserve both
  authors' records and retry a normal fast-forward update. Verify both files by
  reading them back from GitHub and comparing their complete text. Do not claim
  synchronization from a successful primary push alone.
- If the private repository or write is unavailable, retain a ready local copy
  and explicitly report: **«приватная копия не отправлена»**. Otherwise provide
  links to both published journals and commits in both repositories.

## Validation

Run `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`,
`cargo test --locked` and `cargo test --locked --no-default-features`. Use
`python3 tools/check_repository.py` for documentation, contract and brand checks;
Python is a developer check only, not an editor runtime. Build the native binary
with `cargo build --locked --release`. No sibling checkout or Office service may
be required. Record actual platform, native-input, screenshot and DPI evidence in
docs/VALIDATION.md, docs/UI_SPEC.md and both project journals. Synthetic egui DPI
tests do not establish Windows 100%/125%/150% behavior. Never merge or release
without separate owner instruction.
