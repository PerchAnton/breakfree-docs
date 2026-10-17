# ADR 0004 — Local Quick Access Toolbar preferences

- Date: 2026-10-17
- Status: implemented for the owner-requested alpha.10 review
- Scope: local toolbar settings; no document, process protocol, dependency or license change

## Decision

Keep preferences in the GUI-independent Rust `preferences` module. The GUI adapter
maps its typed commands to real file/history actions. Read once when a desktop
window starts; save each command-visibility or placement change immediately. The
ordered list preserves numeric KeyTips, including unavailable Undo/Redo slots.
Newly enabled commands append. An empty toolbar is valid. Explicit move controls
are a later increment. Do not store document text, paths, selection or undo history
in this file, and never put settings into `.bfdoc` documents.

All documents opened by this user/profile share the saved defaults. Existing
windows keep their own toolbar; there is no live cross-window synchronization.
Opening a missing profile uses Save/Undo/Redo above the ribbon without creating
directories or writing a file. Only a user change or confirmed Reset writes.
`--version`, `--capabilities` and rejected arguments/files exit before preferences
or a graphical context are opened. Tests and capture previews use memory or an
explicit temporary profile, never the user's actual profile.

## Paths and schema

| Platform | Settings file |
| :--- | :--- |
| Windows | `%APPDATA%\BreakFree\Docs\settings.json` |
| macOS | `$HOME/Library/Application Support/BreakFree/Docs/settings.json` |
| Linux | `$XDG_CONFIG_HOME/breakfree-docs/settings.json`, if XDG_CONFIG_HOME is absolute; otherwise `$HOME/.config/breakfree-docs/settings.json` |

`BREAKFREE_DOCS_CONFIG_DIR` overrides the directory on every platform and must be
absolute. A missing/invalid profile variable produces an explicit error; never
fall back to the working directory. A relative Linux XDG_CONFIG_HOME is ignored.

```json
{
  "format": "breakfree.docs.settings",
  "settings_version": 1,
  "quick_access": {
    "commands": ["save", "undo", "redo"],
    "placement": "above"
  }
}
```

Allowed commands: `new`, `open`, `save`, `undo`, `redo`, at most once each. Placement
is `above` or `below`. Bound reads to 64 KiB. Strict typed parsing rejects malformed
UTF-8/JSON, missing/duplicate/unknown fields, foreign format, unsupported settings
version, unknown commands/placement and repeated commands. Settings version 1 is
independent of app alpha.10, native document content v2 and process protocol v1.
Future migrations require an explicit decision; there is no silent lossy fallback.

## Failure handling and reset

On load failure use the default toolbar, show the reason/path and keep a persistent
warning in customization. Bad, future or unreadable input is never rewritten by
an ordinary change. On save failure the changed toolbar still works in this window;
the previous file stays intact in the tested failures. Retry Saving retries the
current toolbar after a transient problem has been fixed. Restart after an external
file change to load its settings. The document remains editable after dismissing
the error; settings never affect its dirty flag or history.

Reset Toolbar opens a confirmation. Cancel/Escape leaves the toolbar/file intact.
Reset restores Save/Undo/Redo above the ribbon and first keeps any readable original
bytes in a unique sibling `toolbar-settings-backup-*.json`, including invalid or
future input. The backup is flushed and retained before replacement. It is not
automatically deleted. If backup or replacement fails, show the error and retain
the previous file; defaults apply only to this window. Retry Saving (or a subsequent
customization) retains the confirmed reset's backup requirement until a write succeeds. Oversized/unreadable files,
directories and symbolic links are refused even on reset. Move them aside manually
with Docs closed if necessary. This is preferences backup, not document recovery.

## Write semantics and limits

1. Validate the new toolbar; create the parent directory if needed.
2. Acquire a cooperating-writer lock with `create_new` on `.toolbar-settings.lock`.
3. Read the current regular file under the size bound. For ordinary saves, compare
   its exact bytes with this window's observed bytes, then validate its schema.
   Refuse stale updates rather than replacing a newer window's settings.
4. Write a unique sibling temporary file, flush and sync it, preserving existing
   file permissions. Refuse read-only settings. Reset also prepares its backup.
5. Replace using `tempfile::NamedTempFile::persist`; advance observed bytes only
   on success. Close and remove the lock on normal return or unwinding.

A process crash can leave the lock. Close all Docs windows before manually removing
that lock; there is no automatic stale-lock recovery. The lock coordinates Docs
writers only. No guarantee covers a noncooperating writer racing the final replace,
directory fsync/power loss, network filesystems, ACLs or extended attributes. No
atomicity or crash-recovery claim is made beyond the explicitly tested failure
preservation. The bounded small local file is written synchronously; remote/slow
profile storage has not been performance-tested.

## Evidence

Core tests cover paths, read-only open, ordered/empty roundtrips, invalid inputs,
oversize/non-file paths, lock/stale-window conflicts, partial write/sync/replace and
backup failures, permissions/retry and Unix link rejection. Widget tests cover
restart, numeric hints, reset confirmation/keyboard/modal bounds, error/retry and
unchanged document selection/history. CLI subprocess tests prove information and
invalid-argument paths leave missing or corrupt profiles untouched. Native macOS
restart/error/retry/reset/backup and Unicode save/reopen evidence is recorded in
[VALIDATION](../VALIDATION.md). Native Windows and the named Word 2024 comparison
remain separate acceptance gates.
