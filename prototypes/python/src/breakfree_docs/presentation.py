"""UI-independent projection and UTF-16/Unicode scalar index conversion."""

from bisect import bisect_left, bisect_right
from dataclasses import dataclass

from .model import Document, DocumentError, Style, boundaries, runs


def utf16_offsets(text: str) -> tuple[int, ...]:
    offsets = [0]
    for char in text:
        offsets.append(offsets[-1] + (2 if ord(char) > 0xFFFF else 1))
    return tuple(offsets)


def scalar_offset(text: str, position: int) -> int:
    offsets = utf16_offsets(text)
    index = bisect_left(offsets, position)
    if index == len(offsets) or offsets[index] != position:
        raise DocumentError("A UTF-16 position cannot split a Unicode scalar.")
    return index


def snap_selection(text: str, start: int, end: int) -> tuple[int, int]:
    edges = boundaries(text)
    left = edges[max(0, bisect_right(edges, start) - 1)]
    if start == end:
        return left, left
    return left, edges[min(bisect_left(edges, end), len(edges) - 1)]


@dataclass(frozen=True)
class Span:
    start: int
    end: int
    style: Style


def spans(document: Document) -> tuple[Span, ...]:
    result = []
    offset = 0
    for paragraph in document.paragraphs:
        for text, style in runs(paragraph):
            result.append(Span(offset, offset + len(text), style))
            offset += len(text)
        offset += 1
    return tuple(result)
