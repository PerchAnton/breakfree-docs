# Native editor UI specification

Owner-approved reference: **Microsoft Word 2024 desktop for Windows**, classic
expanded multi-row ribbon, English UI and light theme. Rust is mandatory; the
library does not determine the product layout. This specification is an initial
measurable implementation baseline, not a claim of pixel equivalence.

## Reference evidence and limits

Checked 2026-10-16. [Microsoft navigation documentation](https://support.microsoft.com/en-us/accessibility/word/use-a-screen-reader-to-explore-and-navigate-word)
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
| Ribbon | 112 DIP expanded; white rounded inset (8 DIP horizontally / 4 vertically), 96-DIP groups with bottom captions; collapsible via button, double-click tab or Ctrl+F1 |
| Home groups, full | Clipboard 156, Font 278, Paragraph 256, Styles 300, Editing 116 DIP; 8-DIP group gaps and separators |
| Home groups, compact | Below 1180 DIP available ribbon width: 86/218/216/120/104 DIP; compact Clipboard and one style preview |
| Home overflow | Below 900 DIP available ribbon width: 82/166/164 DIP; Styles/Editing move into More menu; document never lies under the ribbon |
| Controls | Small buttons 24–32 × 24–26 DIP; Paste 48 × 72; two rows in Font/Paragraph, three in Clipboard/Editing; 13-DIP UI text, 11-DIP captions; 15 × 16 group launchers |
| Workspace | Neutral #e8e8e8; white continuous canvas, border #c5c5c5; soft canvas shadow; 10-DIP workspace top gap; centered when space allows; horizontal/vertical scrolling when required |
| Canvas | 816 DIP wide at 100% document zoom, 96-DIP side/top margins, content-driven height; **no page segmentation** |
| Text | Host Arial/DejaVu or embedded fallback; starts at 11 pt (14.667 DIP), 1.25 line height, 0.5-em paragraph gap; model emphasis/alignment projected into hit-testable text |
| Ruler | 22-DIP-high horizontal canvas ruler with shaded margins, 96 DIP per inch, eighth-inch ticks; zoom scales with document; no draggable margins or vertical/page ruler yet |
| Panes | Navigation left 240 DIP (195–320); Formatting right 220 DIP (195–300); visible on request |
| Status | 24 DIP; real Unicode word count and paragraph count, saved state, continuous-layout label and 50–200% zoom; ±10% buttons and click percentage to reset to 100% |
| File | 180-DIP left action rail and scrollable Home/Options body; 158 × 160 blank-document tile, real current-file properties; New/Open/Save/Save As active, Print disabled with explanation |

## Commands and interaction

Active Home commands are plain-text clipboard, bold/italic, four alignments, Find,
Replace selected/next match and Select All. Bold/italic show selected state and set
pending insertion style at an empty selection. Original vector strokes draw clipboard, file, history, search, paragraph, window and
launcher controls. Button names are exposed to the toolkit accessibility tree.
Selected formatting has a pale violet fill even without hover; focus uses an outline.
Font/size fields report the current display values and explain their read-only state.
Style previews remain disabled and do not claim applied model styles.
Native egui hover/pressed/disabled/focus states remain active; unsupported controls
have tooltips explaining the missing capability. Styling uses restrained violet
#7047ae, pale selected-state fill, light chrome and readable dark text.

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
F6 cycles document/command-search/navigation, Alt+Q searches real commands. Standard
Ctrl/Command N/O/S/Shift+S, B/I, Z/Y/Shift+Z, F/H and L/E/R/J are implemented.
Full sequential ribbon KeyTips, complete Word F6 traversal, native assistive technology
and multilingual OS IME behavior remain unverified/incomplete.

## Actual application capture

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
EFRAME_SCREENSHOT_TO="$PWD/artifacts/home.png" cargo run --locked --features capture --example capture_ui -- 1366 900 home
EFRAME_SCREENSHOT_TO="$PWD/artifacts/narrow.png" cargo run --locked --features capture --example capture_ui -- 720 700 home
EFRAME_SCREENSHOT_TO="$PWD/artifacts/file.png" cargo run --locked --features capture --example capture_ui -- 1366 900 file
```

Create `artifacts` first. On Windows PowerShell, set `$env:EFRAME_SCREENSHOT_TO`
to the absolute PNG path and run the same Cargo arguments. `navigation` is also
accepted in place of `home`. This development example adds no production CLI flags.

## Intentional brand differences

- BreakFree Docs title and approved transparent page/rainbow icon.
- Violet selection/active-tab accents with light neutral chrome; rainbow only in the logo.
- No Microsoft account/sign-in, autosave toggle, sharing or service buttons.
- Honest continuous-layout and save-state labels instead of invented page counts.

## Remaining differences and reproducible comparison gate

Current gaps include print pagination and page rulers, vertical ruler, font/color/size controls, lists/styles, contextual object tabs, full KeyTips,
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
selection across formatting/history, zoom/reset and File navigation/dirty New. These
are CPU layout tests, not native Windows scaling or visual validation.
