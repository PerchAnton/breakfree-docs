# Prototype validation and remaining gates

Evaluation date: **2026-10-16, Europe/Kyiv**. This is local prototype evidence, not
a supported-platform declaration, release, server-CI result or architecture approval.
The environment is macOS 27.0.1 arm64, Python 3.14.3, Qt/PySide6 6.11.1 and regex
2026.9.29. See the project log for checked commits, PR and subsequent CI observations.

## Reproducible checks

From an installed checkout using the README virtual environment:

```sh
QT_QPA_PLATFORM=offscreen .venv/bin/python -m unittest discover -s tests -v
.venv/bin/python -m unittest discover -s tests -p test_gui.py -v
.venv/bin/python -m ruff check src tests tools
.venv/bin/python -m ruff format --check src tests tools
.venv/bin/python tools/check_repository.py
.venv/bin/python -m build
.venv/bin/python tools/check_wheel.py
```

The second command uses the native platform plugin and needs a desktop session.
GUI tests send Qt keyboard/input events to the real editor and use controlled
dialog return values. They do not establish a complete manual native-dialog,
OS-input-method, clipboard-provider or screen-reader review. Offscreen tests run
without a desktop. The isolated-wheel check installs only the wheel and its core
dependency into a fresh temporary environment outside this repository and exercises
the generated executable, Unicode/native roundtrip and packaged artwork without Qt.

| Local check | Result |
| :--- | :--- |
| Complete unittest suite, offscreen | 24 tests passed |
| Native macOS Qt editor tests | 8 tests passed, including reopen-current-file after Save |
| Ruff lint and format | Passed for source, tests and tools |
| Local documentation/snapshot/artwork | Passed; packaged icon is byte-identical to the approved asset |
| Workflow YAML | Parsed locally using Ruby YAML |
| Isolated build | Wheel and sdist built with `python -m build`, pinned setuptools 82.0.1 |
| Isolated installed-wheel smoke | Passed outside the checkout, with regex only and no Qt/Office |
| Application preview | Rendered offscreen from `examples/Welcome.bfdoc`; this is actual application code, not a design mockup |

Server-CI results are recorded separately in PROJECT_LOG after publication.

## D01–D09 evidence map

| ID | Automated evidence | Limits / remaining review |
| :--- | :--- | :--- |
| D01 | Core replace/split/merge, selection, cut/paste, deletion and Qt keyboard replacement; model and projected text compared | Short documents only; drag/drop disabled |
| D02 | Exact English, Ukrainian, Russian, combining accents, non-BMP emoji, ZWJ emoji, flag boundaries, NBSP; scalar/UTF-16 conversion and IME preedit/commit tests | OS IME candidate windows, CJK, RTL, complex-script shaping and all writing systems not certified |
| D03 | Bold/italic and all alignment values supported by the engine; formatting projected and native roundtrip compared | No styles, lists, fonts, page layout or compatibility adapters |
| D04 | Exact immutable states compared across text + formatting undo/redo, branching redo, one-step paste, typed-key boundaries and saved checkpoints | 100-operation history; no time coalescing; no persistent undo |
| D05 | Native save/reopen compares complete model; GUI saves/opens through the same session; isolated installed wheel repeats roundtrip | No migration, autosave or crash recovery |
| D06 | Shared lifecycle and GUI Save/Discard/Cancel paths, cancelled save picker, failed save retaining current session | Automated tests control dialog choices; manual platform dialog/accessibility review pending |
| D07 | Partial-write exception, fsync/replace/chmod failures, missing parent; previous bytes, parsed document, temporary cleanup and dirty checkpoint checked | No killed-process/power-loss/network-filesystem guarantee; see NATIVE_FORMAT.md |
| D08 | Corrupt JSON/UTF-8, foreign type, unknown version/fields, duplicate keys, invalid text/styles, unsupported extension; source bytes unchanged | Bounded parser tests, not a completed fuzzing campaign |
| D09 | Qt keyboard typing/selection/formatting/undo/save/open/cut/paste tests; native macOS checkpoint tested too | Full manual focus traversal, screen readers and native Windows/Linux input pending |

The full D1 milestone remains open until the outstanding target-platform and
maintainer reviews are complete. Checkmarks in ROADMAP identify narrow implemented
and tested tasks, not an adopted stack or completed release gate.

## Integration and independence

`test_integration.py` verifies exact capabilities JSON and installed version, bad
arguments and relative paths, failed opens, clean stdout, and prohibits Qt/Office
imports in a headless subprocess running outside the checkout. GUI tests check
that `--new` and `--open` dispatch the same editor session. The installed console
launcher is also started natively with `--new` for smoke evaluation.

Office contracts/ADR are unavailable at the inspected upstream commit. The
[local v1 baseline](../contracts/v1/BASELINE.md) records that limitation. No actual
Office launcher has been tested. The build/test graph requires no Office, Sheets
or Slides repository, runtime service or account.

## Known limits and next gate

- Python wheel/console launcher only, no bundled executable runtime, installer,
  file association, signing, updater, merge or release.
- Windows/Linux native input and desktop-dialog claims require their own results.
  CI uses Python 3.12 on three OSes. The first implementation commit passed all
  three server jobs, but the initial Linux job skipped 8 GUI cases because its Qt
  runtime import failed. Windows/macOS ran all 24. The workflow now installs Linux
  runtime libraries and requires working Qt imports; import failures cannot silently
  skip desktop tests. Exact later outcomes are recorded in PROJECT_LOG. Offscreen
  CI does not establish native platform support.
- Native save is write-then-replace with tested pre-replacement failures, without
  concurrent-writer protection, recovery, directory fsync or ACL preservation.
- No DOCX, PDF, printing, collaboration, macros, cloud services or AI.
- Model/history/projection are designed for short documents with explicit limits;
  long-document performance and pagination remain unmeasured.
- Computer Use permissions were unavailable during this session; no manual
  accessibility-tree or native file-dialog interaction through that tool is claimed.

Next small gate: review ADR/dependency licensing, run the same installed-wheel and
native keyboard/dialog workflow on Windows, and record IME/focus findings before
adding another document feature. Keep the engine/native regression fixtures stable.
