# ADR 0017: Bounded, fail-closed DOCX interchange

Date: 2026-10-24
Status: Initial profile implemented; Save policy superseded by [ADR 0018](0018-primary-docx-and-numbering.md); broader interchange remains incremental

## Context

D4 introduces local DOCX exchange after native structure and pagination. The
native model has stricter semantics and fewer object types than OOXML. A generic
XML-to-text import would silently remove unsupported content and style behavior.

## Decision

Keep `src/docx` separate from the native versioned storage and GUI. Use the locked
quick-xml parser, flate2 compression and crc32fast checksum implementations already
present in the dependency graph. The original package adapter bounds entry sizes,
checks ZIP headers/checksums/descriptors, rejects ambiguous paths and never
extracts parts to disk. The namespace-aware XML adapter forbids DTDs, custom
entities and unsupported nodes. There are no executable macros or external fetches.

The first profile maps paragraphs, runs, named style graphs, exact spacing/indents
and one page section to standard WordprocessingML, with no proprietary sidecar.
A declared subset is enforced symmetrically; incompatible style-toggle compositions
are rejected. Import builds and validates a complete candidate before replacing
any session. Unknown content is reported as an error with source part/property.
Export validates before using the existing checked replacement writer.

DOCX imports have no native save destination. Save creates `.bfdoc`; Export DOCX
is explicit and leaves history/dirty state/destination unchanged. The CLI's existing
`--open` uses the same dispatcher; no new launch protocol command is invented.

## Consequences

Most external Word files are outside this small initial profile. Rejecting them
is intentional, but means D4 is not complete. Add features only with independent
fixtures, preservation tests, failure tests and named external-application checks.
No broad interoperability, identical pagination or release-readiness claim follows
from one file opening. See [the exact profile](../DOCX_SUBSET.md).

## Standards consulted

- [ECMA-376, Office Open XML](https://ecma-international.org/publications-and-standards/standards/ecma-376/): WordprocessingML and OPC package structure.
- [Microsoft: structure of a WordprocessingML document](https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document): document/body/paragraph/run/text and separate package parts.

These are format references, not copied implementation code. The project license
and application identity remain unchanged.
