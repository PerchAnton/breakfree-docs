"""Strict native v1 JSON and same-directory write-then-replace persistence."""

import json
import os
import stat
import tempfile
from pathlib import Path

from .model import MAX_CHARACTERS, Document, DocumentError, Paragraph, Style, runs

FORMAT_ID = "breakfree.docs.native"
CONTENT_VERSION = 1
MAX_FILE_BYTES = 16 * 1024 * 1024


class StorageError(ValueError):
    """A file cannot be read or committed safely."""


def _keys(value, keys: set[str]) -> None:
    if not isinstance(value, dict) or set(value) != keys:
        raise StorageError("Malformed or unsupported native document fields.")


def _unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise StorageError(f"Duplicate JSON field: {key}.")
        result[key] = value
    return result


def encode(document: Document) -> bytes:
    value = {
        "format": FORMAT_ID,
        "content_version": CONTENT_VERSION,
        "paragraphs": [
            {
                "alignment": p.alignment,
                "runs": [
                    {"text": text, "bold": style.bold, "italic": style.italic}
                    for text, style in runs(p)
                ],
            }
            for p in document.paragraphs
        ],
    }
    data = (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    if len(data) > MAX_FILE_BYTES:
        raise StorageError("Document exceeds the native file size limit.")
    return data


def decode(data: bytes) -> Document:
    if len(data) > MAX_FILE_BYTES:
        raise StorageError("File exceeds the 16 MiB limit.")
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_unique_pairs)
        _keys(value, {"format", "content_version", "paragraphs"})
        if value["format"] != FORMAT_ID:
            raise StorageError("This file is not a BreakFree Docs native document.")
        if type(value["content_version"]) is not int or value["content_version"] != CONTENT_VERSION:
            raise StorageError("Unsupported document content version; the file was not changed.")
        items = value["paragraphs"]
        if not isinstance(items, list) or not 1 <= len(items) <= MAX_CHARACTERS + 1:
            raise StorageError("Invalid paragraph collection.")
        paragraphs = []
        total = len(items) - 1
        for item in items:
            _keys(item, {"alignment", "runs"})
            if not isinstance(item["runs"], list):
                raise StorageError("Invalid run collection.")
            text_parts, styles = [], []
            for run in item["runs"]:
                _keys(run, {"text", "bold", "italic"})
                if not isinstance(run["text"], str) or not run["text"]:
                    raise StorageError("Text runs must be nonempty strings.")
                total += len(run["text"])
                if total > MAX_CHARACTERS:
                    raise StorageError("Document exceeds the 100,000 character limit.")
                style = Style(run["bold"], run["italic"])
                text_parts.append(run["text"])
                styles.extend([style] * len(run["text"]))
            paragraphs.append(Paragraph("".join(text_parts), tuple(styles), item["alignment"]))
        return Document(tuple(paragraphs))
    except (UnicodeError, json.JSONDecodeError, RecursionError, DocumentError, TypeError) as error:
        raise StorageError(f"Cannot read this native document: {error}") from error


def native_path(path: Path) -> Path:
    path = Path(path)
    if path.suffix.lower() != ".bfdoc":
        raise StorageError("Only .bfdoc native documents are supported.")
    return path


def load(path: Path) -> Document:
    path = native_path(path)
    try:
        with path.open("rb") as stream:
            data = stream.read(MAX_FILE_BYTES + 1)
        return decode(data)
    except OSError as error:
        raise StorageError(f"Cannot open {path.name}: {error}") from error


def save(document: Document, path: Path) -> None:
    """No in-place truncation. Errors before os.replace preserve the old destination.

    This does not promise recovery after power loss, directory fsync, concurrent
    writer isolation, or behavior on every network filesystem. See NATIVE_FORMAT.md.
    """
    path = native_path(path)
    data = encode(document)
    temporary = None
    try:
        if path.is_symlink():
            raise StorageError("Saving over symbolic links is not supported; choose another path.")
        mode = stat.S_IMODE(path.stat().st_mode) if path.exists() else None
        with tempfile.NamedTemporaryFile(
            mode="wb", prefix=f".{path.name}.", suffix=".tmp", dir=path.parent, delete=False
        ) as stream:
            temporary = Path(stream.name)
            if stream.write(data) != len(data):
                raise OSError("The filesystem reported an incomplete write.")
            stream.flush()
            os.fsync(stream.fileno())
        if mode is not None:
            os.chmod(temporary, mode)
        os.replace(temporary, path)
        temporary = None
    except OSError as error:
        raise StorageError(f"Cannot save {path.name}: {error}") from error
    finally:
        if temporary is not None:
            try:
                temporary.unlink(missing_ok=True)
            except OSError:
                pass
