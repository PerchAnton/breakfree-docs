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
| Ribbon | 104 DIP expanded; white rounded inset (8 DIP horizontally / 4 vertically), 88-DIP groups with bottom captions; collapsible via button, double-click tab or Ctrl+F1 |
| Home groups, full | Clipboard 86, Font 304, Paragraph 246, Styles 350, Editing 116 DIP; 8-DIP group gaps and separators |
| Home groups, compact | Below 1180 DIP available ribbon width: 82/246/218/140/104 DIP; compact Clipboard and one style preview |
| Home overflow | Below 900 DIP available ribbon width: 82/232/206 DIP; Styles/Editing move into More menu; the pinned ribbon reserves workspace; temporary mode overlays it |
| Controls | Small buttons 24–32 × 24–26 DIP; Paste 46 × 69 split into a 37-DIP icon action and 32-DIP options menu; two rows in Font/Paragraph, three in Clipboard/Editing; 13-DIP UI text, 11-DIP captions; 15 × 16 group launchers |
| Font size | 64 × 25 DIP unified outline: 46-DIP editable field and 18-DIP preset arrow; font-family field is 112/64/50 DIP in full/compact/overflow density |
| Workspace | Neutral #e8e8e8; white continuous canvas, border #c5c5c5; soft canvas shadow; 10-DIP workspace top gap; centered when space allows; horizontal/vertical scrolling when required |
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
Home/File KeyTips and Alt entry are described below. Complete non-Home KeyTips,
native assistive technology and multilingual OS IME remain unverified/incomplete.

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
need real-device acceptance. Quick-access, Draw/Help and non-Home command hints are
not complete; no full reference-app keyboard equivalence is claimed.

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

Current gaps include print pagination and page rulers, vertical ruler, font-family/color controls, lists/styles, contextual object tabs, complete non-Home KeyTips,
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

The capture helper also accepts `keytips-root`, `keytips-home` and `keytips-prefix`:

```sh
BREAKFREE_CAPTURE_TO="$PWD/docs/screenshots/keytips-review.png" \
  cargo run --locked --release --features capture --example capture_ui -- 1366 900 keytips-home
```

These modes inject fixture focus/key events; they do not validate native Alt input.
