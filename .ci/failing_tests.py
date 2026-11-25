#!/usr/bin/env python3
"""Re-run each failing test on its own and print its diagnostics.

The per-crate logs are written by `cargo test -p <pkg>`, which buffers every
panic and only prints the `failures:` block once the whole binary finished. When
the binary dies early (memory pressure on the runner) that block never appears,
so the failing tests are re-run one at a time: a single-test binary always prints
its own panic.

Usage: failing_tests.py LOG_DIR OUTPUT [MAX_TESTS]
"""

import re
import subprocess
import sys
from pathlib import Path

FAILED = re.compile(r"^test (?P<name>\S+) \.\.\. FAILED$")
KEEP = re.compile(
    r"(panicked at|^assertion|^  left:|^  right:|^---- .* stdout ----|^failures:|^note: )"
)


def failed_names(log: Path) -> list[str]:
    names: list[str] = []
    for line in log.read_text(encoding="utf-8", errors="replace").splitlines():
        match = FAILED.match(line.strip())
        if match and match.group("name") not in names:
            names.append(match.group("name"))
    return names


def main() -> int:
    log_dir = Path(sys.argv[1])
    output = Path(sys.argv[2])
    limit = int(sys.argv[3]) if len(sys.argv) > 3 else 40

    sections: list[str] = []
    for log in sorted(log_dir.glob("test-*.log")):
        package = log.stem.removeprefix("test-")
        names = failed_names(log)
        if not names:
            continue
        sections.append(f"### {package} ({len(names)} failing)")
        for name in names[:limit]:
            sections.append(f"#### {name}")
            result = subprocess.run(
                [
                    "cargo",
                    "test",
                    "-p",
                    package,
                    "--all-features",
                    name,
                    "--",
                    "--exact",
                    "--nocapture",
                ],
                capture_output=True,
                text=True,
                errors="replace",
            )
            lines = result.stdout.splitlines() + result.stderr.splitlines()
            kept = [line for line in lines if KEEP.search(line)]
            if not kept:
                kept = lines[-30:]
            sections.extend(kept[:60])
            sections.append("")

    output.write_text("\n".join(sections), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
