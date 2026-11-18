#!/usr/bin/env python3
"""Compile a lib-test target once and record Cargo's exact executable path.

Avoid repeated `cargo test` builds when build scripts regenerate source/assets.
Diagnostics remain visible; compilation failures are never swallowed.
Usage: compile_test_binary.py PACKAGE OUTPUT_PATH
"""
import json
import os
from pathlib import Path
import subprocess
import sys


def test_executable(artifact, package):
    target = artifact.get("target", {})
    if (artifact.get("reason") == "compiler-artifact"
            and target.get("name") == package.replace("-", "_")
            and "lib" in target.get("kind", [])
            and artifact.get("profile", {}).get("test")
            and artifact.get("executable")):
        return artifact["executable"]
    return None


def main():
    package, output = sys.argv[1:]
    process = subprocess.Popen(
        ["cargo", "test", "--profile", "evidence", "-p", package,
         "--lib", "--no-run", "--message-format=json"],
        stdout=subprocess.PIPE, text=True,
    )
    candidates = set()
    for line in process.stdout:
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            print(line, end="", flush=True)
            continue
        if message.get("reason") == "compiler-message":
            print(message.get("message", {}).get("rendered", ""), end="", flush=True)
        executable = test_executable(message, package)
        if executable:
            candidates.add(executable)
    code = process.wait()
    if code:
        return code
    if len(candidates) != 1:
        raise RuntimeError(f"Expected one executable for {package}, got {candidates}")
    binary = Path(candidates.pop()).resolve(strict=True)
    if not os.access(binary, os.X_OK):
        raise RuntimeError(f"Not executable: {binary}")
    Path(output).write_text(str(binary))
    print(f"Compiled test executable: {binary}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
