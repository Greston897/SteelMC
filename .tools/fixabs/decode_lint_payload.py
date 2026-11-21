#!/usr/bin/env python3
"""Decode the `LINT_DIAGNOSTICS_V1` payload that the Lint workflow posts as commit comments.

The workflow base64+zlib compresses a JSON object holding the raw `fmt` and
`clippy` logs, then splits the encoded text into 44000 character chunks. Each
chunk is posted as a commit comment whose body starts with
`LINT_DIAGNOSTICS_V1 <sha256 digest> <index>/<count>`.

Usage:
    python3 decode_lint_payload.py <commit-sha> [output-dir]
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import zlib

PREFIX = "LINT_DIAGNOSTICS_V1"


def fetch_comments(sha: str) -> list[str]:
    """Return every commit comment body for `sha`, oldest first."""
    bodies: list[tuple[int, str]] = []
    page = 1
    while True:
        raw = subprocess.run(
            [
                "gh",
                "api",
                f"repos/{os.environ.get('GITHUB_REPOSITORY', 'Greston897/SteelMC')}"
                f"/commits/{sha}/comments?per_page=100&page={page}",
            ],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        batch = json.loads(raw)
        if not batch:
            break
        for comment in batch:
            bodies.append((comment.get("created_at", ""), comment.get("body", "")))
        page += 1
    bodies.sort(key=lambda item: item[0])
    return [body for _, body in bodies]


def collect(chunks: list[tuple[int, int, str]], count: int) -> str:
    """Join the numbered chunks back into the original encoded text."""
    parts: dict[int, str] = {}
    for index, _total, text in chunks:
        parts[index] = text
    missing = [index for index in range(1, count + 1) if index not in parts]
    if missing:
        raise SystemExit(f"missing payload chunks: {missing}")
    return "".join(parts[index] for index in range(1, count + 1))


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    sha = sys.argv[1]
    out_dir = pathlib.Path(sys.argv[2] if len(sys.argv) > 2 else f"/tmp/lint_{sha[:7]}")
    out_dir.mkdir(parents=True, exist_ok=True)

    groups: dict[str, list[tuple[int, int, str]]] = {}
    digests: dict[str, str] = {}
    for body in fetch_comments(sha):
        if not body.startswith(PREFIX):
            continue
        header, _, payload = body.partition("\n")
        fields = header.split()
        if len(fields) != 3:
            continue
        digest = fields[1]
        index_text, _, total_text = fields[2].partition("/")
        index, total = int(index_text), int(total_text)
        groups.setdefault(digest, []).append((index, total, payload.strip()))
        digests[digest] = digest

    if not groups:
        print(f"no {PREFIX} payload found for {sha}")
        return 1

    for digest, chunks in groups.items():
        total = chunks[0][1]
        encoded = collect(chunks, total)
        raw = zlib.decompress(base64.b64decode(encoded))
        actual = hashlib.sha256(raw).hexdigest()
        if actual != digest:
            print(f"digest mismatch: expected {digest}, got {actual}")
            return 1
        logs = json.loads(raw)
        for name, text in logs.items():
            target = out_dir / f"{name}.log"
            target.write_text(text, encoding="utf-8")
            print(f"wrote {target} ({len(text)} chars)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
