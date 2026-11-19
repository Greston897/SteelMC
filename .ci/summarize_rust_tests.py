"""Index every observed Rust test outcome by crate/module; never infer parity.

Input is the unmodified `cargo test --workspace --verbose` log. This report
supplements the cargo exit code; it cannot turn a failed build into a pass.
"""
import argparse
import collections
import json
from pathlib import Path
import re

ANSI = re.compile(r'\x1b\[[0-9;]*m')
BINARY = re.compile(r'/deps/([a-zA-Z0-9_]+)-[0-9a-f]+(?:[\s`)]|$)')
TEST = re.compile(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$')


def summarize(text):
    crate = 'unknown'
    tests = []
    groups = collections.defaultdict(collections.Counter)
    for raw in text.splitlines():
        line = ANSI.sub('', raw).strip()
        if 'Running ' in line:
            match = BINARY.search(line)
            if match:
                crate = match[1]
        elif line.startswith('Doc-tests '):
            crate = line.removeprefix('Doc-tests ').strip() + ':doctest'
        match = TEST.fullmatch(line)
        if match is None:
            continue
        name, status = match.groups()
        status = {'ok': 'passed', 'FAILED': 'failed', 'ignored': 'ignored'}[status]
        module = name.split('::', 1)[0] if '::' in name else '(root or doctest)'
        tests.append({'crate': crate, 'name': name, 'status': status})
        groups[crate + '/' + module][status] += 1
    if not tests:
        raise ValueError('No individual test results found; compilation/runner may have failed')
    return {'scope': 'observed automated Rust tests; no human players required',
            'complete_vanilla_parity': False,
            'totals': dict(collections.Counter(row['status'] for row in tests)),
            'groups': dict(sorted(groups.items())), 'tests': tests}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('log', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    report = summarize(args.log.read_text(errors='replace'))
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({'totals': report['totals'], 'groups': report['groups']}, indent=2))
