# Changelog

Only published entries are releases. This file records candidate scope; version
and publication approval follow [docs/VERSIONING.md](docs/VERSIONING.md).

## 0.1.0-alpha.93 — 2026-10-24

- Add primary DOCX read/write for fixed-grid plain-cell tables: exact column
  widths, rows/cells, empty cells, Unicode, line breaks and a repeating first row.
- Preserve table blocks through shared editing/undo/redo and Save/reopen; reset
  sequential list streams across tables. Validate full-model readback before writes.
- Define an explicit plain-cell layout profile; reject merged/nested/rich cells,
  unknown styles, mismatched widths and unrepresented flow/presentation settings.
- Add independent input markup, native/DOCX fixture pairs and rejection/preservation
  tests. Preserve existing design, palette/icons and optional native v19 format.
- Windows and paper printing remain deferred; VoiceOver remains last.

## 0.1.0-alpha.92 — 2026-10-24

- Extend primary DOCX read/write with HTTP, HTTPS and mailto hyperlink ranges,
  including mixed-format runs, Unicode, adjacent links and links inside lists.
- Resolve referenced external hyperlink relationships without fetching or opening
  destinations; preserve shared targets and reject missing/duplicate references.
- Keep unsupported anchors, fields, tooltips, history flags and target-frame
  metadata explicit errors; reject invalid/grapheme-splitting or empty ranges.
- Add independent import markup, native/DOCX fixtures and shared editing/history
  checks. Preserve design, palette, icons and the optional native format.
- Windows and paper printing remain deferred; VoiceOver remains last.

## 0.1.0-alpha.91 — 2026-10-24

- Make DOCX the primary/default format. Save updates an opened DOCX; Save As
  retains optional `.bfdoc` support. Reject unsupported content without silent
  conversion and detect source changes before overwriting a loaded DOCX.
- Add decimal/bullet list read/write, levels and restarts through real numbering
  definitions; retain shared editing/undo and reject unrepresentable sequences.
- Expand import to relationship-located parts, optional styles/missing unused
  built-ins and supported document defaults represented as an inherited root style.
- Validate each encoded DOCX by exact model readback before writing; add independent
  input markup, list fixture pairs and preservation/history/conflict tests.
- Preserve design/palette/icons. `.doc`/`.docm`, broader DOCX objects/fonts and public
  release remain unsupported or pending. Windows deferred; VoiceOver last.

## 0.1.0-alpha.90 — 2026-10-24

- Begin D4 with a bounded ZIP/XML DOCX adapter and an explicit restricted profile:
  paragraphs/runs, supported formatting, named style graphs, spacing/indents,
  keep rules and single-section page dimensions/margins.
- Add validated File/CLI import and explicit File/command-search export. Imported
  documents save as native `.bfdoc`; export preserves session history/destination.
- Reject unsupported parts/properties, external relationships, macro types,
  ambiguous style-toggle combinations and inexact measurements before data loss.
- Add Unicode/native/DOCX round trips, hostile package/XML and failed-write tests,
  reproducible fixture pair, profile documentation and external open evidence.
- Preserve palette/icons; D4 remains incremental, with objects, broader input
  profiles and package publication still open. Windows deferred; VoiceOver last.

## 0.1.0-alpha.89 — 2026-10-24

- Add shared-model lists, nested levels/restarts, hyperlinks, rectangular table
  editing, sized PNG blocks and heading navigation, with transactional undo/redo.
- Add native v19 structure/page settings and strict legacy gates; retain old-file
  reading, Unicode boundaries, Replace All metadata and failed-save preservation.
- Add page sizes/margins, new-page sections, headers/footers/numbering, caret page
  breaks, keep rules, repeated table headers, editable Print Layout and preview.
- Export the shared layout as 144 dpi raster PDF; expose explicit macOS/Linux CUPS
  queue submission. Paper printing is deferred; no print job was sent.
- Add Insert/Layout commands and KeyTips, output shortcuts, structured-report
  fixtures and regression tests. Fix missing export glyphs and keep-chain placement.
- Preserve palette/icons. Windows deferred; VoiceOver last. No release, DOCX,
  searchable PDF, merged cells, floating objects or general RTL/CJK claim.

## 0.1.0-alpha.88 — 2026-10-24

- Correct Paste proportions from the supplied reference: a broader clipboard,
  inset white paper and a larger front sheet positioned higher; retain hit targets.
- Complete ten tabulation-provenance increments: independent interval/list sources,
  grouped mixed selections and absent marker owners, Formatting rows, inspection
  from both paragraph dialogs, live style-draft descriptions, marker hover/AX
  sources, context-menu inspection and keyboard access with focus restoration.
- Open complete read-only details with Ctrl+Alt+F9, or F9/Sources in a tab preview.
  Scroll with arrows, Page Up/Down, Home/End or the wheel. Preserve numeric input,
  caret, preview, selection and history while inspecting or dismissing details.
- Keep dialog-entry snapshots explicit; ruler inspection describes the original
  document. Preserve existing dialog keyboard order and block background edits.
- Fix a native accessibility crash when dismissing source details: restore focus
  only once the destination exists again. Keep moved-marker source identity and
  preview leader counts correct; return context-menu inspection to the editor.
- Retain native v18, protocol/settings v1, palette and other icons. Alpha.88 awaits
  approval; Windows remains deferred and VoiceOver stays at final review.

## 0.1.0-alpha.87 — 2026-10-24

- Refine original Clipboard/Paste, Cut, Copy and Save vectors from the supplied
  reference: raised clasp and plain front sheet, narrower scissors, folded copy
  corner and a compact disk outline.
- Use finer neutral outlines, a quieter gold clipboard edge and smaller blue
  scissor handles. Reduce the Paste drawing from 28 to 26 DIP, retaining hit targets,
  disabled states, command routing and the existing ribbon layout.
- Add a selected-document capture mode and inspect active/disabled icons in wide
  and compact native windows. Preserve native v18 and all editing behavior.
- Alpha.87 awaits approval; alpha.85 is the latest explicitly approved version.
  Windows remains deferred and VoiceOver stays at final review.

## 0.1.0-alpha.86 — 2026-10-24

- Inherit tab intervals and complete typed stop/leader lists from paragraph styles,
  with nearest-parent resolution, direct override precedence and explicit empty lists.
- Edit tabs inside Create/Modify Style drafts and commit the entire definition in
  one undo/redo operation; nested cancellation returns focus without editing.
- Resolve inherited positions in layout, dialogs and ruler. Materialize direct
  lists only for changed owners; reset paragraph lists/intervals independently.
- Write native v18 and retain strict v1–v17 reading and failed-write preservation.
  Older applications reject v18; Save As retains an older compatible source.
- Add inheritance, history, storage, ruler and GUI regressions; retain the design,
  palette and icons. Windows remains deferred and VoiceOver stays at final review.

## 0.1.0-alpha.85 — 2026-10-23

- Preserve the numeric draft, caret and selection when returning from preview
  controls with F6 or refocusing the same position field with F2, including errors.
- Release stale button focus when returning to marker mode via F4 or the position
  button. A pointer press in the numeric field takes focus immediately.
- Ignore held F6 repeats. Opening a tab preview no longer requests background
  document focus; cancellation restores the original document selection and history.
- Consume the outside dismissing press as well as raw events, retaining focus and
  preventing its release from activating background commands. Defer document focus
  restoration while the window is inactive, then restore it when focus returns.
- Preserve the actual replacement range for text input and paste when old/new
  text share a prefix or suffix. Undo then restores the whole selection before
  another preview, including Unicode and combining characters.
- Add mode-switch/cancellation/re-entry regressions and native capture sequences.
  Preserve the alpha.84 design, palette and icons; native v17 remains unchanged.
- Alpha.84 explicitly approved by the owner; alpha.85 awaits approval. Windows is
  deferred and VoiceOver stays at final review. No merge, tag or release authorized.

## 0.1.0-alpha.84 — 2026-10-23

- Quiet the editor chrome: remove redundant shortcut placeholders, search guidance
  and the style-menu precedence footer. Keep hover help and disabled explanations.
- Start tab previews with help collapsed. Show the position and concise real leader
  coverage; F1 expands the distribution, and Help/F7 reveals mode or control help.
  Choosing a leader no longer opens the detailed breakdown automatically.
- Keep Help beside preview actions without taking keyboard focus or the numeric
  caret. Retain error messages, cancellation and the single Undo/Redo boundary.
- Refine original vector icons from the supplied video: neutral Save/Copy/paper
  outlines, gold clipboard edge, blue list/sort details, stronger yellow/red color
  indicators, dotted borders and a recognizable shading bucket.
- Use a white indent readout and remove its repeated shortcut sentence. Preserve
  the neutral shell, violet brand selection and approved logo. Native v17 unchanged.
- Reserve field height from actual font metrics so the font-size input cannot
  overlap Underline with taller system fonts, as reported by Windows CI for alpha.83.
  Let the style list use available window height and omit empty category headings.
- Alpha.78–83 have no explicit version approval recorded; alpha.84 awaits approval.
  Windows remains deferred; VoiceOver stays at final review.

## 0.1.0-alpha.83 — 2026-10-23

- Refine the native shell from the owner's Word macOS video: neutral flat chrome,
  clearer semantic icon accents, white fields/popups and quieter hover/pressed fills.
  Command-menu labels align to the leading edge.
- Reduce title/tabs/ribbon height from 180 to 158 DIP; preserve Home group gutters,
  responsive overflow, real commands, disabled explanations and keyboard targets.
- Give style cards a formatted sample, separate name strip and restrained selected
  outline. Long names stay clipped to the card with the full tooltip available.
- Use installed Arial for macOS interface text; document font resolution is unchanged.
  Windows keeps installed Segoe UI and other hosts retain the embedded fallback.
- Refit View rows/captions and large icons to the shorter ribbon. Extend the capture
  harness to 2560 × 1440 DIP for larger-window visual checks.
- Preserve the approved logo, violet brand accent, data/history and native v17.
  Alpha.78–82 have no explicit version approval recorded; alpha.83 awaits approval.
  The supplied video is a macOS reference, not matched Windows validation.
  Windows remains deferred; VoiceOver stays at final review.

## 0.1.0-alpha.82 — 2026-10-23

- Toggle focused preview help with F7 without moving focus; ignore held-key repeats.
- Add a Show/Hide help button that preserves the current control focus.
- Remember visibility across previews in the same editor window, without changing
  document data or writing preferences; new windows start with help enabled.
- Preserve the numeric caret/selection when toggling help, including invalid input.
- Expose help visibility as a labeled toggle while keeping hover descriptions and
  full action/disabled-state accessibility labels available when help is hidden.
- Keep palette/icons, native v17, cancellation and one-operation history unchanged.
  Alpha.78–81 have no explicit version approval recorded; alpha.82 awaits approval.
  Windows deferred; VoiceOver acceptance remains at final review.

## 0.1.0-alpha.81 — 2026-10-23

- Show focused-button descriptions in the ruler preview without applying edits.
- Explain mode-specific shortcuts, including numeric input versus leader digits.
- Share the same action descriptions with pointer hover help.
- Explain disabled Restore, invalid-position and deletion states.
- Expand button accessibility labels while retaining selected/disabled metadata.
- Keep palette/icons, native v17, cancellation and one-operation history unchanged.
  Alpha.78–80 have no explicit version approval recorded; alpha.81 awaits approval.
  Windows deferred; VoiceOver acceptance remains at final review.

## 0.1.0-alpha.80 — 2026-10-23

- Add F6 entry/return for preview controls and wrapping Tab/Shift+Tab traversal.
- Activate the focused control with Enter/Space, without repeated key activation.
- Draw a focus outline using the existing accent and request native widget focus.
- Continue keyboard traversal from a pointer-clicked control; return to marker or
  numeric input without applying. Keep the existing marker Tab behavior.
- Skip disabled controls, recover focus to Cancel when a control becomes disabled,
  and preserve Escape cancellation and one-operation history.
- Palette/icons and native v17 are retained. Alpha.78/79 have no explicit version
  approval recorded; alpha.80 awaits approval. Windows deferred; VoiceOver at end.

## 0.1.0-alpha.79 — 2026-10-23

- Open the shared tab preview from the ruler marker context menu.
- Add pointer-accessible None/Dots/Dashes/Line choices and Restore leaders.
- Add Exact position / Back to marker controls without an intermediate commit.
- Add Apply/Cancel for the entire position/leader draft; invalid positions disable
  changes and apply, while Cancel remains available.
- Retain keyboard commands, mixed-owner rules, one Undo/Redo, native v17,
  palette and icons. Alpha.78 has no explicit version approval recorded;
  alpha.79 awaits approval. Windows deferred; VoiceOver at final review.

## 0.1.0-alpha.78 — 2026-10-23

- Add direct ruler-preview leader keys: 0 None, 1 Dots, 2 Dashes, 3 Line.
- Add Shift+F3 reverse cycling and Backspace to restore each original leader
  while retaining the draft position.
- Add F4 return from valid numeric input without applying; invalid input stays
  editable. Numeric digits retain their normal input meaning.
- Show the explicit leader choice separately from mixed-owner distribution.
- Preserve one-operation apply/Undo/Redo and Escape cancellation. Palette/icons,
  native v17 and layout policy remain unchanged. Alpha.77 approved; alpha.78
  awaits owner approval. Windows deferred; VoiceOver reserved for final review.

## 0.1.0-alpha.77 — 2026-10-23

- Show per-leader paragraph counts in ruler tooltips and widget labels, separating
  no fill from no explicit stop; preserve half-open selection semantics.
- Show counts and the common-value selection in the ruler leader context menu.
- Add read-only F1 leader details to keyboard tab previews, including mixed owners.
- Add F3 leader cycling in nonnumeric keyboard previews (mixed starts at None).
- Preserve a chosen leader through movement and F2 exact input, committing position
  and leader once; invalid/deletion previews cannot change leaders, cancel is inert.
- Native v17, palette/icons and design retained. Alpha.76 approved; alpha.77 is a
  five-step candidate. Windows/VoiceOver acceptance remains deferred.

## 0.1.0-alpha.76 — 2026-10-21

- Add None/Dots/Dashes/Line selection to the tab positions dialog, staged through
  Add/Update and committed with positions/alignment in one Undo/Redo.
- Render an illustrative live sample using the actual tab layout adapter.
- Detect mixed leaders as well as positions/types; require explicit replacement,
  preserving Cancel, pending-input validation and exclusive paragraph boundaries.
- Add ruler context-menu leader set/reset for owning selected paragraphs only.
- Extend keyboard dialog traversal and verify compact controls, staging/history,
  roundtrip and no-op Redo. Native v17 and palette/icons retained.
- Alpha.74 is approved; alpha.75 has no explicit approval recorded. Alpha.76 is
  one five-step candidate for review. Windows/VoiceOver acceptance stays deferred.

## 0.1.0-alpha.75 — 2026-10-21

- Store per-stop dot, dash and line leaders for explicit Left/Center/Right tabs.
  Add a validated shared set/clear command with one Undo/Redo; retain leader
  identity during ruler movement and remove it with the stop.
- Render bounded decorative leader meshes inside the tab advance without changing
  stored text, cursor positions, wrapping or field alignment. Grid tabs stay plain.
- Write native v17, retain v1–v16 reading and strictly reject malformed leaders
  and leader fields in older versions. Alpha.74 and older reject v17 saves.
- Add an openable TabLeaders fixture and regression coverage. Palette/icons remain
  unchanged; leader dialog controls are a later step. Owner approved alpha.74;
  alpha.75 awaits approval. Windows/VoiceOver remain deferred.

## 0.1.0-alpha.74 — 2026-10-21

- Fix keyboard tab previews getting stuck on occupied positions: arrows now move
  beyond a rejected destination while invalid previews remain uncommittable.
- Cancel pointer tab drafts if rulers are hidden or File view becomes active,
  keeping document/history unchanged and allowing keyboard re-entry.
- Extend typed-tab regression coverage for adjacent conflicts, mixed owners,
  numeric error/deletion recovery, stale selections, no-op Redo, transform
  cancellation and pointer-to-keyboard transitions. Preserve native v16 and design.
- Owner approved alpha.73; alpha.74 is a candidate for review, not a release.
  Windows and VoiceOver acceptance remain deferred.

## 0.1.0-alpha.73 — 2026-10-21

- Add a ruler creation-type selector: click/Ctrl+Alt+F7 cycles Left/Center/Right;
  its context menu selects directly. The window-local choice does not edit content.
- Create the selected type by pressing the empty ruler, previewing movement and
  releasing inside. Escape, focus loss or outside release cancels; quick clicks
  delivered in one frame still create exactly one history entry.
- Keyboard creation and F2 exact input use the selected type, frozen in the draft.
  Preserve same-type missing-only insertion, cross-type conflict rejection,
  cancellation/Redo and one Undo/Redo. Native v16, palette and icons retained.
- Windows and VoiceOver remain deferred; alpha.71–72 have no explicit approval recorded.

## 0.1.0-alpha.72 — 2026-10-21

- Edit Left/Center/Right positions in one dialog; Add/Update stages type changes,
  OK replaces the selected lists as one Undo/Redo, and Cancel preserves the document.
- Show left, centered and mirrored-right ruler markers with type/coverage labels;
  mixed types at one position have a distinct marker and preserve each owner's type.
- Move/delete all types by pointer, keyboard or exact F2 input using the existing
  validated preview/cancel transaction. Traversal includes every type.
- Retain native v16, palette/icons and left-tab creation defaults. Windows and
  VoiceOver acceptance remain deferred. Alpha.71 approval was not explicitly received.

## 0.1.0-alpha.71 — 2026-10-21

- Store typed center/right tab stops beside existing left stops; validate sorted,
  distinct positions and a combined 32-stop limit. Shared edits use one Undo/Redo.
- Align each following field to its center or right edge, including formatting,
  Unicode and zoom; preserve forward tab progression, wrapping and grid fallback.
- Write native v16 and retain strict v1–v15 readers. Older applications reject v16;
  retain an older copy when necessary. Add a runnable AlignedTabs.bfdoc example.
- Preserve palette/icons and existing left-tab controls; type selection/markers
  follow separately. Windows and VoiceOver acceptance remain deferred.

## 0.1.0-alpha.70 — 2026-10-21

- F2 opens exact numeric position input in keyboard ruler tab previews, for
  creation and movement. Enter/Tab applies once and exits; Escape cancels.
- Validate 0.5–1440 pt in half-point steps without rounding or clamping typed
  values. Invalid text and destination conflicts cannot apply an older preview.
- Preserve mixed-list rules, one Undo/Redo, native v15, palette and icons.
  Windows and VoiceOver acceptance remain deferred.

## 0.1.0-alpha.69 — 2026-10-20

- Ctrl+Alt+Shift+F8 starts a ruler tab creation preview at 36 pt. Ctrl+Alt+F8
  also creates when the selected paragraphs have no explicit stops.
- Arrows adjust by 0.5 pt, Shift+arrows by 5 pt; Enter/Tab applies once and exits.
  Escape/Delete cancels creation. Duplicate/capacity errors remain editable drafts.
- Preserve mixed-list missing-only insertion, one Undo/Redo, cancellation/Redo,
  native v15, design, palette and icons. Windows and VoiceOver remain deferred.

## 0.1.0-alpha.68 — 2026-10-20

- Show the sorted union of selected paragraph tab positions, with partial coverage
  in marker tooltips. Pointer and keyboard routes can edit mixed lists.
- Add a position only where missing; move/remove only where the source exists.
  Preserve every unrelated stop and commit the whole selection in one Undo/Redo.
- Reject destination/capacity conflicts without partial edits; retain preview,
  cancel, half-open selection and stale-source protection. Native v15, palette,
  icons and dimensions unchanged. Windows and VoiceOver remain deferred.

## 0.1.0-alpha.67 — 2026-10-20

- Click empty horizontal ruler space within the text column to create a left tab;
  right-click an existing tab to delete it. Each action uses one shared Undo/Redo.
- In keyboard tab mode, Delete previews removal; Enter or Tab applies and exits,
  Escape cancels. Existing movement and outside-release cancellation are retained.
- Reject duplicate/full/mixed lists without mutation. Preserve half-open selection,
  native v15, design, palette and icons. Windows and VoiceOver remain deferred.

## 0.1.0-alpha.66 — 2026-10-20

- Add Ctrl+Alt+F8 access to existing ruler tab stops, half-point arrow movement,
  Shift+arrow 5 pt movement and exact preview values. Enter applies and exits;
  Tab/Shift+Tab applies and selects the next/previous stop. Escape cancels.
- Reuse the validated preview and single Undo/Redo transaction; isolate text input,
  reject duplicate destinations and cancel on focus/geometry changes.
- Preserve native v15, palette and icons. Windows and VoiceOver remain deferred.

## 0.1.0-alpha.65 — 2026-10-20

- Show explicit left tab markers on the horizontal ruler. Drag an existing marker to preview half-point movement; release applies one shared Undo/Redo transaction across paragraphs with identical lists.
- Escape, focus/pointer loss, resize/zoom and release outside the ruler cancel. Duplicate destinations cancel without merging stops. Stale document/selection drafts cannot overwrite edits; no-op movement preserves Redo.
- Preserve indent marker priority: overlapping positions and mixed lists remain editable through Tab positions. No drag-to-create/delete or tab keyboard controller yet. Preserve design, palette, icons and native v15. Alpha.64 approved; alpha.65 awaits approval. VoiceOver final review and Windows remain deferred.

## 0.1.0-alpha.64 — 2026-10-20

- Add a native Tab positions dialog through Home → Paragraph spacing and command search. Add, update, remove or clear explicit left stops in a draft; OK commits the entire selected-paragraph change as one Undo/Redo operation.
- Preserve mixed selections until an explicit replacement is prepared, reject duplicates/range/limit errors, and prevent unsubmitted position input from being silently discarded by OK. Cancel/Escape and no-op acceptance preserve document/history; nested paragraph drafts are retained.
- Preserve design, palette, icons and native v15. Interactive ruler tab editing, other stop types, leaders and style inheritance remain planned. Alpha.63 approved; alpha.64 awaits approval. VoiceOver final review and Windows remain deferred.

## 0.1.0-alpha.63 — 2026-10-20

- Store explicit left tab positions in native v15: up to 32 strictly increasing unique positions, 0.5–1440 pt from the text-column origin. A shared command changes selected paragraphs in one Undo/Redo operation; empty lists restore interval-only layout.
- Lay out each Tab at the next explicit position, then resume the existing interval grid after the last position. Preserve indents, zoom, Unicode boundaries, cursor indices, formatting, paragraph editing and save/reopen. Reject malformed lists and the new field in older file versions.
- Preserve design, palette and icons. Position editing in the GUI/ruler, other tab types, leaders and named-style inheritance remain planned. Alpha.62 approved; alpha.63 awaits approval. VoiceOver final review and Windows remain deferred.

## 0.1.0-alpha.62 — 2026-10-20

- Add a native Tab interval dialog from Home → Paragraph spacing and command search. Apply 0.5–144 pt in half-point steps, or restore the default 36 pt, to selected paragraphs in one Undo/Redo operation.
- Show mixed interval settings, reject invalid input, preserve the paragraph dialog draft and leave the document unchanged on Cancel/Escape. Keep native v14 and the existing tab-layout limitations.
- Preserve design, palette and icons. Correct the File pane's stale native-format label. Alpha.61 approved; alpha.62 awaits approval. VoiceOver final review and Windows remain deferred.

## 0.1.0-alpha.61 — 2026-10-20

- Add left tab-grid layout, literal Tab input and per-paragraph interval storage in native v14. Default stops repeat every 36 pt; the validated engine interval API is one Undo/Redo operation.
- Preserve Unicode/grapheme boundaries, cursor indices, tab formatting, paragraph geometry and strict v1–v13 imports. Modified Tab cannot delete a selection.
- Tab paragraphs use fixed spacing (Justify falls back to left placement); custom ruler stops, leaders, named-style inheritance and a GUI interval editor remain future work. Preserve palette/icons and design. Alpha.60 approved; alpha.61 awaits approval. VoiceOver and Windows remain deferred.

## 0.1.0-alpha.60 — 2026-10-20

- Enable Home Increase/Decrease indent using shared Rust commands: change each selected paragraph's resolved left indent by 36 pt, clamped to 0–144 pt, in one Undo/Redo operation.
- Preserve right and first-line/hanging indents, unrelated formatting and clamped inheritance. Exclude the paragraph beginning at an exclusive selection end. No-op steps preserve redo.
- Add command-search entries and Docs-specific Home KeyTips II/DI. Preserve ribbon geometry, palette/icons and native v13. Alpha.59 approved; alpha.60 awaits approval. VoiceOver final review and Windows remain deferred.

## 0.1.0-alpha.59 — 2026-10-20

- Record the owner's decision to postpone VoiceOver speech/navigation acceptance until the final accessibility review. The owner clarified that VoiceOver was manually turned off; observed deactivation is not evidence of an OS or application failure.
- Preserve the alpha.58 AccessKit and native-keyboard evidence without claiming speech acceptance. No editor behavior, design, palette, icons or file-format changes. Windows remains deferred.
- Alpha.58 approved; alpha.59 documentation/validation increment awaits approval.

## 0.1.0-alpha.58 — 2026-10-20

- Expose full indent-source information on the focused marker while inspection is open, with keyboard instructions and an explicit uncommitted-preview indication for assistive technology.
- Route accessibility Focus/Click actions into the ruler keyboard controller without pointer dragging or document changes. Report unavailable markers as disabled.
- Audit ruler/menu focus and keyboard behavior through the actual AccessKit output; preserve design, palette/icons and native v13. Native screen-reader evidence is recorded separately in VALIDATION. Alpha.57 approved; alpha.58 awaits approval. Windows remains deferred.

## 0.1.0-alpha.57 — 2026-10-20

- Open the ruler context menu with Shift+F10 while a marker is focused. Anchor it to that marker and initially select its source command; arrows, Tab/Shift+Tab, Home/End navigate, Enter/Space activates, Escape returns to the marker.
- Preserve keyboard indent previews while opening or dismissing the menu. During a pending preview only its current source command is available; apply/cancel before other source/reset commands. Inspection retains the preview and reset acceptance remains one undo operation.
- Close stale keyboard menus on focus loss or cancelled resize previews. Preserve pointer behavior, design, palette/icons, native v13 and protocol/settings v1. Alpha.56 approved; alpha.57 awaits approval. Windows remains deferred.

## 0.1.0-alpha.56 — 2026-10-20

- Add explicit first-line, continuation-line, left and right source inspection commands to the horizontal ruler context menu. Opening resets list scroll and focuses the chosen marker; dismissal returns to that marker.
- Open the same context menu on ruler markers as on the scale; preserve primary-button dragging.
- Reuse the full source panel and its keyboard/wheel scrolling without document or history changes. Right-click takeover retains the existing cancellation of uncommitted ruler previews; F1 remains the draft-preserving route.
- Preserve menu reset commands, design, palette/icons and native v13. Add all-marker focus/read-only/compact-scale and takeover regressions. Alpha.55 approved; alpha.56 awaits approval. Windows remains deferred.

## 0.1.0-alpha.55 — 2026-10-20

- Scroll the open ruler source list with the mouse wheel while hovering its panel. Point, line and page input respect the list bounds; horizontal or modified wheel input does not change it.
- Consume background scroll and drain residual smoothing after dismissal, preserving the canvas position, marker focus and pending indent preview. A fresh wheel gesture after inspection can scroll the document again.
- Add unit/direction/bounds, pointer scope, synthetic-scale, cancellation and one-operation undo/redo coverage. Preserve design, palette/icons and native v13. Alpha.54 approved; alpha.55 awaits approval. Windows remains deferred.

## 0.1.0-alpha.54 — 2026-10-20

- Show every indent source group in the F1 ruler information view, with paragraph counts and wrapped style names. Hover tooltips retain their compact three-group summary.
- Add Up/Down, Page Up/Down and Home/End scrolling inside a bounded panel, with a scroll-position indicator and a fixed keyboard hint. Reopening starts at the top; inspection preserves marker focus, pending drafts and history.
- Preserve the two-axis ruler design, palette/icons, native v13 and protocol/settings v1. Add full-list and compact keyboard regression coverage. Alpha.54 awaits approval; the last explicit approval is alpha.52. Windows remains deferred.

## 0.1.0-alpha.53 — 2026-10-18

- Add F1 access to value/source details while a ruler marker has keyboard focus. Enter, Escape or F1 returns to that marker without applying or discarding its pending indent preview.
- Keep inspection read-only: editing/navigation keys are consumed while details are open. Focus loss, hidden rulers and pointer takeover retain existing cancellation policy.
- Preserve shared provenance, two-axis design, palette/icons and native v13. Add keyboard/draft/focus/bounds regressions and native captures. Alpha.52 approved; alpha.53 awaits approval. Windows remains deferred.

## 0.1.0-alpha.52 — 2026-10-18

- Show indent provenance in ruler marker tooltips and accessibility descriptions: direct paragraph formatting, supplying paragraph/ancestor style, or document default.
- Aggregate source counts independently of common/mixed values across the same half-open paragraph selection. First-line/continuation tips include left and special-indent sources; long source lists show three entries plus a remainder count.
- Reuse the existing formatting inspector; sources follow detached previews and Undo/Redo without adding edits. Preserve design, palette/icons and native v13. Alpha.51 approved; alpha.52 awaits approval. Windows remains deferred.

## 0.1.0-alpha.51 — 2026-10-18

- Add separate ruler menu commands to reset left, right or first-line/hanging indentation to the paragraph style, preserving every other direct override.
- Shift+Delete previews reset of the focused marker property; first-line and continuation markers share the signed first-line/hanging property. Delete retains reset-all. Enter/Tab commits one Undo/Redo; Escape/F6 cancels.
- Keep per-paragraph inheritance, half-open selection, no-op redo and persistence semantics. Preserve two-axis design, palette/icons and native v13. Alpha.50 approved; alpha.51 awaits approval. Windows remains deferred.

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
