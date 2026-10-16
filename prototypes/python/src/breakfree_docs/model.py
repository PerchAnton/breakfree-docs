"""Immutable document values and Unicode index rules; no UI or storage dependencies."""

from dataclasses import dataclass

import regex

MAX_CHARACTERS = 100_000
ALIGNMENTS = ("left", "center", "right", "justify")


class DocumentError(ValueError):
    """An edit or document violates a supported invariant."""


def validate_text(text: str, *, paragraphs: bool = True) -> None:
    if not isinstance(text, str) or len(text) > MAX_CHARACTERS:
        raise DocumentError(f"Text must contain at most {MAX_CHARACTERS:,} characters.")
    for char in text:
        code = ord(char)
        if (
            (code < 32 and char not in ("\t", "\n"))
            or code == 127
            or 0xD800 <= code <= 0xDFFF
            or char in ("\u2028", "\u2029")
            or (char == "\n" and not paragraphs)
        ):
            raise DocumentError(f"Unsupported control or text separator U+{code:04X}.")


def boundaries(text: str) -> tuple[int, ...]:
    """Return extended grapheme boundaries as Python Unicode scalar offsets."""
    return (0, *(match.end() for match in regex.finditer(r"\X", text)))


def validate_range(text: str, start: int, end: int) -> None:
    if type(start) is not int or type(end) is not int or not 0 <= start <= end <= len(text):
        raise DocumentError("Selection is outside the document.")
    edges = boundaries(text)
    if start not in edges or end not in edges:
        raise DocumentError("An edit cannot split a Unicode grapheme cluster.")


@dataclass(frozen=True)
class Style:
    bold: bool = False
    italic: bool = False

    def __post_init__(self) -> None:
        if type(self.bold) is not bool or type(self.italic) is not bool:
            raise DocumentError("Emphasis values must be booleans.")


@dataclass(frozen=True)
class Paragraph:
    text: str = ""
    styles: tuple[Style, ...] = ()
    alignment: str = "left"

    def __post_init__(self) -> None:
        validate_text(self.text, paragraphs=False)
        if (
            not isinstance(self.styles, tuple)
            or len(self.text) != len(self.styles)
            or any(not isinstance(s, Style) for s in self.styles)
        ):
            raise DocumentError("Every character must have an explicit supported style.")
        if self.alignment not in ALIGNMENTS:
            raise DocumentError("Unknown paragraph alignment.")


@dataclass(frozen=True)
class Document:
    paragraphs: tuple[Paragraph, ...] = (Paragraph(),)

    def __post_init__(self) -> None:
        if (
            not isinstance(self.paragraphs, tuple)
            or not self.paragraphs
            or any(not isinstance(p, Paragraph) for p in self.paragraphs)
        ):
            raise DocumentError("A document must contain at least one paragraph.")
        validate_text(self.text)

    @property
    def text(self) -> str:
        return "\n".join(p.text for p in self.paragraphs)

    def locate(self, position: int) -> tuple[int, int]:
        if not 0 <= position <= len(self.text):
            raise DocumentError("Position is outside the document.")
        for index, paragraph in enumerate(self.paragraphs):
            if position <= len(paragraph.text):
                return index, position
            position -= len(paragraph.text) + 1
        raise DocumentError("Position is outside the document.")

    def style_at(self, position: int) -> Style:
        index, offset = self.locate(position)
        styles = self.paragraphs[index].styles
        return styles[min(max(offset - 1, 0), len(styles) - 1)] if styles else Style()


def runs(paragraph: Paragraph):
    """Yield compact (text, style) spans for both persistence and presentation."""
    start = 0
    while start < len(paragraph.text):
        end = start + 1
        while end < len(paragraph.text) and paragraph.styles[end] == paragraph.styles[start]:
            end += 1
        yield paragraph.text[start:end], paragraph.styles[start]
        start = end
