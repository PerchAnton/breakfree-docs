# Native editor UI specification

Owner-approved reference: **Microsoft Word 2024 desktop for Windows**, classic
expanded multi-row ribbon, English UI and light theme. Rust is mandatory; the
library does not determine the product layout. This specification is an initial
measurable implementation baseline, not a claim of pixel equivalence.

## Reference evidence and limits

Checked 2026-10-17. [Microsoft navigation documentation](https://support.microsoft.com/en-us/accessibility/word/use-a-screen-reader-to-explore-and-navigate-word)
identifies title/quick actions, ribbon, document, status, navigation and File areas.
[Microsoft keyboard documentation](https://support.microsoft.com/en-us/accessibility/word/keyboard-shortcuts-in-word)
supplies the familiar file/edit/alignment, command-search and ribbon-collapse keys.
Both apply to Word 2024 and other versions; they are not a frozen screenshot of a
particular retail build. The [publisher's Word 2024 sample, pp. 5–11](https://store.ccilearning.com/wp-content/uploads/2026/01/3290-Microsoft-2024-Word-Associate-Book-Sample.pdf)
corroborates the ordered Home/Insert/Draw/Design/Layout/References/Mailings/Review/
View/Help tabs and Home grouping. Its PDF text was accessible, but the screenshot
endpoint failed. Draw visibility and contextual tabs depend on settings/content.

The official [Word 2024 theme announcement](https://support.microsoft.com/en-gb/word/what-s-new-in-word-2024-for-windows-and-mac) and its [theme image](https://support.microsoft.com/en-gb/word/media/office-theme.png) were also inspected in a browser. The Word portion shows a light, rounded ribbon surface, compact icon rows, understated tab selection and a style gallery. This is a publisher image without controlled window/DPI/build metadata, so it informs composition only. No external artwork is included in the application.

A running Windows/Word 2024 reference is unavailable on this macOS host. Therefore
**no matched screenshot comparison, measured Word pixel deltas, visual fidelity
percentage or native Windows 100%/125%/150% pass is reported**. Exact reference build,
font metrics, spacing and order under a named Windows configuration remain an R1 gate.

## Implemented geometry

Values below are Docs design values in device-independent points (DIP), **not
measurements taken from Word**. Source constants are in `src/presentation.rs`; original vector controls are in `src/chrome.rs`.

| Area | Docs value / observable behavior |
| :--- | :--- |
| Default window / minimum | 1366 × 900 / 640 × 480 DIP; native custom title controls |
| Title | 44 DIP high; 24 × 24 approved transparent icon, 28 × 28 Save/Undo/Redo icons; actual filename and dirty marker; 28-DIP search field; 46-DIP-wide vector window controls |
| Tabs | 32 DIP; File, Home, Insert, Draw, Design, Layout, References, Mailings, Review, View, Help; narrow windows use More tabs |
| Ribbon | 104 DIP expanded; near-white rounded inset (8 DIP horizontally / 4 vertically), 88-DIP groups with bottom captions; collapsible via button, double-click tab or Ctrl+F1 |
| Home groups, full | Clipboard 86, Font 304, Paragraph 246, Styles 350, Editing 116 DIP; 8-DIP group gaps and separators |
| Home groups, compact | Below 1180 DIP available ribbon width: 82/246/218/140/104 DIP; compact Clipboard and one style preview |
| Home overflow | Below 900 DIP available ribbon width: 82/232/206 DIP; Styles/Editing move into More menu; the pinned ribbon reserves workspace; temporary mode overlays it |
| Controls | Small buttons 24–32 × 24–26 DIP; Paste 46 × 69 split into a 37-DIP icon action and 32-DIP options menu; two rows in Font/Paragraph, three in Clipboard/Editing; 13-DIP UI text, 11-DIP captions; 15 × 16 group launchers |
| Font size | 64 × 25 DIP unified outline: 46-DIP editable field and 18-DIP preset arrow; font-family field is 112/64/50 DIP in full/compact/overflow density |
| Workspace | Neutral #ebebeb; white continuous canvas, border #c5c5c5; soft canvas shadow; 10-DIP workspace top gap; centered when space allows; horizontal/vertical scrolling when required |
| Canvas | 816 DIP wide at 100% document zoom, 96-DIP side/top margins, content-driven height; **no page segmentation** |
| Text | Host Arial/DejaVu or embedded fallback; default 11 pt (14.667 DIP), per-run 1–400 pt in half-points and underline; 1.25 line height, default-size 0.5-em paragraph gap; model styles/alignment projected into hit-testable text |
| Ruler | 22-DIP-high horizontal canvas ruler with shaded margins, 96 DIP per inch, eighth-inch ticks; zoom scales with document; no draggable margins or vertical/page ruler yet |
| Panes | Navigation left 240 DIP (195–320); Formatting right 220 DIP (195–300); visible on request |
| Status | 24 DIP; real Unicode word count and paragraph count, saved state, continuous-layout label and 50–200% zoom; ±10% buttons and click percentage to reset to 100% |
| File | 180-DIP left action rail and scrollable Home/Options body; 158 × 160 blank-document tile, real current-file properties; New/Open/Save/Save As active, Print disabled with explanation |

## Commands and interaction

Active Home commands are plain-text clipboard, bold/italic/underline, font size,
Grow/Shrink Font, four alignments, Find, Replace selected/next match and Select All. Emphasis controls show selected state and set
pending insertion style at an empty selection. Original vector strokes draw clipboard, file, history, search, paragraph, window and
launcher controls. Button names are exposed to the toolkit accessibility tree.
Selected formatting has a pale violet fill even without hover; focus uses an outline.
Font-family selection remains read-only with an explanation. Font size is editable:
1–400 pt in half-point steps, an empty field for mixed selections, and common presets
in a scrollable menu. Enter or leaving the field applies one shared command; Escape
cancels. Tab preserves traversal to the next control; Enter returns to the document.
Invalid input produces a visible error without changing model/history. Keyboard Save
commits pending size input before writing. Size and underline are disabled during
document IME composition. At a caret they set the next insertion style only.
Grow/Shrink Font steps each run independently through common presets (8–72 pt);
outside that interval it moves one point, bounded to 1–400 pt. Mixed selections
retain differing sizes; one click is one undo operation. Ctrl/Command+Shift+>/<
(period/comma) invokes the same command while the document has focus. Composition
disables these commands. The Paste icon performs plain-text paste; its lower menu
contains the same action and an explained disabled Paste Special. Find opens
navigation directly; its separate 18-DIP arrow opens options. Select opens a real
command menu; overflow retains Find/Replace/Select All/Formatting.
Style previews remain disabled and do not claim applied model styles. A single
63-DIP gallery holds inline Normal/No Spacing/Heading 1 names and three disabled
navigation buttons; no selected style frame is drawn.
Native egui hover/pressed/disabled/focus states remain active; unsupported controls
have tooltips explaining the missing capability. Styling uses restrained violet
#7047ae, pale selected-state fill, light chrome and readable dark text.

The Font group launcher opens a modal with a 420-DIP content width: Font, Font style
and Size columns (172/136/92 DIP), preset list, mixed-state Underline checkbox,
54-DIP-high clipped preview and 82 × 27 DIP OK/Cancel buttons. These are Docs design
values, not measured reference values. Style choices set bold/italic together;
untouched properties retain mixed values. Size is initially focused and selected.
Enter/OK applies one command; Escape/Cancel discards all draft changes. Invalid size
stays visible in the dialog. Background document commands and window close are
blocked until the draft is resolved. Font-family choice is explained and disabled.
The Paragraph launcher retains the formatting pane. The official keyboard source
above confirms the Windows Font dialog shortcuts; its complete dialog and effects
are not claimed as implemented.

Other tabs expose disabled, explained controls. View has real rulers, navigation
and zoom; Help shows version and shortcuts. Object-specific contextual tabs are
planned when their document objects exist; none is shown for imaginary objects.
No application account, cloud connection or copied product branding is present.

Text input uses the actual native toolkit text widget with a model-derived rich
galley; it is not a painted string over a hidden document. Committed input and IME
commit call the same Rust commands as tests. Preedit stays transient. The model
owns undo; one widget update batches committed text events into one transaction.
Explicit paste/deletion/formatting commands have separate history boundaries.
Left/Right and deletion honor extended grapheme boundaries. Tab traverses controls;
F6/Shift+F6 cycles the document, visible navigation/formatting panes, status and
ribbon; Alt+Q searches real commands. Standard
Ctrl/Command N/O/S/Shift+S, B/I/U, Shift+P (focus/select size), Z/Y/Shift+Z, F/H and
L/E/R/J are implemented. Ctrl/Command+D or Shift+F opens the Font dialog.
Home/File KeyTips and Alt entry, plus quick-access/tab/View extensions, are described below. Contextual KeyTips,
native assistive technology and multilingual OS IME remain unverified/incomplete.

## Alpha.14 navigation results

The existing Home Find/Replace and View Navigation entry points now share the
same literal matcher. The Windows composition reference includes search results
and previous/next arrows ([Microsoft navigation documentation](https://support.microsoft.com/en-gb/word/use-the-navigation-pane-in-word?nochrome=true), inspected 2026-10-17; applies to Word 2024 Windows).
This source supports the workflow, not measured pixel equivalence or every shortcut.

| Component | Implemented measurement / behavior |
| :--- | :--- |
| Navigation pane | 260 DIP default, resizable 220–320 DIP; neutral `SURFACE`; consumes workspace width |
| Search field | 26 DIP height, adjacent Clear; focus retained during result traversal |
| Results header | Current result / total from model ranges; 24 × 24 DIP original vector chevrons |
| Result list | Virtual 64-DIP rows plus 3-DIP spacing, all matches available; real paragraph number and bounded context |
| Match cues | Bold excerpt text, existing violet selection; selected document range remains visible while search owns focus |
| Keyboard | Enter/Down next, Shift+Enter/Up previous while in search; wraps, Escape closes pane and restores document focus |
| Unsupported modes | Headings and Pages disabled with style/pagination explanations |

Exact case-sensitive, nonoverlapping matching rejects partial grapheme clusters;
no normalization, case folding, wildcard/whole-word options or replace-all. A
result selection changes no document/history; replacement uses the existing shared
command and the matched run's insertion style. Search updates after edits/undo.
Excerpts limit preceding/matched/following context to 18/32/24 graphemes, with an
ellipsis when shortened. Paragraph/tab separators use display-only separators;
source text is never rewritten. Long matches are selected in full in the document.

### Alpha.14 native captures

Captured on macOS, English/light theme, 100% document zoom, backing scale 2:

- [Navigation, 1366 × 900 DIP](screenshots/rust-navigation-alpha14-macos.png).
- [Navigation, 640 × 480 DIP](screenshots/rust-navigation-alpha14-macos-narrow.png).

The harness searches the actual Welcome fixture. Alpha.13 colors/icons and Home
gutters are retained. Native long-document input separately checks wrapped
navigation through 55 matches. Initial inspection caught missing font glyphs in
arrow labels and context: chevrons now use the existing vectors, excerpts the
same body-font families as the document. Remaining complex-script/font fallback
limits still apply. At 640 DIP the continuous canvas scrolls horizontally; it
is not repaginated. Native Windows 100/125/150%, a matched reference build and
full accessibility/IME remain unverified. No similarity score is assigned.

## Alpha.13 icon palette

On 2026-10-17 the owner supplied two local reference images showing a macOS
editing window and start screen in Russian. They support the requested color
roles: light neutral surfaces, dark glyphs, restrained blue details, a gold
clipboard, yellow highlighting and a red text-color indicator. They do not
identify the application's exact version/build or a Windows configuration.
Source images and desktop/file-list content are not included in this repository.
The approved Windows composition target remains; these are supplemental palette
references, not a controlled same-size/theme/language/zoom comparison.

| Role | Docs sRGB value / use |
| :--- | :--- |
| Window chrome | `#f3f4f5` title/tabs/status |
| Ribbon and File surface | `#fafafa`; white document/field interiors |
| Workspace / File rail | `#ebebeb` / `#ebedef`, replacing the tinted File rail |
| Text / boundaries | `#323130` / `#d5d5d5` |
| Hover / pressed | `#e7eaee` / `#dce0e5` |
| Brand focus / selection | `#7047ae` / `#e8e0f2` |
| Blue / pale blue | `#3a7db5` / `#e3eff9`: Save/New/Copy, scissors handles, arrows and text-effect glyph |
| Gold / pale gold | `#b0893e` / `#fff5d6`: Clipboard, Open and brush details |
| Yellow / red / eraser | `#f4d23e` / `#ca414c` / `#b771a7`: small highlight/color/eraser cues |

These are chosen Docs values, not sampled or certified reference colors. Original
Rust vector geometry uses dark outlines and small colored parts. Disabled icon
accents blend one part semantic color with two parts neutral `#e8e8e8`; labels
remain disabled gray. Unsupported highlighting, text color, effects and Format
Painter still do nothing and explain why. Their swatches are icon cues, not
claims about the current document's formatting. Hover, pressed, selected and
focus states retain the native button interaction. File Home, Options and Print
now share the same icon/text alignment as the existing file actions; Print stays
disabled. Home group widths, 8-DIP gaps, hit regions and overflow rules are unchanged.

### Alpha.13 native captures

Captured and visually inspected on macOS on 2026-10-17, English UI, light theme,
100% document zoom and backing scale 2:

- [Home, 1366 × 900 DIP](screenshots/rust-palette-alpha13-macos.png).
- [Home, 640 × 480 DIP](screenshots/rust-palette-alpha13-macos-narrow.png).
- [File Home, 1366 × 900 DIP](screenshots/rust-palette-alpha13-macos-file.png).

Home uses the real Formatting fixture (21 words/four paragraphs); File starts
with an empty document. Icon/label separation, Clipboard/Font gutter, disabled
cues and File selection were inspected. Normal-binary input also checked active
Cut/Copy appearance, Bold/history, File Options/Home/Back and F10 Home navigation.
Intentional differences are the approved Docs logo, violet focus/selection and
original icon geometry. Reference typography, precise spacing, saturated enabled
advanced commands, start-screen templates/recents and native platform chrome
still differ. No numerical similarity score or Windows scaling pass is inferred.

## Alpha.12 toolbar ordering

More Commands in the customization menu opens an ordered list of the currently
visible commands, with Move Up/Move Down controls on its right and OK/Cancel below.
The [official customization guide](https://support.microsoft.com/en-us/office/customize-the-quick-access-toolbar)
was checked on 2026-10-17 for this sequence. Its current applies-to list covers
several Office versions but does not explicitly list Word 2024; this is behavioral
guidance, not a verified Word 2024 dialog image or pixel measurement.

| Element | Docs value / behavior |
| :--- | :--- |
| Dialog content | 420 DIP wide, light surface, neutral border and restrained violet selection/focus |
| Ordered list | 368-DIP content width, 3-DIP inset, 26-DIP rows; stable command IDs and numbered accessible names |
| Move controls | 30 × 28 DIP, original vector arrows and bordered hover/pressed/disabled controls; unavailable directions disabled |
| Footer | 82 × 27 DIP OK/Cancel; no immediate writes while drafting |
| Keyboard | Up/Down/Home/End select; Alt+Up/Down move; Tab/Shift+Tab cycles enabled targets; Enter/Space activates focused buttons; Enter from list accepts; Escape cancels |

Selection follows the moved command; focus returns to its row if a move button
becomes disabled. Background editing, shortcuts and window close are blocked while
the draft is open. OK updates the toolbar and its numeric KeyTips, saves changed
preferences and restores focus to customization. Cancel/Escape restores that focus
without changing the toolbar. Failed writes retain previous file bytes and use the
existing error/Retry Saving path. Document content/history/selection are unchanged.
The Alt+arrow shortcuts are Docs choices pending native Windows reference review.

### Alpha.12 native captures

Captured and inspected on macOS on 2026-10-17, English UI, light theme, 100% document
zoom and backing scale 2. The fixture enables all five supported commands below
the ribbon, using the real 20-word/four-paragraph Welcome document:

- [Ordering dialog, 1366 × 900 DIP](screenshots/rust-qat-order-alpha12-macos.png).
- [Ordering dialog, 640 × 480 DIP](screenshots/rust-qat-order-alpha12-macos-narrow.png).

The list, labels, controls and footer fit both sizes. Native input additionally
verified focus following Undo, draft cancellation, apply, renumbered Undo/Redo and
restart in the normal binary. The full Options catalog, add/remove lists, import/
export customization and context menu remain unimplemented. BreakFree identity and
violet focus are intentional brand differences; dialog composition, font metrics,
spacing and exact shortcuts still require a matched Word 2024 Windows comparison.
The existing Welcome emoji fallback remains visible. See [validation](VALIDATION.md#alpha12-toolbar-order-validation).

## Alpha.11 Home separator correction

Each Home group now reserves its complete specified width independently of its
contents. The 8-DIP gap is retained at every density, with a 1-DIP separator at
its center (3.5 DIP clear of both neighboring group rectangles). This corrects
Clipboard's separator crossing Bold and the font-family border. The same fix
applies to every Home group, including temporary expansion. Existing group sizes,
light background, captions and control styling remain as specified above.

### Alpha.11 native captures

Captured 2026-10-17 on macOS, English UI, light theme, 100% document zoom and
backing scale 2. These are actual application renders, not matched Word images:

- [Full Home, 1366 × 900 DIP](screenshots/rust-home-gutters-alpha11-macos.png).
- [Compact Home, 960 × 700 DIP](screenshots/rust-home-gutters-alpha11-macos-compact.png).
- [Overflow Home, 640 × 480 DIP](screenshots/rust-home-gutters-alpha11-macos-narrow.png).
- [Temporary Home, 640 × 480 DIP](screenshots/rust-home-gutters-alpha11-macos-temporary.png).

The first three use `Formatting.bfdoc` (21 words, four paragraphs); temporary mode
uses `Welcome.bfdoc` (20 words, four paragraphs). Inspect the Clipboard/Font gap,
font-family outline, emphasis row, adjacent dividers and More button. The known
Welcome emoji fallback remains visible. The capture helper's temporary fixture
supplies deterministic focus, as its KeyTips fixtures already do. Native pointer
Bold, size entry, undo/redo and temporary Home were checked separately in the
normal executable; see [validation](VALIDATION.md#alpha11-home-separator-validation).

## Alpha.10 persistent quick-access settings

The alpha.9 geometry and five real commands remain. Visibility/order/placement
now persist per user/profile; the menu says “Applies to all documents.” Existing
windows retain their own state, and conflicting saves report an error. The
[official customization guide](https://support.microsoft.com/en-us/office/customize-the-quick-access-toolbar)
was rechecked on 2026-10-17 for above/below placement, command visibility and reset.
This multi-version guidance is not a measured Word 2024 Windows baseline.

| Component | Measurable behavior / dimensions in DIP |
| :--- | :--- |
| Customize menu | Minimum width 230; existing checkbox order and 24-high action targets retained. Saved order drives toolbar numeric hints after restart |
| Reset confirmation | 370-wide content; exact default commands/placement and backup explanation. Tab focuses Reset; Enter/Space activates the focused button; Escape cancels. Background editing/commands and close are blocked until resolved |
| Error feedback | 430-wide content with reason/path and OK. Menu retains “Not saved · this window only” and Retry Saving; successful retry clears warning |
| Backup feedback | Menu shows “Settings backup kept”; tooltip gives the actual backup path |
| Unchanged geometry | 28 × 28 toolbar commands on 30 pitch; 24 × 28 arrow; 32-high below-ribbon strip; same compact Home layout |

[ADR 0004](adr/0004-local-toolbar-preferences.md) defines strict settings v1 and
failure semantics. Reset backup/retry messaging is an intentional Docs addition.
Reordering controls, document-specific toolbar profiles, import/export, context
customization and exact reference keyboard/window behavior remain unimplemented.

### Alpha.10 native captures

Actual Rust rendering on macOS, Light theme, English UI, 100% document zoom,
backing scale 2. Three inspected captures use an isolated temporary settings profile:

- [1366 × 900 DIP customization after loading saved below-ribbon settings](screenshots/rust-qat-settings-alpha10-macos.png).
- [640 × 480 DIP reset confirmation](screenshots/rust-qat-reset-alpha10-macos-narrow.png).
- [640 × 480 DIP damaged-settings error](screenshots/rust-settings-error-alpha10-macos-narrow.png).

All shown controls/text fit their captured windows. Error paths wrap; the original
fixture's missing emoji glyph remains a font-fallback limitation. These are native
renderings, not Windows DPI or matched Word screenshots. No similarity score.

Reproduce with the existing `capture_ui` example: modes `keytips-settings`,
`keytips-settings-reset`, `settings-error`, the sizes above, and
`BREAKFREE_CAPTURE_TO` set to an absolute PNG path. These modes use isolated fixture
preferences; staged key events do not establish OS input. The separate ordinary
binary keyboard/restart checks are recorded in [VALIDATION](VALIDATION.md).

## Alpha.9 quick-access and View

Historical checkpoint; alpha.10 above supersedes window-local settings and reset.

Inspected 2026-10-17: Microsoft's [toolbar customization](https://support.microsoft.com/en-us/office/customize-the-quick-access-toolbar),
[keyboard toolbar guidance](https://support.microsoft.com/en-us/accessibility/office-accessibility/use-a-keyboard-to-customize-the-quick-access-toolbar)
and [View zoom guidance](https://support.microsoft.com/en-us/excel/zoom-in-or-out-of-a-document-presentation-or-worksheet).
These support above/below placement, command visibility, horizontal keyboard
navigation and separate Zoom/100%/width controls. The pages cover multiple Office
versions; they are not a measured Word 2024 Windows screenshot baseline.

| Component | Implemented behavior / dimensions in DIP |
| :--- | :--- |
| Quick Access Toolbar | Default Save/Undo/Redo; optional New/Open. Commands are 28 × 28 on a 30 pitch; customization trigger 24 × 28. Above: x=43, y=8 in the 44-high title. Below: x=8, y=2 in a separate 32-high strip |
| Customize menu | Minimum width 230; visibility checkboxes, Show Above/Below, Reset. Window-local only, newly enabled commands append. Defaults restore order and above-ribbon placement; no file/model/history change |
| Keyboard | Root hints 1–5 follow visible order, including disabled history slots. Alt+0 opens customization. Up from focused tabs enters the toolbar; Down also enters when below. Left/Right skips unavailable commands; Enter/Space activates, Down opens options, Tab returns to tabs. Menu Up/Down/Tab, Enter/Space and layered Escape use the shared focus policy |
| Collapsed mode | A below-ribbon toolbar remains visible. The temporary ribbon starts after its 32-high strip and does not cover its controls. Opening the temporary ribbon still leaves the canvas position unchanged |
| View | Views/Show/Zoom widths 120/156/264, 8-DIP gaps, 88-high inner groups in the existing 104-high ribbon. View-mode tiles 58 × 66, Zoom/100% tiles 52 × 66, Show rows 23 high on 24 pitch, zoom side commands 144 × 23. Checkbox left edges align |
| Zoom | 100% exits fit mode. Canvas Width recomputes `(workspace width - 48) / 816 * 100`, clamped to 50–200%, after window or pane changes. Slider/plus/minus/dialog acceptance exit fit; cancelling the dialog preserves it. Status percent opens the dialog |
| Limits | Ruler is a canvas ruler. Read Mode, Print Layout, Gridlines, One Page and Multiple Pages are disabled with explanations. Fit can reach its 50% minimum before a very narrow workspace fits; no pagination is implied |

View keeps K/R/Q and adds J for 100% (provisional), C for Canvas Width (Docs-specific).
Alt+0 is also Docs-specific. The menu currently covers five file/history commands;
arbitrary reordering, context-menu customization, persisted settings, complete
Word keyboard equivalence and native screen-reader acceptance remain open.

### Alpha.9 native captures

Actual Rust native rendering on macOS, Light theme, English UI, backing scale 2.
Four captures inspected: View and QAT menu at 1366 × 900 DIP, below-ribbon toolbar
at 960 × 720, Canvas Width at 640 × 480 (displayed 73%). The first layout inspection
caught centered rather than aligned Show checkboxes; the corrected geometry is
regression-tested. No supported label clips in these captures. The existing
fixture emoji still has a missing glyph; this is not a full-script rendering claim.

![View groups, alpha.9, macOS](screenshots/rust-view-alpha9-macos.png)

[QAT menu](screenshots/rust-qat-menu-alpha9-macos.png) ·
[QAT below the ribbon](screenshots/rust-qat-below-alpha9-macos.png) ·
[narrow Canvas Width](screenshots/rust-view-fit-alpha9-macos-narrow.png).

```sh
BREAKFREE_CAPTURE_TO="$PWD/docs/screenshots/view-review.png" \
  cargo run --locked --release --features capture --example capture_ui -- 1366 900 view
```

Other new capture modes: `view-fit`, `keytips-qat`, `keytips-qat-below` (the last
uses staged fixture key events to move the toolbar). Windows PowerShell uses
`$env:BREAKFREE_CAPTURE_TO` as in the existing capture instructions below.
These captures validate rendering; native keyboard evidence is recorded separately
in VALIDATION.md. No Word installation or matched Windows 100/125/150% comparison
was available. Intentional brand differences and the comparison gate below remain.

## Alpha.8 quick-access and tab KeyTips

Historical checkpoint; alpha.9 above supersedes fixed toolbar order and View geometry.

This extends alpha.7's input state machine without moving editing into the GUI.
The [official QAT keyboard guide](https://support.microsoft.com/en-us/accessibility/office-accessibility/use-a-keyboard-to-customize-the-quick-access-toolbar)
describes position-based numbers; numbers depend on toolbar order, not a universal
Word default. The [official Word navigation guide](https://support.microsoft.com/en-us/accessibility/word/use-a-screen-reader-to-explore-and-navigate-word)
documents Alt+W,K for navigation and Alt+W,Q for Zoom. Inspected 2026-10-17.
No controlled Word 2024 Windows instance is available. **JI Draw, Y1 Help and
W,R ruler are provisional Docs assignments** pending that comparison; do not
present them as verified mappings for a named Word 2024 build.

| Level | Implemented behavior |
| :--- | :--- |
| Quick Access Toolbar | Root 1 Save, 2 Undo, 3 Redo; also direct Alt+1/2/3. Fixed positions never renumber when history is empty. Unavailable actions show gray hints and do nothing; enabled actions use the same command/history/save path as the buttons |
| Tabs | H Home, N Insert, JI Draw, G Design, P Layout, S References, M Mailings, R Review, W View, Y1 Help. Multi-key prefixes filter hints and support Backspace. Direct Alt+first-key starts a sequence; later keys may retain Alt |
| Narrow tab overflow | Root T opens the actual More tabs menu; hidden rows show their tab codes at the right. T is Docs-specific. Direct hidden-tab sequences work without opening the menu. Escape closes this menu and returns to root; expanding the window past overflow returns to root without a stranded popup |
| Unimplemented tabs / Help | Retain their own scope, suppress invalid access-key text, explain unavailable commands or informational Help, and return to root with Escape. No action hints are invented for placeholder groups |
| View | K toggles Navigation Pane, R toggles canvas rulers, Q opens Zoom. Active toggle state is visible. Arrows/Tab hand off to the same enabled-control focus registry; Ctrl+Left/Right moves between Show and Zoom |
| Zoom | Modal 280-DIP content width, selected percent field, 75/100/150/200% presets, bounded whole-number 50–200% input, OK/Enter and Cancel/Escape. Invalid values retain the draft. Background shortcuts and window close cannot bypass it. Acceptance changes view scale only, with no document-history entry |

QAT badges use dedicated title anchors rather than File/other Save button bounds.
Existing badge geometry remains 17 DIP high, 12-DIP text; disabled badges use
#e0e0e0 with #7d7d7d text. View uses a 110-DIP Views column, 145-DIP Show buttons
and an 88-DIP Zoom button, with 30-DIP control heights inside the existing ribbon.
These are measured implementation dimensions, not reference-app measurements.
The continuous canvas, 50–200% zoom range and lack of pagination remain explicit.
Toolbar customization, contextual object commands and advanced tabs are not added.

### Alpha.8 native captures

![Quick-access and all-tab hints, alpha.8, macOS](screenshots/rust-keytips-root-alpha8-macos.png)

[View hints](screenshots/rust-keytips-view-alpha8-macos.png) ·
[More tabs at 640 × 480 DIP](screenshots/rust-keytips-tabs-alpha8-macos-narrow.png) ·
[Zoom at 640 × 480 DIP](screenshots/rust-zoom-alpha8-macos-narrow.png).

Captured and inspected on 2026-10-17: English/light, Welcome fixture, 100% document
zoom, backing scale 2; wide captures 1366 × 900 DIP. Root uses tabs-only mode.
The native capture fixture waits for both measurement frames and popup animation;
it injects deterministic events and does not prove OS key delivery. Native macOS
interaction is recorded separately in VALIDATION. The inherited missing emoji
glyph remains visible. Brand differences remain the original logo/name and violet
accents. Remaining reference differences include simplified View controls/Zoom,
fixed quick-access order, continuous canvas and unimplemented advanced tabs. No
matched Word 2024 Windows capture or similarity percentage is available.

## Alpha.7 KeyTips and Alt entry

The [official keyboard guide](https://support.microsoft.com/en-us/accessibility/word/keyboard-shortcuts-in-word),
[official Home KeyTips image](https://support.microsoft.com/en-us/accessibility/media/word-365-ribbon-keytips.png)
and [text-formatting guide](https://support.microsoft.com/en-gb/accessibility/word/use-a-screen-reader-to-insert-and-change-text-in-word)
were inspected on 2026-10-17. They support Alt/F10, Home entry, the emphasis,
size/Font and alignment routes. The image is labeled Word 365, not a controlled
Word 2024 screenshot. [File keyboard guidance](https://support.microsoft.com/en-us/accessibility/office-accessibility/keyboard-shortcuts-for-the-file-menu-in-microsoft-365-for-windows)
informs the File routes. Exact mappings and behavior under a named Word 2024 build,
especially Editing and paste submenus, remain subject to the Windows comparison.
No reference image is bundled as application artwork.

| Entry / level | Implemented sequence and effect |
| :--- | :--- |
| Bare Alt release / F10 | Show top-level hints; repeat to leave and restore prior focus. Alt used in a chord, with Shift/Ctrl/Command, or across focus loss does not arm bare-Alt entry |
| Top level | F File; H Home; Q search. N/G/P/S/M/R/W select Insert/Design/Layout/References/Mailings/Review/View and hand off to existing focus navigation; hidden-tab routes still work |
| Home Font | 1 Bold, 2 Italic, 3 Underline; FS editable size, FN Font dialog, FG/FK Grow/Shrink |
| Home Paragraph | AL/AC/AR/AJ left/center/right/justify |
| Home Clipboard | X Cut, C Copy; V opens Paste, T inserts plain text. Only enabled commands receive hints |
| Home Editing | FD Find, R Replace; SL opens Select, A selects all. SL remains a two-step menu route when Editing moves into More |
| Responsive More | E opens the actual overflow; FD/R/A/P select Find/Replace/Select All/Formatting. E and P here are Docs-specific assignments |
| File | N New, O Open, S Save, A Save As, T Options. These use Docs' existing local actions/settings, without additional reference-app pages or cloud services |
| Prefix / cancellation | Show matching full codes after a prefix; Backspace removes one letter, invalid input keeps it unchanged. Escape clears prefix, then menu → Home → top level → prior focus |
| Navigation / pointer | Tab/arrows/Enter/Space/F6 dismiss hints and continue existing navigation. Pointer press dismisses hints before normal click routing. Commands retain their usual focus destination |

Badges are **Docs design values**, not Word measurements: 17 DIP high, width
10 + 7 × code length, 12-DIP white text on #373737, 2-DIP corners. They are painted
on a noninteractive tooltip layer using real control rectangles. Main-control
centers sit 2 DIP below the lower edge; menu/File hints sit at the right of each
row. Prefix filtering does not change layout. Disabled features have no active
hint; Cut/Copy hints appear only with a selection. No imaginary command is added.

`gui::keytips` owns only presentation/input state. It dispatches the same Action
and menu handlers as pointer/focus navigation. Input processing preserves event
order, consumes access-key text partners and suppresses held-key repeats until
release. Size entry immediately resumes native text handling. IME events, active
composition and modal drafts block entry. Ctrl+Alt key events cannot be mistaken
for Ctrl-only commands; their committed text is retained (synthetic AltGr case).

Bare Alt observes egui modifier-state transitions; a very short tap coalesced
entirely between frames may be invisible to this adapter. F10 remains available.
The macOS automation tool rejects modifier-only presses, so bare Alt has event-test
evidence only. Native AltGr, layout switching, IME and Windows 100/125/150% still
need real-device acceptance. The alpha.8 section records subsequent quick-access,
tab and View coverage; no full reference-app keyboard equivalence is claimed.

### Alpha.7 native captures

![Home KeyTips, alpha.7, macOS](screenshots/rust-keytips-home-alpha7-macos.png)

[Top-level hints](screenshots/rust-keytips-root-alpha7-macos.png) ·
[Prefix F](screenshots/rust-keytips-prefix-alpha7-macos.png) ·
[Home at 640 × 480 DIP](screenshots/rust-keytips-home-alpha7-macos-narrow.png).
Captured and inspected on 2026-10-17: English/light theme, Welcome fixture,
100% document zoom, backing scale 2. Wide views are 1366 × 900 DIP. The capture
harness injects deterministic focus/key events into the actual native renderer;
these pictures establish rendering, not OS key delivery. The separate native
interaction smoke is recorded in VALIDATION. The inherited missing emoji glyph
and continuous canvas remain visible. Brand differences remain the approved logo,
name and restrained violet accent; no reference branding is used. A matched Word
2024 Windows capture is unavailable, so no pixel deltas or similarity percentage
are supplied.

## Alpha.6 collapsed ribbon and Home keyboard behavior

The [official ribbon visibility guidance](https://support.microsoft.com/en-us/office/foundations-experiences/show-or-hide-the-ribbon-in-office)
and keyboard reference above were checked on 2026-10-17. They cover Word 2024 and
other Office versions; the implementation below remains a documented subset.

| Trigger | Implemented behavior |
| :--- | :--- |
| Ctrl+F1 / double-click tab / tab context menu | Switch pinned ribbon and tabs-only mode; close temporary expansion |
| Click a collapsed tab | Overlay the same 104-DIP ribbon directly below the 44+32-DIP title/tabs; retain the document's position and collapsed preference |
| F10 / Alt+H | Focus active tab / open Home and focus its first enabled control |
| Left/Right on tabs; Tab/Down/Enter/Space | Change the active tab; enter its commands (Home has full command routing) |
| Tab / Shift+Tab within Home | Traverse enabled controls in group order; return to the active tab at the ends; disabled features are skipped |
| Arrows / Ctrl+Left or Right | Move spatially within a Home group / focus the next or previous group containing enabled commands |
| Down / Alt+Down on a menu or size field | Open dropdown and focus its first enabled choice; arrows/Tab navigate, Enter/Space applies |
| Escape | Close a menu first; then close temporary ribbon and focus its tab; then return to document. In the size field, discard its draft and return directly to document |
| Command or click outside | Dismiss temporary expansion; preserve model selection for commands. Enter/blur commits size input; Escape cancels |
| F6 / Shift+F6 | Cycle document → visible navigation → visible formatting → status → ribbon, forwards/backwards; never open a hidden pane |
| Resize with Editing focused | Redirect focus to the visible More trigger when Editing moves to overflow |

The temporary ribbon uses the same group geometry and responsive breakpoints as
pinned Home, with a subtle 3-DIP-offset shadow. Its overlay intentionally covers
the canvas below the tabs until dismissed; it does not imply pagination. Focus
and open-menu state live in `src/ribbon_navigation.rs`, separate from document
commands. The native size input keeps cursor/selection/IME behavior. Live document
IME preedit blocks ribbon keyboard entry. Alpha.7 extends this focus router with
KeyTips below; native screen-reader traversal still requires review.

### Alpha.6 native captures

![Temporary Home ribbon, alpha.6, macOS](screenshots/rust-temporary-alpha6-macos.png)

[Tabs-only, 1366 × 900 DIP](screenshots/rust-collapsed-alpha6-macos.png) ·
[Temporary Home, 640 × 480 DIP](screenshots/rust-temporary-alpha6-macos-narrow.png).
Captured and inspected on 2026-10-17, macOS, English/light theme, document zoom
100%, backing scale 2. The same Welcome fixture has unchanged canvas position
between collapsed and temporary captures. The narrow ribbon fits; More remains
visible. This fixture still exposes the existing missing emoji glyph/font-fallback
limitation. These are working program captures, not Word comparisons.

## Actual application capture (alpha.5 history)

![Alpha.5 Font dialog on macOS](screenshots/rust-font-alpha5-macos.png)

Alpha.5, captured 2026-10-16: [Home](screenshots/rust-home-alpha5-macos.png) and
Font dialog at 1366 × 900 DIP / 2732 × 1800 px; minimum-window
[Home](screenshots/rust-home-alpha5-narrow-macos.png) and
[Font](screenshots/rust-font-alpha5-narrow-macos.png) at 640 × 480 DIP / 1280 × 960 px.
All four native renders were inspected. Formatting fixture, English UI, light theme,
Arial, 100% document zoom and macOS backing scale 2. The Font capture selects the
whole fixture and reports mixed size/style/underline without editing it. Home's
caret rests at the final 11-pt italic run. No ribbon controls overlap; all modal
controls fit the minimum window. Find's primary target and its 18-DIP arrow are
separate; the 104-DIP ribbon geometry is otherwise retained from alpha.4.

The development capture helper now waits eight frames and requests an actual
native framebuffer screenshot, allowing modal measurement passes to finish. Its
previous two-pass capture omitted the initial modal; those incomplete images were
replaced. This does not change the production CLI. The native keyboard/save smoke
is recorded in [VALIDATION](VALIDATION.md). No matched Word/Windows capture exists.

Historical alpha.4 captures follow:

![Alpha.4 Home on macOS](screenshots/rust-home-alpha4-macos.png)

Alpha.4, captured 2026-10-16: same Formatting fixture, English UI, light theme,
Arial and 100% document zoom as alpha.3. Native client sizes/backing scale 2:
1366 × 900 DIP (2732 × 1800 px), [960 × 700](screenshots/rust-home-alpha4-compact-macos.png)
(1920 × 1400 px) and [640 × 700](screenshots/rust-home-alpha4-narrow-macos.png)
(1280 × 1400 px). All three renders were inspected. Menus, captions, size input
and font-step buttons remain within Home; narrow documents scroll horizontally.

| Measured Docs implementation change | Alpha.3 | Alpha.4 |
| :--- | :--- | :--- |
| Expanded ribbon height | 112 DIP | 104 DIP |
| Font/Paragraph row origin separation | 36 DIP | 29 DIP |
| Full Clipboard width | 156 DIP | 86 DIP; icon-only Cut/Copy/Format Painter |
| Font size control | Separate 50 + 17 DIP controls, 1-DIP gap | Unified 64-DIP outline |
| Style gallery | Separate AaBbCc cards and small labels | Inline names, one gallery border, stacked navigation |
| Minimum-width font stepping | Not implemented | Grow/Shrink visible at 640 DIP |

These compare Docs revisions, not measured Word deltas. The official theme image
was re-inspected in the browser; it supports the composition changes only.
Alpha.4 also passed a limited native macOS input/dialog smoke; see
[validation](VALIDATION.md). Windows/reference and native DPI checks remain open.

Historical alpha.3 captures follow:

![Alpha.3 character formatting on macOS](screenshots/rust-formatting-macos.png)

Alpha.3, captured 2026-10-16: actual native rendering of
[Formatting.bfdoc](../examples/Formatting.bfdoc), English UI, light theme, 100%
document zoom, Arial, 1366 × 900 DIP / 2732 × 1800 physical pixels (backing scale 2).
The model contains four paragraphs and 21 Unicode words. The first run is bold
28 pt; the Home field reports 28 at its caret. Subsequent runs use 14, 11 and
18.5 pt, including underlined `é · ї · ё`. The same fixture was inspected at
[720 × 700 DIP / 1440 × 1400 physical pixels](screenshots/rust-formatting-narrow-macos.png).
Home retains size/underline controls without overlap; the continuous canvas scrolls
horizontally at this width. No pagination or native Windows result is implied.

The following captures retain the earlier alpha.1 shell checkpoint (before active
size/underline controls), rather than representing the current formatting scope:

![Refined Home on macOS](screenshots/rust-home-refined-macos.png)

Captured from the running native Rust executable using eframe's opt-in screenshot
feature, with `examples/Welcome.bfdoc`, Home expanded, navigation hidden, English UI,
light theme, document zoom 100%. Client size is 1366 × 900 DIP / 2732 × 1800 physical
pixels (macOS backing scale 2), host Arial body font. This is actual model content:
20 Unicode words, four paragraphs, saved state and center alignment on the first
paragraph. It is not a Windows capture or a paired Word comparison. The emoji at
the end renders as a missing glyph on this host; its scalar content survives edits
and native saves. No complete emoji/complex-script rendering support is claimed.

Additional native renders from the same harness:

- [720 × 700 DIP Home / overflow](screenshots/rust-home-narrow-macos.png), same saved fixture, backing scale 2.
- [1366 × 900 DIP File](screenshots/rust-file-macos.png), a real new empty document: zero words, one paragraph, zero characters; no fabricated recent files.
- [Earlier Home capture](screenshots/rust-home-macos.png) is retained as the previous implementation checkpoint.

The capture helper uses the same `DocsApp`, controls and layout as the editor.
To reproduce fixed-size captures (a native desktop session is required):

```sh
BREAKFREE_CAPTURE_TO="$PWD/artifacts/home.png" cargo run --locked --features capture --example capture_ui -- 1366 900 home
BREAKFREE_CAPTURE_TO="$PWD/artifacts/narrow.png" cargo run --locked --features capture --example capture_ui -- 720 700 home
BREAKFREE_CAPTURE_TO="$PWD/artifacts/file.png" cargo run --locked --features capture --example capture_ui -- 1366 900 file
BREAKFREE_CAPTURE_TO="$PWD/artifacts/formatting.png" cargo run --locked --features capture --example capture_ui -- 1366 900 formatting
BREAKFREE_CAPTURE_TO="$PWD/artifacts/formatting-compact.png" cargo run --locked --features capture --example capture_ui -- 960 700 formatting
BREAKFREE_CAPTURE_TO="$PWD/artifacts/formatting-narrow.png" cargo run --locked --features capture --example capture_ui -- 640 700 formatting
BREAKFREE_CAPTURE_TO="$PWD/artifacts/font.png" cargo run --locked --features capture --example capture_ui -- 1366 900 font
BREAKFREE_CAPTURE_TO="$PWD/artifacts/font-narrow.png" cargo run --locked --features capture --example capture_ui -- 640 480 font
```

Create `artifacts` first. On Windows PowerShell, set `$env:BREAKFREE_CAPTURE_TO`
to the absolute PNG path and run the same Cargo arguments. `navigation` is also
accepted in place of `home`. This development example adds no production CLI flags.

## Intentional brand differences

- BreakFree Docs title and approved transparent page/rainbow icon.
- Violet selection/active-tab accents with light neutral chrome; rainbow only in the logo.
- No Microsoft account/sign-in, autosave toggle, sharing or service buttons.
- Honest continuous-layout and save-state labels instead of invented page counts.

## Remaining differences and reproducible comparison gate

Current gaps include print pagination and page rulers, vertical ruler, font-family/color controls, lists/styles, contextual object tabs, KeyTips for future commands,
view-mode buttons, additional File operations and complete native window/accessibility
behavior. Metrics are provisional until measured against the named reference.

For the next review, record Windows edition/build, Word 2024 edition/build, Docs
commit, GPU/font setup, English UI and Light/White theme. Use the same physical
client size (first 1366 × 900, then 1920 × 1080), document zoom 100%, same Unicode
fixture text and installed Arial. Capture each application separately at Windows
100%, 125% and 150%. Record resulting DIP sizes, title/tab/ribbon/status heights,
control bounds, group gaps and clipped labels. Check wide and overflow layouts,
pointer selection, Ctrl shortcuts, Shift navigation, IME commit/cancel, file pickers
and unsaved dialogs. Attach paired images and a delta table; do not invent a score.

Local widget tests exercise synthetic scales 1/1.25/1.5 at 960/1366/1920 physical
widths, verify group bounds and nonoverlap with workspace/status, and collapsed
ribbon. Added 27 width/scale cases at 640/720/899/900/916/1195/1196/1366/1920 DIP and
1/1.25/1.5 verify each Home control stays in its group, controls do not overlap, and
title/search/window controls remain separate. A separate test exaggerates UI font
metrics to prevent the Windows 125% launcher-height regression. Pointer tests cover disabled commands,
selection across formatting/history, new font-step buttons/shortcuts, Paste/Find/
Select menus, Escape cancellation, narrow overflow, zoom/reset and File navigation/
dirty New. All supported font sizes are tested for bounded step behavior. These
are CPU layout tests, not native Windows scaling or visual validation.

The capture helper also accepts `keytips-root`, `keytips-home`, `keytips-prefix`,
`keytips-tabs`, `keytips-view` and `keytips-zoom`:

```sh
BREAKFREE_CAPTURE_TO="$PWD/docs/screenshots/keytips-review.png" \
  cargo run --locked --release --features capture --example capture_ui -- 1366 900 keytips-home
```

These modes inject fixture focus/key events; they do not validate native Alt input.
