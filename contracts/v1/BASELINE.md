# Docs process contract v1 — provisional local snapshot

**Snapshot version:** `docs-process-v1-provisional-2026-10-16`

**Source:** the owner's implementation instruction in the Docs chat, 2026-10-16.

**Office revision inspected:** [`cfaaf419e84d9981acd421d130113ca2265061fd`](https://github.com/PerchAnton/breakfree-office/tree/cfaaf419e84d9981acd421d130113ca2265061fd).

The recursive Office tree, README and roadmap were inspected at this exact commit.
`contracts/v1/`, architectural ADR and the referenced development standard are absent.
**Conformance against a published Office contract has not been performed.** The
owner-provided baseline below is used temporarily, without blocking the editor.
The source commit identifies inspected Office evidence, not an invented upstream
contract revision. No build step downloads Office main.

## Process interface

Executable: `breakfree-docs` (`breakfree-docs.exe` on Windows after pip installation).
No arguments and `--new` open the same editor with a new document.
`--open <absolute_path>` opens that editor with a supported `.bfdoc` document.
Paths with spaces are one quoted argument. Relative paths, conflicting commands,
positional arguments and unknown flags are rejected with exit code 2.

`--capabilities` writes one JSON object and LF to stdout, exits 0 and does not
import Qt, start a GUI or need Office. `--version` prints the installed build's
package version and exits 0. Failed opens print a clear error to stderr and exit 1
before starting the GUI. No documents are modified by argument parsing or reads.

```json
{
  "protocol_version": 1,
  "app_id": "breakfree.docs",
  "app_version": "<actual installed build version>",
  "display_name": "BreakFree Docs",
  "capabilities": ["new", "open"],
  "formats": [
    {"id": "breakfree.docs.native", "extensions": ["bfdoc"], "read": true, "write": true}
  ]
}
```

The example describes an installation with the desktop extra. A core-only
installation advertises `capabilities: []` and reports missing desktop dependencies
if asked to launch a GUI; native read/write support remains available to the model.
Dependency metadata is inspected without importing or initializing Qt.

The capabilities list advertises the available process launch actions. Saving and
formatting are GUI actions, not invented process commands. Content version 1 in
`.bfdoc` is separate from process `protocol_version: 1`. Build version comes only
from package metadata generated from `pyproject.toml`.

## Later Office reconciliation gate

When a published Office contract becomes available, compare its exact tagged or
committed revision against this file, record differences and update tests. Test
launches from Office only after it has a launcher. Office must remain optional:
installation, tests, capabilities and the writing loop work with no neighboring
repositories, network services or accounts.
