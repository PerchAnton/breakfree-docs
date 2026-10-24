# ADR 0018: Primary DOCX persistence and bounded list interchange

Date: 2026-10-24
Status: Implemented; primary-format choice explicitly approved by the owner

## Decision

DOCX is the default for new documents and the first file-dialog format. Opening a
supported DOCX retains its path; Save writes it through the checked replacement
writer. `.bfdoc` remains an optional format that stores every current model feature.
Opening an existing `.bfdoc` retains that explicit destination. Save As permits
conversion in either direction. Binary `.doc`, macro-enabled `.docm`, templates and
arbitrary OOXML content are not supported by this format-priority decision.

This supersedes ADR 0017's forced-native Save policy. Before DOCX writing, validate
the model, encode a standard package, decode the candidate and require exact model
equality. Unsupported content or a failed write leaves the prior destination,
path, dirty checkpoint and undo history intact. Never silently fall back to
`.bfdoc`; the user explicitly selects it when the DOCX subset cannot represent a
feature. DOCX is primary, but it does not yet cover every feature of the editor.

A loaded DOCX retains its original bytes in the session. Saving to that source
checks for external changes (including canonical path aliases); conflicts request
Save As. This is a best-effort pre-write check, not cross-process atomic compare-
and-swap or crash recovery. No new native content version is introduced.

## Import and list model

OPC relationships locate the main document, styles and numbering. Safe relative
and package-absolute internal paths are resolved; all package parts/content types
remain accounted for. Styles are optional; unused built-in definitions need not
be present. Document defaults are parsed semantically, independent of property
order. Different supported defaults become an explicit root paragraph style,
retaining inheritance rather than flattening each paragraph. Unspecified imported
properties use the documented baseline; unsupported fonts/themes still fail.

Read/write decimal and bullet lists with levels 0–8 and explicit restarts. Lists
remain model metadata, not literal prefixes in editable text. Each active native
level exports its own numbering stream; ancestor resets, breaks, kind changes and
explicit restarts create new instances. This prevents OOXML's implicit level-reset
rules from changing the model's sequence. Reading tracks instance counters and
rejects linked/discontinuous or reset rules the sequential model cannot preserve.

The initial geometry is explicit: 24 pt inset plus 18 pt per level, 18 pt hanging,
with default paragraph indents/font properties. Other geometries, compound/Roman/
letter labels, picture bullets, arbitrary level overrides and style-based numPr
remain errors. See [the profile](../DOCX_SUBSET.md) for precise restrictions.

## Evidence and references

Tests cover an independently authored package with relocated parts/no styles,
external defaults, native/DOCX list round trips, restart editing and undo/redo,
unsupported numbering, save checkpoint/history behavior, conflicts and native
fallback. External rendering and native dialog checks are reported separately in
[validation](../VALIDATION.md#alpha91-primary-docx-and-list-validation).

- [OOXML level restart rules](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.levelrestart?view=openxml-3.0.1).
- [OOXML numbering start overrides](https://learn.microsoft.com/en-us/dotnet/api/documentformat.openxml.wordprocessing.startoverridenumberingvalue?view=openxml-3.0.1).

No project license, dependency, brand, Office process command or release-stage
change accompanies this decision.

## Alpha.92 extension: hyperlink ranges

Map supported external hyperlink relationships to existing model-owned ranges.
Runs split at both character-format and link boundaries; adjacent links remain
separate, and reused imported relationship IDs keep their targets. Validate
grapheme boundaries through the model before accepting the document. No imported
link is fetched or opened. Unknown link metadata is rejected rather than discarded;
this adds no model version, dependency, UI redesign or native format change.

OOXML reference: [WordprocessingML hyperlink example](https://download.microsoft.com/download/e/1/4/e14fb96f-83b8-4a2a-84db-7fa8acbe061a/Office%20Open%20XML%20Part%201%20-%20Fundamentals.pdf).
The model does not store history/tooltip/anchor/frame properties, so those remain
outside this bounded profile.
