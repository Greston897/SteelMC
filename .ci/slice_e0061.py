"""Print the source around an E0061 (wrong argument count) diagnostic.

The generated registries are single lines hundreds of kilobytes long, so
rustc's own excerpt of the offending call is useless: it can only show the
one line. This slices the real source at the reported line:column instead.
"""

from __future__ import annotations

import glob
import re

DIAGNOSTIC = re.compile(r"^(\S+\.rs):(\d+):(\d+): error\[E0061\]", re.M)
# rustc follows the diagnostic with the callee's definition site; that note is
# what actually names the constructor whose arity disagrees.
DEFINITION = re.compile(r"^\s*--> (\S+\.rs):(\d+):(\d+)$", re.M)


def render(path: str, line: int, col: int) -> str | None:
    """Return an excerpt centred on `line:col`, or None if the file is gone."""
    try:
        source = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return None
    offset = sum(len(part) for part in source.splitlines(keepends=True)[: line - 1])
    offset += col - 1
    return source[max(0, offset - 300) : offset + 200].replace("\n", " ")


def main() -> None:
    found = False
    for log in sorted(glob.glob("/tmp/b?.log")):
        with open(log, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        for match in DIAGNOSTIC.finditer(text):
            path, line, col = match.group(1), int(match.group(2)), int(match.group(3))
            excerpt = render(path, line, col)
            if excerpt is None:
                continue
            found = True
            print(f"--- {path}:{line}:{col}")
            print(f"    ...{excerpt}...")
            for note in DEFINITION.finditer(text, match.end(), match.end() + 2000):
                note_path, note_line = note.group(1), int(note.group(2))
                try:
                    source = open(note_path, encoding="utf-8", errors="replace").read()
                except OSError:
                    continue
                body = source.splitlines(keepends=True)
                snippet = " ".join("".join(body[max(0, note_line - 1) : note_line + 2]).split())
                print(f"    defined at {note_path}:{note_line}: {snippet[:200]}")
                break
    if not found:
        print("no E0061 diagnostic found")


if __name__ == "__main__":
    main()
