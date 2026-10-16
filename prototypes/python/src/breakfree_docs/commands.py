"""The single editing implementation used by the desktop and headless tests."""

from dataclasses import dataclass, replace

from .model import (
    ALIGNMENTS,
    Document,
    DocumentError,
    Paragraph,
    Style,
    boundaries,
    validate_range,
    validate_text,
)


@dataclass(frozen=True)
class Operation:
    label: str
    before: Document
    after: Document
    before_selection: tuple[int, int]
    after_selection: tuple[int, int]


class Editor:
    """Bounded snapshot history. Each command is exactly one undo operation."""

    def __init__(self, document: Document | None = None):
        self.document = document or Document()
        self.selection = (0, 0)
        self.typing_style = Style()
        self.undo_stack: list[Operation] = []
        self.redo_stack: list[Operation] = []

    def _commit(self, document: Document, label: str, selection: tuple[int, int]) -> None:
        if document != self.document:
            self.undo_stack.append(
                Operation(label, self.document, document, self.selection, selection)
            )
            del self.undo_stack[:-100]
            self.redo_stack.clear()
            self.document = document
        self.selection = selection

    def replace_text(self, start: int, end: int, text: str, label: str = "Edit text") -> None:
        old = self.document
        validate_range(old.text, start, end)
        validate_text(text)
        first, left = old.locate(start)
        last, right = old.locate(end)
        a, b = old.paragraphs[first], old.paragraphs[last]
        joined = a.text[:left] + text + b.text[right:]
        styles = a.styles[:left] + (self.typing_style,) * len(text) + b.styles[right:]
        paragraphs = []
        offset = 0
        for part in joined.split("\n"):
            paragraphs.append(Paragraph(part, styles[offset : offset + len(part)], a.alignment))
            offset += len(part) + 1
        result = Document(old.paragraphs[:first] + tuple(paragraphs) + old.paragraphs[last + 1 :])
        self.selection = (start, end)
        caret = start + len(text)
        # Insertions such as a ZWJ can join clusters on both sides of the caret.
        caret = next(edge for edge in boundaries(result.text) if edge >= caret)
        self._commit(result, label, (caret, caret))

    def emphasize(self, start: int, end: int, attribute: str, enabled: bool) -> None:
        validate_range(self.document.text, start, end)
        if attribute not in ("bold", "italic") or type(enabled) is not bool:
            raise DocumentError("Unsupported emphasis command.")
        if start == end:
            self.typing_style = replace(self.typing_style, **{attribute: enabled})
            return
        paragraphs = []
        offset = 0
        for paragraph in self.document.paragraphs:
            styles = tuple(
                replace(style, **{attribute: enabled}) if start <= offset + i < end else style
                for i, style in enumerate(paragraph.styles)
            )
            paragraphs.append(replace(paragraph, styles=styles))
            offset += len(paragraph.text) + 1
        self.selection = (start, end)
        self._commit(Document(tuple(paragraphs)), f"Set {attribute}", (start, end))

    def align(self, start: int, end: int, alignment: str) -> None:
        validate_range(self.document.text, start, end)
        if alignment not in ALIGNMENTS:
            raise DocumentError("Unsupported alignment command.")
        first, _ = self.document.locate(start)
        last, offset = self.document.locate(end)
        if end > start and offset == 0 and last > first:
            last -= 1
        paragraphs = tuple(
            replace(p, alignment=alignment) if first <= i <= last else p
            for i, p in enumerate(self.document.paragraphs)
        )
        self.selection = (start, end)
        self._commit(Document(paragraphs), "Align paragraphs", (start, end))

    def undo(self) -> None:
        if self.undo_stack:
            operation = self.undo_stack.pop()
            self.redo_stack.append(operation)
            self.document = operation.before
            self.selection = operation.before_selection
            self.typing_style = self.document.style_at(self.selection[0])

    def redo(self) -> None:
        if self.redo_stack:
            operation = self.redo_stack.pop()
            self.undo_stack.append(operation)
            self.document = operation.after
            self.selection = operation.after_selection
            self.typing_style = self.document.style_at(self.selection[0])
