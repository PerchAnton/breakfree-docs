# ADR 0011 — Character-style precedence and semantic runs

Date: 2026-10-18. Status: implemented prototype policy for owner review.
Depends on ADR 0006–0010. Rust and the existing native GUI remain unchanged.

## Decision

Resolve formatting in this order: paragraph catalog inheritance, one optional
character style, then direct overrides. Explicit false overrides true at either
style layer. Strong sets bold; Emphasis sets italic; both inherit every other
property. No style means Default paragraph font, without clearing direct overrides.
These first built-ins are immutable; custom definitions/parents are a later step.

Store the stable enum reference alongside per-scalar direct properties in the
GUI-independent model. `CharacterFormat::resolve` is the common cascade used by
commands, selected toolbar state and layout. Never infer references from visual
appearance. Reject character references inside paragraph definitions.

Applying/removing a style on a grapheme-aligned range is one shared command and
one history operation. Preserve text, direct overrides, paragraph IDs/alignment,
selection and other ranges. Mixed selections have no single selected style.
At a caret, set pending formatting in one undoable session operation without
changing document dirty state. Typing and Enter retain this pending reference;
moving the caret follows the existing insertion-format policy. Repeating the same
choice does not clear redo. Clear character formatting removes direct properties
but preserves the named character and paragraph styles. Remove the named style
separately with Default paragraph font.

Native v7 stores `character_style` beside each run's `direct` object. Strict legacy
readers remain separate; missing/null v7 references and all older imports mean no
character style. Save writes v7; alpha.23 and earlier reject it. Use Save As to keep
an older copy. Unknown/duplicate fields and unknown IDs are rejected before any
session replacement. Existing checked temporary-file saves retain their guarantees.

## UI and limits

Home Styles offers the three choices in its scrollable list, also reachable from
command search and KeyTips (B/I/Z within wide Styles or narrow Editing overflow).
Strong/Emphasis labels preview their property. Existing light palette, original
icons, gutters and disabled-command policy are preserved. Physical Windows/Word
behavior matching is not established by these product rules or CPU widget tests.

Custom character definitions, inheritance between character styles, linked styles,
font family/color and full complex-script support remain out of scope. Validation
must distinguish model/widget tests, native macOS input and Windows acceptance.
