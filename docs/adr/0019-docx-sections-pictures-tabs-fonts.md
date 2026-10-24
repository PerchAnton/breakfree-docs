# ADR 0019: Extended bounded DOCX exchange

Date: 2026-10-24
Status: Implemented prototype; external save-back and release acceptance open

Extend ADR 0018 using the existing native model and shared commands. DOCX remains
primary. Standard internal relationships carry PNG, headers and footers. A section
break belongs to the preceding OOXML paragraph, whereas the model stores new
section settings on its first paragraph; the adapter translates that boundary.
Only next-page sections are represented. Plain single-line headers/footers use
installed Arial 9 pt and a PAGE field for the number; restart settings belong to
sectPr. Missing later header/footer references inherit the preceding section.
Explicit export parts prevent accidental inheritance of intentionally empty text.

PNG remains an atomic object with exact pixels, whole-point size and alternative
text. Reject external/floating/cropped/transformed content and mixed object runs.
The bounded plain table profile remains; section-dependent table width is checked
against the active section, not always the first page.

Explicit left/center/right positions and dot/dash/line leaders retain style/direct
replace-list semantics. Emit clear records for inherited positions removed by an
override; reject external additive/partial lists that cannot retain that authored
meaning. Empty lists and inherited lists stay distinct. Non-default periodic
tab grids have no direct per-paragraph OOXML equivalent and remain rejected.

Native v20 adds optional Arial/Times New Roman/Courier New family overrides and
bounded core/app package-property XML. The same family cascade drives editing,
layout, previews, and persistence. Host font programs are not embedded; missing
families fall back locally. Original metadata is retained verbatim through edits,
not falsely regenerated: imported word/page counters may therefore be stale.
The GUI reports model statistics independently. Unknown rendering-related parts,
themes, settings and custom properties remain explicit errors.

Freeze v1–v19: reject new fields there, including null values. Their absent family
uses the prior Arial/default baseline. V20 keeps all earlier structure/tab fields.
No new dependency or project license is adopted. A local review archive collects
locked dependency notices and provenance; it is not authorization to distribute.

References:
- [Inline picture structure](https://learn.microsoft.com/en-us/office/open-xml/word/how-to-insert-a-picture-into-a-word-processing-document).
- [Header inheritance in Word](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/5de15e59-3ec1-4f80-aa4d-00bdd447b86f).
- [Current exact subset and limits](../DOCX_SUBSET.md).
- [Validation and remaining acceptance gates](../VALIDATION.md).


## Alpha.95 extension

Preserve a standard custom-properties package part as passive bounded XML beside
core/app metadata; do not interpret typed values or add an editing surface.
Native v21 permits the new key and explicitly freezes v20. PAGE field cached
decimal display text is derived and normalized for profile comparison; actual
section restart/continuation settings remain unchanged.

Reference: [Custom property structure](https://learn.microsoft.com/en-us/office/open-xml/word/how-to-set-a-custom-property-in-a-word-processing-document).
