# BreakFree Docs — Roadmap

**Status: planning.** Nothing below is a delivered feature. There are no promised dates; acceptance criteria determine completion.

## 0 · Technical foundation

- [ ] Agree the software license with the owner.
- [ ] Select the first target platform, UI toolkit and implementation language.
- [ ] Define a document model, local file format and supported Unicode behavior.
- [ ] Add a minimal application shell, reproducible build instructions and test runner.

**Exit:** the shell builds on the declared target and architecture decisions are written down.

## 1 · A dependable small editor

- [ ] New document, paragraphs, selection and text editing.
- [ ] Basic emphasis and paragraph alignment.
- [ ] Keyboard shortcuts, undo and redo.
- [ ] Local save/reopen and unsaved-change prompts.
- [ ] Versioned persistence with tests for interrupted and failed saves.

**Exit:** the workflow in [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md) passes without losing supported content.

## 2 · Everyday document structure

- [ ] Styles and lists.
- [ ] Tables and images.
- [ ] Page settings and pagination investigation.
- [ ] Search and replace, with Unicode and undo tests.
- [ ] Accessibility and keyboard-navigation review.

**Exit:** every newly supported element survives an edit/save/reopen round trip; regressions have tests.

## 3 · Interoperability and output

- [ ] Implement a documented subset of DOCX import/export.
- [ ] Create redistributable fixtures and a supported/unsupported feature matrix.
- [ ] Warn about unsupported or potentially lossy operations.
- [ ] Investigate printing and PDF export with visual comparison tests.

**Exit:** compatibility statements describe tested behavior rather than implying complete Word parity. Native persistence remains independently tested.

## 4 · First public preview

- [ ] Package verified builds for the declared platform.
- [ ] Publish known limitations, recovery instructions and release notes.
- [ ] Audit dependencies and asset licenses.
- [ ] Establish a private security-reporting channel before binaries.
- [ ] Add further platforms only after testing them.

Full Word parity, macros, cloud collaboration and enterprise deployment are outside the first release scope.

[Back to Docs](README.md) · [Suite roadmap](https://github.com/PerchAnton/breakfree-office/blob/main/ROADMAP.md)
