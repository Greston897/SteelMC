"""Print the source around an E0061 (wrong argument count) diagnostic.

The generated registries are single lines hundreds of kilobytes long, so
rustc's own excerpt is unreadable: it can only show that one line. This finds
the  file:line:column  rustc reports and prints the real source around it, and
then the "defined here" note that names the callee whose arity disagrees.
"""

from __future__ import annotations

import glob
import re

# rustc prints the location on its own line as `--> path:line:col`, so the
# diagnostic line and the location line have to be matched separately.
DIAGNOSTIC = re.compile(r"^error\[E0061\][^\n]*\n\s*--> (\S+):(\d+):(\d+)", re.M)
DEFINITION = re.compile(r"defined here\n\s*--> (\S+):(\d+):(\d+)")

SPAN_BEFORE = 400
SPAN_AFTER = 260


def excerpt(path: str, line: int, col: int) -> str | None:
    """One-line window of `path` centred on `line:col`."""
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            source = handle.read()
    except OSError:
        return None
    offset = sum(len(part) for part in source.splitlines(keepends=True)[: line - 1]) + col - 1
    window = source[max(0, offset - SPAN_BEFORE) : offset + SPAN_AFTER]
    return " ".join(window.split())


def main() -> None:
    printed = False
    for log in sorted(glob.glob("/tmp/b?.log")):
        with open(log, encoding="utf-8", errors="replace") as handle:
            text = handle.read()
        for match in DIAGNOSTIC.finditer(text):
            path, line, col = match.group(1), int(match.group(2)), int(match.group(3))
            rendered = excerpt(path, line, col)
            if rendered is None:
                continue
            printed = True
            print(f"--- call site {path}:{line}:{col}")
            print(f"    ...{rendered}...")
        for match in DEFINITION.finditer(text):
            path, line = match.group(1), int(match.group(2))
            rendered = excerpt(path, line, 1)
            if rendered is None:
                continue
            printed = True
            print(f"--- defined at {path}:{line}")
            print(f"    ...{rendered}...")
    if not printed:
        print("no E0061 diagnostic found")


if __name__ == "__main__":
    main()
