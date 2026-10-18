# Changelog

Only published entries are releases. This file records candidate scope; version
and publication approval follow [docs/VERSIONING.md](docs/VERSIONING.md).

## 0.1.0-alpha.50 — 2026-10-18

- Add Reset indents to style to the horizontal ruler context menu. Ctrl+Shift+F8, then Delete opens the same reset preview; Enter applies, Escape cancels. Tab also commits before moving to the next marker, consistent with ruler editing.
- Remove direct left, right and first-line/hanging overrides from the selected paragraphs in one Undo/Redo operation. Each paragraph inherits its own style chain; spacing, alignment, text and style references remain unchanged.
- Preserve half-open selections, no-op redo history, stale-operation protection, the two-axis ruler design, palette/icons and native v13. Add core and actual-widget reset/history/persistence checks. Alpha.49 approved; alpha.50 awaits approval. Windows remains deferred.

## 0.1.0-alpha.49 — 2026-10-18

- Refine the horizontal ruler to a 24-DIP neutral scale with compact indent markers, centered inch labels, quarter/eighth-inch ticks and shaded margins. Keep it visible during vertical scrolling.
- Add a 20-DIP vertical ruler in the workspace gutter, aligned with the continuous canvas and its text origin. Both axes follow zoom and the shared visibility toggle. Vertical margins remain read-only; this is not pagination.
- Replace the permanent two-line readout with a contextual keyboard/drag overlay; retain exact range tooltips and mixed-origin indicators. Preserve pointer/keyboard transactions, palette/icons and native v13.
- Add zoom/DPI alignment, visibility and long-document scroll regression checks. Windows and matched Word 2024 comparison remain deferred. Alpha.49 awaits owner approval; no release or merge.

## 0.1.0-alpha.48 — 2026-10-18

- Show Mixed independently for differing resolved left/right and first-line/hanging amounts across selected paragraphs. Matching inherited/direct values remain common; an exclusive selection end at the next paragraph start excludes that paragraph.
- Mark differing ruler origins with a small dash and expose exact minimum/maximum values and selection scope in tooltips/accessibility labels. Keep first-paragraph marker anchors and the existing transactional editing policy.
- Recompute indicators from keyboard/pointer previews, cancellation and undo/redo without mutating the document. Preserve palette/icons, native v13, protocol/settings v1 and dependencies. Windows remains deferred. Alpha.47 approved; alpha.48 awaits approval.

## 0.1.0-alpha.47 — 2026-10-18

- Add keyboard ruler control: Ctrl+Shift+F8 enters/reveals the ruler, Tab/Shift+Tab selects markers, arrows preview 0.5 pt steps and Shift+arrows 5 pt. Enter or changing marker commits one Undo/Redo; Escape/F6, focus/geometry loss or pointer takeover cancels the pending preview.
- Show resolved left/right, first-line and hanging amounts in exact points, with a focus outline and keyboard hints. Keep selected ranges, bounds, inheritance and redo on no-op; prevent toolkit arrow navigation from stealing marker focus.
- Preserve palette/icons, native v13, protocol/settings v1 and dependencies. Windows/reference acceptance remains deferred. Alpha.46 approved; alpha.47 awaits approval.

## 0.1.0-alpha.46 — 2026-10-18

- Add prototype horizontal ruler handles for first/continuation lines and left/right indents. Drag previews through the shared validated geometry command; release commits one Undo/Redo, Escape or focus/geometry loss cancels without changing history.
- Keep selection, style inheritance, direct formatting and redo on no-op/cancellation. Clamp to existing limits, use half-point steps and share narrow-column scaling with text layout. Reject stale drafts; block editing/shortcuts during drag.
- Preserve palette/icons, native v13, protocol/settings v1 and dependencies. Native Windows and matched reference acceptance remain deferred. Alpha.45 approved; alpha.46 awaits approval.

## 0.1.0-alpha.45 — 2026-10-18

- Fix caret hit testing at soft-wrap ends with justified trailing spaces: logical row width now includes stored whitespace after visible ink, preventing the cursor from jumping before the final space.
- Add 108 indent layout cases across three widths/scales, both special-indent signs/zero and four alignments, with long Unicode/mixed-format text, empty/trailing paragraphs, mesh/height and cursor invariants. Cover projected input, zoom/selection, edit/history/save/reopen.
- Fix compact style creation overflow observed in alpha.44 Windows CI. Move redundant creation guidance to the existing checkbox tooltip on compact windows and reduce preview padding; add taller-font metric checks at three scales. Native Windows acceptance remains deferred.
- Add a native indent-layout capture scenario; preserve palette/icons, native v13, protocol/settings v1 and dependencies. Alpha.44 approved; alpha.45 awaits approval.

## 0.1.0-alpha.44 — 2026-10-18

- Add inherited first-line/hanging indents, explicit direct zero and reset in one Undo/Redo with paragraph geometry. Paragraph/style dialogs expose type and amount; source inspection reports the supplying style/override.
- Shape the first row and continuation at their own widths, preserving text, mixed runs, cursor indices and paragraph delimiters. Keep narrow-view scaling and compact dialog bounds.
- Write native v13; read v1–v13. Older files inherit no special indent; older schemas reject the new key even when null. Alpha.43 and earlier reject v13 saves; use Save As to retain an older copy.
- Preserve palette/icons and dependencies. Alpha.43 approved; alpha.44 awaits approval. Native Windows acceptance remains deferred; ruler handles and pagination are not implemented.

## 0.1.0-alpha.43 — 2026-10-18

- Add left/right paragraph indents (0–144 pt, half-point steps) inherited independently from paragraph styles. Direct zero overrides inheritance; partial edits and reset use one Undo/Redo with spacing.
- Extend paragraph/style dialogs and formatting source inspection. Wrap and align inside the indented column; proportionally reduce visual indents on narrow viewports to retain a 32 DIP text column, without changing saved values.
- Keep compact style dialogs within the minimum window after adding fields. Preserve palette/icons and dependencies.
- Write native v12; read v1–v12. Older files inherit zero indents and older schemas reject the new keys. Alpha.42 and earlier reject v12 saves; use Save As to retain an older copy.
- Alpha.42 approved; alpha.43 awaits approval. Windows acceptance remains deferred. First-line/hanging indents and interactive ruler handles are future increments.

## 0.1.0-alpha.42 — 2026-10-18

- Add paragraph space before (0–200 pt, half-point increments) to style inheritance and direct formatting. Explicit zero overrides inherited spacing; partial edits and reset share one Undo/Redo.
- Extend paragraph/style dialogs and formatting provenance. Apply spacing once before each paragraph, including the first; add it to the previous paragraph's space after without repeating it on wrapped lines.
- Write native v11, read v1–v11; older files default to zero before. Reject the new property in older schemas. Alpha.41 and earlier reject v11 saves; use Save As to retain an older copy.
- Preserve palette/icons and dependencies. Alpha.41 approved; alpha.42 awaits approval. Native Windows acceptance remains deferred.

## 0.1.0-alpha.41 — 2026-10-18

- Add direct paragraph line spacing and space-after overrides, inherited when unset, with validated partial updates and one Undo/Redo. Preserve spacing through Enter, joins, style changes and Replace All.
- Enable the existing Home spacing icon and H → PS KeyTip. The modal changes only checked fields, supports Use style spacing and Cancel, and guards background commands. Formatting inspection reports direct spacing sources; layout uses the same resolved values.
- Write native content v10; retain strict v1–v9 readers. Older imports inherit spacing; alpha.40 and earlier reject v10 saves. Use Save As to retain an older copy. Process/settings remain v1.
- Preserve palette/icons and dependencies. Alpha.40 approved; alpha.41 awaits approval. Native Windows acceptance deferred by the owner.

## 0.1.0-alpha.40 — 2026-10-18

- Add an 18-case synthetic scale/keyboard matrix for style dependency navigation: three window sizes, 100/125/150% scale and expanded/collapsed Home. Exercise Alt+H/F10, F3, arrows, Tab/Shift+Tab, Enter, Space and Escape without assigning menu focus in the test.
- Add a validated Unicode, 32-style native acceptance fixture, Windows manual checklist and CI review artifact with executable/source provenance. These do not establish native Windows input or display-scaling acceptance; no suitable Windows desktop was available.
- Preserve runtime behavior, palette/icons and format/protocol/settings versions. Alpha.39 approved; alpha.40 awaits approval.

## 0.1.0-alpha.39 — 2026-10-18

- Resolve pending style navigation against the current document and filters. If a return destination was deleted, renamed out of the query or is no longer used, restore the exact filters, explain the unavailable row and focus the filter instead of retaining a stale jump.
- Cancel pending navigation and its notice on explicit filter changes. Cover a 32-style reverse-order inheritance chain, repeated keyboard jumps, return, stale destinations and unchanged document/history.
- Preserve palette/icons and native/protocol/settings versions. Alpha.38 approved; alpha.39 awaits approval.

## 0.1.0-alpha.38 — 2026-10-18

- Show the original parent name in Back to: <style>. Resolve the current name from its stable ID; constrain the button width and show the full name in its tooltip when truncated.
- Preserve return/filter/history behavior, palette and icons. Alpha.37 approved; alpha.38 awaits approval.

## 0.1.0-alpha.37 — 2026-10-18

- Add Back to parent after Required by navigation. Restore the original parent focus and name/category/used-only filters, even after further jumps or filter changes, without applying formatting or changing undo history. Closing the menu discards the return point.
- Preserve palette/icons and native/protocol/settings versions. Alpha.36 approved; alpha.37 awaits approval.

## 0.1.0-alpha.36 — 2026-10-18

- Navigate from Required by names to their style rows without applying formatting. Pointer and menu keyboard activation clear name/category filters, preserve Used in document, and focus/scroll to the stable-ID destination.
- Preserve palette/icons and native/protocol/settings versions. Alpha.35 approved; alpha.36 awaits approval.

## 0.1.0-alpha.35 — 2026-10-18

- Show Required by with every directly assigned paragraph/character style that needs an ancestor, including transitive parents and styles hidden by name/category filters. Direct-plus-parent styles keep both their status and consumers.
- Preserve palette/icons, native v9 and protocol/settings v1. Alpha.34 approved; alpha.35 awaits owner approval.

## 0.1.0-alpha.34 — 2026-10-18

- Distinguish Direct use, Parent dependency and Not used beneath style names in Home menus. Direct assignment wins when a style is also a parent; filtering and undo history remain unchanged.
- Preserve the approved palette/icons, native v9 and protocol/settings v1. Alpha.33 approved; alpha.34 awaits owner approval.

## 0.1.0-alpha.33 — 2026-10-18

- Add Used in document filtering, intersected with name/category filters. Include empty-paragraph styles, text references and transitive parent dependencies.
- Exclude pending typing, next-style links and unused descendants. Default paragraph font appears for unstyled text/empty paragraphs; Reset clears the usage filter too.
- Keep filtering read-only and session-local, palette/icons and native v9 unchanged. Alpha.32 approved; alpha.33 awaits approval.

## 0.1.0-alpha.32 — 2026-10-18

- Filter the Styles menu by name and All/Paragraph/Character/Custom scope, with Reset and an empty-result message.
- Add F3 focus and Down-to-results navigation without changing document content/history. Filters remain session-local; Unicode matching uses lowercase substrings, not linguistic normalization.
- Preserve style application/context commands, palette/icons and native v9. Alpha.31 approved; alpha.32 awaits approval.

## 0.1.0-alpha.31 — 2026-10-18

- Show alignment, line spacing and space-after values and sources in Formatting.
- Distinguish paragraph-stored alignment from inherited intervals; preserve explicit zero and report mixed values/sources independently.
- Inspect the same half-open paragraph range as editing commands, including empty paragraphs and caret positions. Read-only inspection preserves history and native v9.
- Keep the palette/icons. Alpha.30 approved; alpha.31 awaits owner approval.

## 0.1.0-alpha.30 — 2026-10-18

- Add a read-only Formatting pane inspector showing resolved Bold/Italic/Underline/size and authored source: direct formatting, character style/parent, paragraph style/parent or document default.
- Report mixed values independently from mixed sources; caret inspection uses future typing. Paragraph separators do not invent character properties.
- Preserve palette/icons, history and native v9. Alpha.29 approved by the owner; alpha.30 awaits approval.

## 0.1.0-alpha.29 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.28 in the project chat.

- Choose a custom or built-in character parent, or inherit directly from the
  paragraph, in Create/Modify Character Style. Preview the resolved cascade.
- Validate missing parents and direct/indirect cycles before committing name,
  properties and parent together as one Undo/Redo. Direct overrides still win.
- Deletion remaps direct child parents to its chosen replacement; reject cycles
  without changing the document or history.
- Write native v9 parent references; read v1–v8 with parentless legacy character
  styles. Alpha.28 and earlier reject v9. Use Save As to retain older files.
- Preserve palette/icons and add cascade, graph, migration and GUI regressions.

## 0.1.0-alpha.28 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.27 in the project chat.

- Delete custom character styles with an explicit replacement: another custom
  style, Strong/Emphasis or Default paragraph font (remove the reference).
- Remap all text and pending-input references and remove the definition in one
  Undo/Redo. Preserve text, paragraph styles, selection and direct overrides.
- Add a cancellable deletion draft, Home/search/context access and X KeyTip.
  Reject built-ins, missing IDs, self-replacement and missing replacements.
- Retain native v8, palette, icons and dependencies; add engine/widget regressions.

## 0.1.0-alpha.27 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.26 in the project chat.

- Rename custom character styles in Modify Character Style. Select the current
  name on entry; validate names using the existing character catalog rules.
- Commit name and property edits together as one Undo/Redo. Preserve IDs, catalog
  order, text references, pending formatting and direct overrides. Invalid/no-op
  edits retain history; Cancel leaves the document unchanged.
- Refresh displayed names through existing Home/search actions. Native v8,
  approved palette, original icons and dependencies remain unchanged.

## 0.1.0-alpha.26 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.25 in the project chat.

- Modify custom character-style bold, italic, underline and size in a previewed
  draft. Preserve names, IDs, catalog order, run references and direct overrides.
- Commit all property changes as one Undo/Redo; Cancel, invalid values and no-op
  edits preserve history. Built-ins stay fixed. Native content remains v8.
- Add Home Styles and command-search access, J KeyTip and a context command for
  unused definitions. Keep the approved palette and original icons.
- Cover exact history, persistence, invalid/no-op definitions, actual rendering,
  dialog cancellation and minimum-window bounds with regression tests.

## 0.1.0-alpha.25 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.24 in the project chat.

- Create document-owned named character styles with optional bold, italic,
  underline and size. Unset properties inherit the paragraph; direct overrides win.
- Validate names, stable IDs and a separate 64-style character catalog. Creation
  and optional selection/pending application use one Undo/Redo transaction.
- Add a Create Character Style draft with preview, Cancel, validation, Home Styles,
  command search and K KeyTip; retain the approved palette and original icons.
- Write native v8 and read v1–v8 without flattening definitions/references. Earlier
  builds reject v8 saves; use Save As to retain an older copy.
- Add engine, migration, failure-retention, actual layout and widget regressions.
  Character-style parents, modifying/renaming/deleting these definitions and native
  Windows acceptance remain separate follow-ups.

## 0.1.0-alpha.24 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.23 in the project chat.

- Add built-in Strong and Emphasis character styles and Default paragraph font
  through Home Styles, command search and wide/narrow KeyTips.
- Resolve paragraph style → character style → direct overrides. Applying/removing
  a character style retains direct formatting; clearing direct formatting retains
  named styles. Selection and pending-caret changes support one-step Undo/Redo.
- Save semantic run references in native v7; retain strict v1–v6 readers. Earlier
  builds reject v7 saves; use Save As to retain an older file.
- Add Unicode, cascade, history, migration, failed-save and real-widget regressions
  and a synthetic character-style fixture. Keep the approved palette and icons.
- Custom character-style definitions and native Windows acceptance remain open.

## 0.1.0-alpha.23 — Unreleased; submitted for owner approval

Prepared 2026-10-18. The owner approved alpha.22 in the project chat.

- Choose a following paragraph style in Create/Modify Style, independently of
  inheritance. Enter inserts the break and applies the successor in one Undo/Redo.
- Preserve direct formatting and Unicode; paste/IME commits retain ordinary
  replacement behavior. Remap following references when deleting custom styles.
- Write native v6; retain strict v1–v5 readers with Same style migration. Older
  applications reject new saves; use Save As to retain an older document.
- Add shared-engine and actual-widget regressions, a following-style fixture and
  documentation. Retain the approved palette/icons; Windows acceptance stays open.

## 0.1.0-alpha.22 — Unreleased; version approved by owner

Prepared 2026-10-18. The owner approved alpha.21 in the project chat.

- Delete custom styles with separate choices for paragraph replacement and the
  new parent of direct child styles. Show real affected counts and warn that
  inherited appearance may change. Keep direct formatting, text and alignment.
- Remove the definition and replace all direct references in one undo/redo,
  including references from built-in definitions. Keep indirect parent links and
  surviving IDs. Reject invalid IDs, self references and cycles without mutation.
- Add a Delete Style draft through Home, command search, KeyTips and gallery/list
  context menus. Cancel has initial focus; invalid drafts disable Delete. Guard
  background editing, file actions, close requests and text composition.
- Add model/storage/widget regressions, a deletion fixture and native captures.
  Preserve the light palette, original icons, logo and Home gutters.

Content remains v5; protocol/preferences remain v1. No dependency/license change.
Character styles, following-style rules and Windows/Word/DPI/physical IME acceptance
remain open. No merge, tag or release is authorized.

## 0.1.0-alpha.21 — Unreleased; version approved 2026-10-18

Prepared 2026-10-18. The owner approved alpha.20 in the project chat.

- Rename custom paragraph styles in Modify Style, preserving stable IDs, paragraph
  and descendant references, direct formatting, selection and pending input.
  Name, parent and supported definition edits commit together in one undo/redo.
- Select the current custom name on entry; reject invalid/duplicate names before
  applying. Cancel and unchanged OK preserve history, including an existing redo.
  Built-in names stay fixed; case-only custom name changes are allowed.
- Offer Modify Style from the gallery and any Styles-list row's context menu,
  including unused styles. Refresh gallery labels, tooltips and command search.
  Keep the light palette, original icons and Home gutters.
- Add engine/storage/widget regressions for Unicode, capacity, exact history,
  unchanged layout, save/reopen, failed writes and context-menu targeting.
  Correct File's stale format-version explanation to v5.

Native content stays v5; protocol/settings stay v1. No dependency/license change.
Deletion, character styles and native Windows/Word/DPI/IME acceptance remain open.
No merge, tag or release is authorized.

## 0.1.0-alpha.20 — Unreleased; version approved 2026-10-18

Prepared 2026-10-18. The owner approved alpha.19 in the project chat.

- Create custom named paragraph styles with a built-in/custom parent or no parent,
  inherited formatting, live validation and a character preview. Optionally apply
  to the selection; creation and application are one undo/redo operation.
- Keep Unicode names and stable document-local IDs; reject duplicate names, missing
  parents, cycles and invalid values. Preserve direct overrides and selection.
- Expose creation through Styles/More, command search and KeyTips. Show custom
  styles in the bounded keyboard-scrollable menu, search and selected gallery card.
  Keep the palette, original icons and Home spacing. Fix name focus after invalid Enter.
- Write native content v5 and retain strict v1–v4 readers; opening older files
  preserves bytes. Earlier builds reject v5 saves. Use Save As for an older copy.
- Add model/storage/widget regressions for creation, optional application, parent
  inheritance, exact undo, errors, save failure, Unicode, long names and menu traversal.

No new dependency, protocol/settings or license change. Rename/delete, character
styles, templates and native Windows/Word/DPI/IME acceptance remain separate work.
No merge, tag or release is authorized.

## 0.1.0-alpha.19 — Unreleased; version approved 2026-10-18

Prepared 2026-10-18. The owner approved alpha.18 in the project chat.

- Add Modify Style for Normal, No Spacing and Heading 1: parent selection,
  inheritable bold/italic/underline, size, line spacing and space after. A validated
  draft previews character appearance; Cancel/Escape leaves the document intact.
- Apply all definition changes with one undo/redo, preserving direct formatting,
  text, alignment, selection and pending input. Reject parent cycles and invalid
  values. Descendants and Home previews resolve the document's current catalog.
- Open through Styles menus, card context menus, command search or KeyTips.
  Preserve the approved palette, original icons and responsive Home gutters.
- Save document-owned definitions in native content v4; retain strict v1/v2/v3
  readers, frozen legacy defaults, source-byte preservation and checked saves.
  Earlier builds reject v4; use Save As to keep the original older file.
- Cover graph validation, exact history, edit/batch preservation, stale previews,
  malformed catalogs, save failures and actual-widget/layout/modal behavior.

No dependency, process/settings protocol or license change. Custom/character
styles, templates, Windows/reference/DPI/IME acceptance remain separate gates.
No merge, tag or release is authorized.

## 0.1.0-alpha.18 — Unreleased; version approved 2026-10-18

Prepared 2026-10-17. The owner approved alpha.17 in the project chat.

- Activate Home's Normal, No Spacing and Heading 1 gallery, with responsive menus,
  selected/mixed state, keyboard traversal, KeyTips and command search. Preserve
  the palette, original icons and group gutters.
- Separate inherited character values from explicit overrides in the Rust model.
  Apply a paragraph style in one undo; keep direct formatting and alignment.
  Clear character formatting now restores each paragraph's inherited values.
  No Spacing changes actual line/paragraph spacing; Heading 1 supplies bold 18 pt.
- Preserve style identity through text editing, previewed batch replacement,
  undo/redo and native content v3. Read v1/v2 unchanged and preserve their appearance
  as explicit overrides; opening does not rewrite the source. Older builds reject
  v3 saves. Use Save As to retain an older file.
- Correct previously ignored paragraph gaps in the layout adapter and retain
  keyboard focus in overflow menus after resizing a temporary ribbon.
- Add inheritance, migration, save-failure and actual-widget/layout regressions.
  Document fixed catalog rules and the deferred style-definition editor.

No new dependency, process/settings schema or license change. Custom/character
styles, pagination and native Windows/Word comparison remain separate work.
Version approval does not authorize merge, tag or release.

## 0.1.0-alpha.17 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.16 in the project chat.

- Activate the original Home eraser icon as Clear character formatting in full,
  compact and overflow ribbon densities. Preserve the palette, icon artwork and
  group spacing; compact layouts prioritize this working control over disabled
  Change case.
- Reset selected bold, italic, underline and size to regular 11 pt through one
  shared-engine transaction. Preserve text and paragraph alignment. At a caret,
  reset pending input only; unchanged content adds no undo and retains redo.
- Add document-only Ctrl+Space, Home KeyTips H → FC and command search. Keep
  Control distinct from Command on macOS; ignore modified chords, repeat and
  IME confirmation, and block modal background edits.
- Add range/history/Unicode/native-roundtrip and widget regression coverage;
  retain the existing keyboard and Home containment checks.

No dependency, document/protocol/settings schema or license change. Named styles,
paragraph-format reset, native Windows/IME and matched Word comparison remain
separate gates. Version approval does not authorize merge, tag or release.

## 0.1.0-alpha.16 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.15 in the project chat. Preserve the
approved palette, original vector icons and Home spacing.

- Add Replace All from Home Replace with a modal preview of real match/change
  counts, paragraph context and replacement text. Cancel/Escape leaves content
  untouched; initial focus is Cancel. Apply commits exactly the prepared result.
- Build the validated result in the GUI-independent command engine, using original
  match ranges and each match's first-character style. Keep unaffected formatting;
  equal-text matches preserve mixed formatting and consume no history.
- Record all changes as one undo/redo operation. Reject stale previews, invalid
  ranges, unsupported control characters and oversized results before editing.
  Do not rescan inserted text or create per-match history entries.
- Block background commands/close while preview is open and guard replacement IME
  confirmation. Virtual preview rows and bounded controls adapt to window size.
- Add core and widget regressions for Unicode, mixed styles, paragraph operations,
  cancellation, keyboard Apply, stale/failed/no-op transactions and dense matches.
- Clear the completion notice after later editor actions. Validate 121 full / 51
  core-only tests, two native preview captures and macOS keyboard/Save/Open.

No dependency, document/protocol/settings schema or project-license change.
Native Windows/IME and matched Word comparison remain separate acceptance gates.
Version approval does not authorize a merge, tag or release.

## 0.1.0-alpha.15 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.14 in the project chat. Preserve
alpha.13 colors, original vector icons and the Home group spacing.

- Add Match case and Whole words controls to Navigation, with keyboard Tab/Space
  and Escape back to the document. Options persist only in the current window.
- Share a cached literal query between counts, results, highlighting and replacement.
  Ignoring case uses Unicode simple folding; whole words use Unicode word boundaries
  including internal apostrophes. Preserve original ranges and complete graphemes.
- Keep replacement on the shared command/history path. Query/options changes are
  read-only; overlong queries produce an explicit error, without truncation.
- Add the small Rust regex dependency for Unicode case tables and bounded literal
  matching, with locked versions and updated dependency notices/inventory. User
  patterns are escaped; no wildcard/regular-expression search is exposed.
- Extend core and actual-widget regressions for multilingual case/word boundaries,
  literal punctuation, keyboard focus, formatting, undo/redo, persistence and bounds.
  All 110 full / 44 core-only tests pass; two native captures and macOS keyboard,
  replacement/history and system Save/Open checks passed.

No document/protocol/settings schema or project-license change. Native Windows,
IME, matched Word comparison and full release acceptance remain open. No merge,
tag or release is authorized by version approval.

## 0.1.0-alpha.14 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. The owner approved alpha.13 in the project chat. Its palette
and original vector icons are retained; approval does not authorize publication.

- Refine the existing Navigation pane with previous/next controls, an active
  result counter, highlighted excerpts and a virtualized list of all matches.
  Remove the silent 40-row limit and scroll the document to the selected result.
- Keep search focus on Enter/Down and Shift+Enter/Up; wrap at either end. Clear
  retains search focus; Escape closes the pane and returns to the document.
- Share one GUI-independent literal matcher between the count, selection and
  replacement. Reject partial combining/emoji graphemes instead of expanding a
  match into different text. Search is explicitly case sensitive, without
  normalization, wildcards or whole-word options.
- Verify two native captures and macOS keyboard search/replace, Undo/Redo and
  system Save/Open; document the OS-reserved Command+H limitation.
- Add Unicode, dense-result, excerpt, replacement/history/native-roundtrip and
  actual-widget regressions, including nine size/scale combinations and an IME
  confirmation guard. Preserve the shared editing and persistence commands.

No dependency, schema, license, merge, tag or release changes. Native Windows
input/scaling and matched Word 2024 comparison remain acceptance gates.

## 0.1.0-alpha.13 — Unreleased; version approved 2026-10-17

Prepared 2026-10-17. This focused visual update follows the owner's supplied
macOS reference captures. Alpha.12 version approval has not been received.

- Give original vector icons small semantic accents: gold Clipboard/Open,
  blue file/copy/scissors details and arrows, yellow highlight, red font-color
  indicator and a muted pink eraser. Keep text/alignment/history outlines dark.
- Add explicit enabled/disabled tint handling; unavailable commands remain muted
  and disabled with their existing explanations. Color indicators do not add
  text-color, highlighting or Format Painter functionality.
- Unify light neutral shell/ribbon/canvas/File colors, soften violet selection,
  and add original Home/Options/Print icons to the existing File actions.
- Inspect three native captures and normal macOS input for selection, Bold,
  undo/redo, File Home/Options and KeyTips. All 95 existing tests and 36 core-only
  tests pass; no test assertions or layout dimensions were relaxed.

The supplied images inform color roles only: they do not identify a Word build
or establish Windows/DPI equivalence. No source images, external icon assets,
new dependencies, schema changes, merge, tag or release.

## 0.1.0-alpha.12 — Unreleased; submitted for owner approval

Prepared 2026-10-17. The owner approved alpha.11 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add More Commands to Quick Access Toolbar customization, with an ordered list
  and Move Up / Move Down controls. OK applies and saves the draft; Cancel/Escape
  retains the previous toolbar. Unchanged OK does not write settings.
- Keep command selection and keyboard focus after moves. Support Up/Down/Home/End,
  Alt+Up/Down, Tab/Shift+Tab and Enter/Space, with disabled boundary controls.
  Numeric KeyTips follow the applied order, including unavailable history actions.
- Reuse settings conflict/error/retry handling; preserve previous bytes on failed
  writes and keep document content, selection and undo history independent.
- Add five actual-widget regressions, including 18 size/scale/list combinations,
  and two native captures. Verify native macOS cancellation, apply, renumbered
  Undo/Redo and order restoration after a normal application restart.

No dependency, license, settings-schema, document-content or process-protocol
change. Native Windows DPI/input and matched Word 2024 acceptance remain open.
No merge, tag or release.

## 0.1.0-alpha.11 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. Alpha.10 approval has not been received. This focused design
fix follows the owner's report of a Home separator overlapping Font controls.

- Reserve each Home group's full width exactly once, keeping the 8-DIP gap even
  when its content is narrower. Center separators in the gap so they cannot cross
  Bold, the font-family outline or adjacent groups.
- Add a regression for group spacing and separator/control separation across nine
  window widths, three synthetic scales and pinned/temporary ribbon states.
- Inspect four native Home captures and verify pointer Bold, font-size entry,
  undo/redo and temporary Home through native macOS input. Stabilize focus in
  the temporary-ribbon capture fixture; production focus handling is unchanged.

No dependency, license, document-content or process-protocol change. Matched
Word 2024 Windows/native DPI acceptance remains open. No merge, tag or release.

## 0.1.0-alpha.10 — Unreleased; submitted for owner approval

Prepared 2026-10-17. The owner approved alpha.9 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Persist Quick Access Toolbar command visibility, order and above/below placement
  in a versioned local user profile, restoring them at desktop startup. Keep
  settings independent of documents, history and headless information commands.
- Add confirmed Reset with a retained original-file backup, visible load/save
  errors, a menu warning and Retry Saving. Preserve corrupt/future files and
  tested failed writes; reject stale edits from another window under a shared lock.
  A retry after failed reset retains the backup requirement, including after further
  current-window customization.
- Retain the compact native toolbar geometry, ordered numeric hints and keyboard
  menu focus. Add a keyboard-accessible reset confirmation with Escape cancellation.
- Add nine settings tests, five actual-widget regressions and a CLI no-side-effects
  test. Verify native macOS restart, failure/retry/reset/backup and Unicode editing,
  undo, Save/Open; inspect three native captures. Document schema and Windows paths.

No new dependency, license, document-content or process-protocol change. Explicit
reordering controls, live cross-window refresh, crash-lock recovery and matched
Word 2024 Windows/native DPI acceptance remain open. No merge, tag or release.

## 0.1.0-alpha.9 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.8 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add a Quick Access Toolbar menu for New/Open/Save/Undo/Redo visibility, placement
  above/below the ribbon and reset. Preferences apply to the current window;
  adding a command appends it. KeyTips follow visible order, including disabled
  history commands; Alt+0 opens customization.
- Add keyboard toolbar entry from the tab row, Left/Right navigation that skips
  unavailable actions, menu navigation and layered Escape. Keep the toolbar
  accessible when the collapsed ribbon opens temporarily.
- Rebuild View with compact Views/Show/Zoom groups, aligned checkboxes and original
  vector controls. Add 100% and a real Canvas Width fit that follows window/pane
  changes within 50–200%. Manual zoom leaves fit mode; status percentage opens Zoom.
- Preserve document selection, shared undo/save, native content v2 and protocol v1.
  Unsupported pagination/view modes remain disabled with explanations.
- Add six actual-widget regressions, including 18 toolbar/View width/scale cases,
  and four native captures. Validate the normal macOS binary through keyboard,
  system Save/Open and independently decoded Unicode/style results.

No toolbar persistence/reordering editor, pagination, Windows visual equivalence,
new dependency or license change is claimed. Alt+0 and View C are Docs-specific;
View J and exact reference mappings require Word 2024 Windows acceptance.

## 0.1.0-alpha.8 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.7 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add Quick Access Toolbar KeyTips: fixed 1 Save, 2 Undo, 3 Redo, including direct
  Alt+number entry. Disabled history actions keep dimmed hints without renumbering.
- Give every tab an entry and retained KeyTip level, including Draw JI and Help
  Y1. Add a More tabs hint/menu at narrow widths; Escape and resizing restore the
  root level. Unimplemented commands remain disabled and Help is informational.
- Add View K Navigation Pane, R Canvas Rulers and Q Zoom routes. Group the real
  View controls, expose selected toggle states, and retain arrow/group navigation
  when leaving hints, including the temporary collapsed ribbon.
- Add a native Zoom draft dialog with percent entry, presets, validation and
  OK/Cancel. Zoom changes only the view; it cannot change document history or
  dispatch background commands while the dialog is open.
- Add regression coverage for quick-access history/save, all tabs, overflow,
  View state, modal focus and control bounds at synthetic scales. Extend the
  capture fixture with tab-overflow, View and Zoom states.

Native content v2 and process protocol v1 stay unchanged. Draw/Help/ruler mappings
are provisional pending a named Word 2024 Windows comparison. Quick-access order
is fixed in this build; toolbar customization is not implemented. Native Windows
Alt/AltGr/IME/DPI and exact reference matching remain open. No merge, release,
dependency or license change is included.

## 0.1.0-alpha.7 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.6 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Add bare-Alt release and F10 entry into sequential KeyTips. Home/File/search
  access keys dispatch existing commands; hints follow real control positions.
- Add Home emphasis, size/Font dialog, alignment, clipboard and Editing routes,
  including a narrow-window More level. Preserve collapsed mode and selection.
- Filter multi-letter prefixes, support Backspace and layered Escape, restore
  previous focus, and hand arrows/Tab back to ribbon navigation. Consume paired
  text and repeated access keys so they cannot modify the document accidentally.
- Guard composition, dialogs, lost window focus and modifier chords. Treat AltGr
  Ctrl+Alt key events separately from command shortcuts while retaining text input.
- Add seven widget regressions, nine badge width/scale cases and four native
  render captures. Verify native macOS F10/Alt+H sequences, size input, menus,
  Font dialog and Save/Open with independently decoded Unicode/styles.

Native content v2 and process protocol v1 stay unchanged. Bare Alt is covered by
synthetic modifier transitions; the macOS input tool cannot send a modifier-only
press. Native Windows Alt/AltGr/IME/DPI and matched Word 2024 checks remain open.
Non-Home command KeyTips, Draw/Help and quick-access hints remain incomplete.
No merge, release, dependency or license change is included.

## 0.1.0-alpha.6 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-17. The owner approved alpha.5 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Open a collapsed ribbon temporarily from a tab or Alt+H, over the canvas without
  reflow. Commands, outside clicks and Escape dismiss it while retaining tabs-only
  mode. Ctrl+F1, double-click and the tab context menu switch the pinned mode.
- Add F10, Tab/Shift+Tab, directional navigation, Ctrl+Left/Right between Home
  groups, Enter/Space and keyboard dropdowns. Disabled commands are skipped;
  menus close one level at a time. Resize moves hidden Editing focus into More.
- Cycle visible document/panes/status/ribbon regions with F6/Shift+F6. Preserve
  native size-field editing, document selection, undo and Unicode save/reopen.
- Add six real-widget regressions and nine temporary-ribbon width/scale cases.
  Inspect three native macOS captures and exercise native keyboard/menu/history,
  F6, pinning and system Save/Open. Update measured UI behavior and remaining gates.

Native content v2 and process protocol v1 stay unchanged. Bare-Alt activation and
full sequential KeyTips remain pending, as do native Windows/Word/DPI/IME reviews.
No merge, release, dependency or license change is included.

## 0.1.0-alpha.5 — Unreleased; version approved by owner on 2026-10-17

Prepared 2026-10-16. The owner approved alpha.4 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Give Home's Font launcher its own native Font dialog, also available through
  Ctrl/Command+D and Ctrl/Command+Shift+F. Keep the Paragraph launcher separate.
- Add compact Font / Font style / Size columns, preset sizes, mixed underline,
  a clipped preview, and OK/Cancel. Unsupported font-family choice stays disabled.
- Stage changes without editing the document; OK applies changed properties in
  one shared undo transaction, preserving untouched mixed runs and Unicode ranges.
  Escape/Cancel keeps the original; invalid sizes keep the dialog open. Document
  shortcuts and window close cannot bypass the modal draft.
- Split Find into a direct navigation action and a separate options arrow; retain
  Select and overflow menus and the 104-DIP Home layout.
- Extend engine and real-widget tests for dialog keyboard/pointer behavior,
  mixed formatting, history, save/reopen and minimum-window bounds at synthetic DPI.
- Verify native macOS Font cancellation/validation/undo and Unicode Save/Open;
  inspect Home/Font at 1366×900 and 640×480 DIP. Capture helper waits for modal layout.

Native content v2 and protocol v1 stay unchanged. No release, merge, dependency or
license change is included. Native Windows/Word comparison remains unverified.

## 0.1.0-alpha.4 — Unreleased; version approved by owner on 2026-10-16

Prepared 2026-10-16. The owner approved alpha.3 in the project chat; that approval
covers its version number, not a merge, tag or release.

- Tighten Home to 104 DIP: compact Clipboard with split Paste, closer Font/Paragraph
  rows, joined size field, original dropdown/vector icons and inset group launchers.
- Replace separate AaBbCc tiles with a single gallery of inline style-name previews;
  styles remain disabled and no applied style is fabricated.
- Add working Grow/Shrink Font buttons and Ctrl/Command+Shift+>/< shortcuts. Each
  selected run steps independently through presets in one shared undo transaction;
  pending caret style, Unicode boundaries and unrelated formatting are preserved.
- Add real Find/Select menus; keep commands reachable via overflow at 640 DIP.
  Extend control/menu geometry and pointer/keyboard regressions.
- Inspect native macOS captures at 1366, 960 and 640 DIP. Verify a native macOS
  keyboard/Unicode-paste → format → undo/redo → system Save/Open loop plus dirty
  New/Cancel/Discard. Automated non-ASCII character typing remains unresolved;
  native IME and Windows/Word comparison are not certified.

Native content v2 and protocol v1 stay unchanged. No release, merge, dependency or
license change is included.

## 0.1.0-alpha.3 — Unreleased; version approved by owner on 2026-10-16

Prepared 2026-10-16. No release or tag is published by this entry.

- Add real Home font-size input/presets (1–400 pt, half-point steps), underline,
  Ctrl/Command+U and Ctrl/Command+Shift+P; show uniform/mixed selection values.
- Keep selection, pending insertion style and exact undo/redo in shared Rust
  commands. Enter/blur applies size, Escape cancels, invalid input preserves the
  document; Save/close resolves pending size before acting on the model.
- Render actual run sizes and underline in the hit-testable text projection;
  document zoom does not modify stored sizes.
- Write native content v2, with strict read-only v1 conversion to 11-pt/no-underline
  defaults. **Older v1-only builds cannot open v2 saves.** No lossy downgrade;
  process protocol remains v1. Add a formatted Unicode v2 fixture and ADR 0003.
- Extend engine/storage/widget regression coverage for Unicode, formatting,
  history, malformed v2 files and failed writes preserving previous bytes.

Native Windows keyboard/DPI/IME/accessibility and a matched Word 2024 comparison
remain unverified. Font-family choice, named styles and pagination remain out of scope.

## 0.1.0-alpha.2 — Unreleased; submitted for owner approval

Prepared 2026-10-16. No release or tag is published by this entry.

- Require a version increment and dated changelog entry for every delivered change
  set, including documentation and journal work, under the updated owner policy.
- Define one coherent work iteration as the increment unit; its validation and
  separate journal commit share that version. Later changes increment it again.
- Advance Cargo package/lock metadata to `0.1.0-alpha.2`; CLI, capabilities and GUI
  Help obtain the version from the same build metadata. Editor behavior, native
  content v1 and process protocol v1 are unchanged.
- Preserve separate owner approval for stage transitions, merge, tags and releases.

## 0.1.0-alpha.1 — Unreleased; submitted for owner approval

Prepared 2026-10-16. No release or tag is published by this entry.

- Standalone Rust document model, shared edit commands, undo/redo and native GUI.
- Custom Home ribbon, File, navigation/formatting panes and real document statistics.
- Refined light desktop shell: original vector controls, compact Font/Paragraph
  rows, style previews, File Home/Options, status zoom buttons and responsive
  bounds tests; native wide/narrow/File captures.
- Versioned `.bfdoc` content v1, checked temporary-file replacement, file errors and
  Save/Discard/Cancel; process integration v1 and independent executable.
- English/Ukrainian/Russian and grapheme regression tests, CPU widget event tests,
  reproducible builds and platform evidence tracked in the validation report.
- Adopt the owner's pre-0.1.0 alpha/beta/RC cycle; supersede the earlier unreleased
  `0.2.0-dev.1` build label. No document-format or process-protocol version change.

Native Windows input/DPI, matched Word 2024 comparison, full shaping/accessibility,
pagination, installers and license adoption remain open. DOCX/PDF/printing and
cloud features are not implemented. See [validation](docs/VALIDATION.md) and
[roadmap](ROADMAP.md) for detailed evidence and gates.
