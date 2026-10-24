# ADR 0016: Structured blocks and shared page placement

Date: 2026-10-24. Status: reversible implementation for review (alpha.89).
The owner authorized D2/D3 implementation; this does not approve a release or
close native Windows, VoiceOver or physical-printer acceptance.

## Decision

The Rust model owns list metadata, grapheme-aligned hyperlinks, atomic table/image
paragraphs, paragraph flow rules and page sections. Commands validate a complete
candidate before one history commit. GUI dialogs are cancellable drafts; they
reject stale document snapshots. Enter continues lists; an empty list paragraph
exits the list. Page Break splits at the caret. Links allow http, https and mailto
only, and open externally only following explicit activation.

Tables contain rectangular plain-text cells and widths in document points. Cells
are edited in a native table dialog; rows/columns can be inserted/removed. Tables
and PNG pictures occupy separate paragraphs, so text editing cannot accidentally
merge an object marker with text. Copy exports cells as tab/newline text and image
alternative text. Pictures have explicit size and alternative text; no floating
wrap/anchor semantics are implied. PNG framing/checksums are checked by the model;
pixel decoding is checked on import and object commands using the already locked
image dependency (now also available without desktop features).

Native v19 adds required page and paragraph structure fields to the run-based
format. Versions 1–18 read with default structure, but reject v19 fields. New saves
require v19. Application/protocol/settings versions remain independent.

A toolkit-independent paginator consumes measured rows in points. A replaceable
Rust/egui adapter shapes the same content for editable Print Layout, preview and
output. Sections start on new pages and may change page size/margins/header/footer
and restart numbers. Explicit breaks override keep-with-next. Keep chains move
when they fit on one page; oversized chains may split. Keep-lines paragraphs that
cannot fit fail visibly. Text avoids isolated first/last rows when physically
possible. Tables split between rows and optionally repeat their first row; images
stay whole. Unfit rows/images fail visibly rather than disappear. Maximum 500 pages.

The editor retains one logical text projection and its existing native TextEdit
input/IME path; page placement changes glyph geometry, not the document source.
Table cells remain separately edited plain text. Continuous view remains available.
Heading navigation follows Heading 1 and its style descendants.

PDF 1.4 export is original Rust code using the shared page paint output at 144 dpi.
It contains raster page images, not searchable text or tagged accessible content.
It embeds no font programs. Existing installed-font preference and bundled fallback
apply; identical output on machines with different installed fonts is not promised.
No additional font assets or license changes are introduced. The current output
limit is 128 MiB. Native and PDF saving share failure-preserving write/replace code;
no crash recovery, power-loss durability or general filesystem atomicity is claimed.

The macOS/Linux CUPS adapter discovers queues on request and submits only after an
explicit Print document action. Arguments bypass the shell; submission success is
not paper-output evidence. Windows uses PDF export pending a native adapter.
Physical printing is deferred by the owner; no validation job was submitted.

## Limits and follow-up

No merged/rich-text cells, floating images, multiple columns, continuous section
breaks, outline levels beyond Heading 1 ancestry, bookmarks, contextual table
ribbon, general RTL/CJK claim, searchable/tagged PDF or font embedding. These are
explicit subsequent scope, not silently converted features. Existing unsupported
DOCX/PDF imports remain rejected. UI/OS input and output fixtures are recorded in
VALIDATION.md separately from model tests and server CI.

## References

- [CUPS printing options](https://www.cups.org/doc/options.html)
- [CUPS lp command](https://www.cups.org/doc/man-lp.html)
- [PDF reference](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.6.pdf)
