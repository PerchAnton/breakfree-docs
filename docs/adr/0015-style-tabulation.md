# ADR 0015 — Inherited paragraph tabulation

Date: 2026-10-24. Status: implemented for review in alpha.86.
Extends ADR 0007 and ADR 0014; no dependency, license or process-contract change.

## Resolution and representation

A paragraph style has two independent optional properties:
`tab_interval_half_points` (1–288) and `tab_stops` (up to 32 sorted unique stops).
A style stop contains `position_half_points` (1–2880), `kind` (left/center/right)
and optional `leader` (dots/dashes/line). A missing/null list inherits; an explicit
empty list suppresses ancestor positions. The nearest supplying ancestor wins for
each property independently. Root defaults remain a 72-half-point interval and
no explicit stops. Positions, types and leaders inherit as one complete list;
there is no positional merge, tombstone or per-stop inheritance in this increment.
Existing bounded traversal and whole-catalog cycle/ID validation remain mandatory.

Paragraph direct intervals override the style interval. Nonempty paragraph stop
arrays override the complete style list. Empty arrays inherit unless
`clear_tab_stops: true` explicitly suppresses the style list. The flag only affects
empty lists; it does not suppress authored nonempty arrays. Retaining the existing
paragraph arrays avoids converting every editing/persistence path at once. The
resolved tabulation view is computed separately from other paragraph geometry and
is never saved as a cache. Layout, dialogs and ruler use this same view.

## Editing and history

Modify/Create Style owns a local definition draft. Tab interval and Tab positions
open nested drafts; their OK changes only the definition draft. The outer OK
commits all style changes once. Nested Cancel/Escape returns to its entry button;
outer Cancel/Escape discards the complete draft. No inherited values are flattened
when inspecting or cancelling. The existing parent selector also governs tabs.

The paragraph interval dialog can clear its override to inherit. Use inherited
positions clears direct stop arrays and the explicit-empty flag, retaining the
independent interval. Clear all creates an explicit empty override when it removes
positions. Existing one-transaction commands and half-open selection rules apply.
Ruler movement, creation, deletion and leader edits resolve each selected paragraph
first. Only a changed owner materializes its resulting complete list; unaffected
owners and no-op previews preserve inheritance and redo. Undo restores authored
values and references, not just equivalent rendered positions. Stale drafts fail.

## Storage and compatibility

Native v18 adds style interval/list fields and the paragraph empty-list flag.
Versions 1–17 reject the new fields even when null or empty; typed decoding still
rejects duplicates, unknown fields, invalid values and foreign types. Existing
v1–v17 files retain their direct lists and intervals. Absent interval/empty arrays
now inherit, but legacy catalogs have no tabulation properties, so opening keeps
their previous effective defaults and layout. A subsequent explicit style edit can
change those formerly default paragraphs. We do not infer direct overrides from
legacy rendered defaults. Opening never rewrites the source; saving writes v18.
Alpha.85 and older reject v18. Use Save As to retain an older compatible copy.

Windows and VoiceOver acceptance remain deferred. Decimal tabs, merging individual
ancestor stops, full pagination remain separate work.

## Read-only provenance (alpha.88)

The tab_sources module reports authored sources independently for interval and
complete stop lists. Direct values, assigned styles, ancestors and defaults remain
distinct even when their values match. A stop filter includes only actual owners
and counts absent paragraphs separately. Selection uses the existing half-open
paragraph rule. Group identity uses stable style IDs, not display names.

Formatting and marker descriptions read current model values. Optional Sources
inspection in the paragraph dialogs records the values at dialog entry; it is
labeled accordingly and does not imply that uncommitted text has been applied.
Nested style dialogs inspect the current style draft, without a paragraph direct
override. Changing a parent and reopening a nested dialog recomputes its sources.
Ruler preview inspection reads the original live document, so materialized preview
overrides are not mistaken for the authored source. Close/Escape restores the
previous field/control focus; the numeric cursor and draft survive inspection.
The modal list scrolls with the wheel or keyboard. No inspection changes history,
persistence or the native content version. Windows and speech review stay deferred.
