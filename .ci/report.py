#!/usr/bin/env python3
"""Publish CI log tails where they can always be read.

Log and artifact downloads are served from a host that some environments cannot
reach, and annotated `run` steps are truncated to the first ten annotations. A
commit comment is plain REST and is not truncated, so the tail of a failing log
goes there.

Usage: report.py TITLE FILE [FILE...]
"""

import json
import os
import sys
import urllib.request

MAX_CHARS = 50_000


def main() -> int:
    title = sys.argv[1]
    paths = sys.argv[2:]

    token = os.environ.get("GITHUB_TOKEN")
    repo = os.environ.get("GITHUB_REPOSITORY")
    sha = os.environ.get("GITHUB_SHA")
    if not token or not repo or not sha:
        print("report.py: missing GitHub context, skipping", file=sys.stderr)
        return 0

    parts = [f"## {title}"]
    for path in paths:
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except FileNotFoundError:
            parts.append(f"### `{path}`\n\n*missing*")
            continue
        tail = text[-MAX_CHARS:]
        parts.append(f"### `{path}`\n\n```\n{tail}\n```")
    body = "\n\n".join(parts)

    request = urllib.request.Request(
        f"https://api.github.com/repos/{repo}/commits/{sha}/comments",
        data=json.dumps({"body": body}).encode(),
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github+json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request) as response:
            print(f"report.py: posted comment {response.status}")
    except Exception as error:  # noqa: BLE001 - reporting must never fail the job
        print(f"report.py: could not post comment: {error}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
