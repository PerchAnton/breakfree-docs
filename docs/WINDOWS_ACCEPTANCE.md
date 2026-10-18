# Windows style-navigation acceptance

Status for alpha.40: **not executed on a Windows desktop**. No suitable Windows
machine was available in this session. CPU widget tests on a Windows runner and
synthetic scale factors do not establish OS keyboard routing, native DPI,
physical Alt/AltGr, IME, accessibility or visual equivalence.

## Obtain the exact review build

Use the Windows review artifact from a successful Actions run for the intended
PR commit. It is a temporary test artifact, not an installer or release. Extract
all files together. `windows-build.json` records the source SHA, version and EXE
SHA-256; a PR run may use GitHub's merge-test SHA. Compare these to the run and
`Get-FileHash .\target\release\breakfree-docs.exe -Algorithm SHA256`.

Alternatively build the review checkout with the Rust MSVC toolchain and Visual
Studio C++ Build Tools described in README:

```powershell
cargo build --locked --release
cargo test --locked dependency_keyboard_navigation_scale_matrix --lib
.\target\release\breakfree-docs.exe --version
.\target\release\breakfree-docs.exe --capabilities
Copy-Item .\examples\StyleNavigation.bfdoc "$env:TEMP\Docs-navigation.bfdoc"
$document = (Resolve-Path "$env:TEMP\Docs-navigation.bfdoc").Path
.\target\release\breakfree-docs.exe --open $document
```

The extracted artifact also supports the last five commands without a Rust
installation. Work on the copied fixture, never the original. Record the initial
copy's hash. The fixture contains a heading, Unicode text and a 32-level custom
character-style chain: `Chain 01` and `Chain 17` are directly assigned; the final
parent is `Parent Україна Русский é`, based on Strong.

## Repeat at each OS scale

Record Windows edition/build, keyboard layout, monitor resolution, display scale,
app version/SHA, session type (local/RDP) and window client dimensions. Use a light
theme, English app UI and 100% document zoom. Change **Windows display scaling**
to 100%, 125%, then 150%; restart the editor at each scale. Record unavailable
scales as untested. Do not substitute document zoom or screenshot resizing.
Restore the previous display setting afterwards.

For each scale use a wide window (1366 × 900 DIP when it fits) and the minimum
640 × 480 DIP window, with both expanded and collapsed Home:

1. Tap Alt, then H. Repeat using F10, H. Check KeyTips and focus. Open Styles with
   L in wide Home, or Editing/More with E when Styles is in overflow.
2. Press F3, type `Normal`, enable Used in document. Press Down to enter rows.
   Use Tab/Shift+Tab and Up/Down to reach Required by → Heading 1. Enter follows
   the link without applying the style; the query clears, used-only stays on,
   Heading 1 is focused and scrolled into view, Back to: Normal appears.
3. Reach Back with Up/Down and activate with Space. Check restored query/used-only
   filters and Normal row focus. Repeat by pointer. Neither route changes text,
   selection formatting, dirty state or Undo history.
4. Reset filters. Search the long parent name and use Required by → Chain 17,
   then its Required by → Chain 01. Back must return to the original parent and
   original filters. Check that all row text/controls remain reachable through
   scrolling; long labels must not push the popup outside the window.
5. Escape closes the menu; return to document focus. Reopen: no stale Back state.
   Repeat after resizing and after changing display scale with the menu open.
   Record focus loss, clipped controls, wrong-target activation or stuck KeyTips.
6. After navigation only, close and reopen the copy: no save prompt is expected,
   and the file hash must be unchanged. Separately edit a word, use Ctrl+Z/Ctrl+Y,
   save the copy and reopen to check the editing path still works.
7. Check physical AltGr and Ukrainian/Russian layout input in the document and
   filter; it must not trigger unrelated ribbon actions. Record IME separately.

Capture the whole app at each scale with the menu, destination focus and Back
visible. Do not include unrelated applications or private documents. For any
Word comparison, record matching window/theme/language/zoom settings separately;
no similarity percentage is inferred from these checks.

## Evidence to record

| Scale | Wide / minimum | Expanded / collapsed | Keyboard / pointer / focus | Clipping | File/history | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| 100% | Not run | Not run | Not run | Not run | Not run | Pending Windows access |
| 125% | Not run | Not run | Not run | Not run | Not run | Pending Windows access |
| 150% | Not run | Not run | Not run | Not run | Not run | Pending Windows access |

Record pass/fail per case, screenshot paths and exact reproduction steps. Keep
CI evidence separate from this native acceptance table. Never mark a case passed
because the executable built or a CPU test succeeded.
