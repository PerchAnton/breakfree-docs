# Working in BreakFree Docs

Read README.md, ROADMAP.md, docs/REQUIREMENTS.md, CONTRIBUTING.md, existing ADR and
docs/PROJECT_LOG.md before making changes. The repository now includes a reversible
Python/Qt editor prototype. ADR 0001 is proposed for review; the Rust-first
long-term proposal and license decisions remain open.

## Scope

Build a reliable writing and local-persistence loop before advanced document
compatibility. Follow D0–D8 and preserve D01–D09. The referenced shared development
standard is absent at the inspected Office revision; do not invent its contents.
Use local evidence reporting and the pinned `contracts/v1/` snapshot until it exists.

## Rules

- Preserve BreakFree Docs and its approved app-specific icon at `docs/brand/breakfree-docs.webp`. Do not replace it with the suite icon or a musical symbol.
- Keep public documentation in English and distinguish proposed, implemented and verified work.
- Obtain owner review for architecture, license changes and major dependencies.
- Write original project code and documentation. Do not copy another application's source or branding. Retain all legally required dependency and asset notices.
- Keep the document model, command engine and layout independent of the UI toolkit.
- Do not execute imported macros/scripts or silently upload documents.
- Do not discard unsupported content silently or overwrite a valid file after a failed save.
- Test Unicode, editing, undo/redo, layout and persistence separately.
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

## Validation

Run the documented unit/Qt tests and Ruff checks for relevant implementation changes.
Use `tools/check_repository.py` for local links, snapshot and brand identity; check
changed YAML separately. Build a wheel and run `tools/check_wheel.py` for packaging
changes. No sibling checkout or Office service may be required. Record results and
limitations separately in `docs/VALIDATION.md` and the project log.
