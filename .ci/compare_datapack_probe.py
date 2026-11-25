#!/usr/bin/env python3
"""Compare complete datapack probes without treating blocked cases as passing."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('datapack_live_probe', HERE / 'datapack_live_probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def compare(steel, vanilla):
    parsed = {}
    for engine, report in [('steel', steel), ('vanilla-26.3', vanilla)]:
        if report.get('schema') != 1 or report.get('engine') != engine:
            raise ValueError(f'invalid {engine} provenance')
        rows = report.get('results')
        if not isinstance(rows, list) or len(rows) != len(probe.CASE_IDS):
            raise ValueError(f'incomplete {engine} inventory')
        ids = [row['name'] for row in rows]
        if tuple(ids) != probe.CASE_IDS:
            raise ValueError(f'wrong or duplicate {engine} case inventory')
        if any(row.get('status') not in ('passed', 'failed', 'blocked') for row in rows):
            raise ValueError(f'unknown {engine} case status')
        parsed[engine] = {row['name']: row for row in rows}
    results = []
    for name in probe.CASE_IDS:
        s, v = parsed['steel'][name], parsed['vanilla-26.3'][name]
        if v['status'] != 'passed':
            outcome = 'invalid_reference' if v['status'] == 'failed' else 'blocked_reference'
        elif s['status'] == 'passed':
            outcome = 'matched_assertions'
        elif s['status'] == 'failed':
            outcome = 'steel_failed'
        else:
            outcome = 'blocked_steel'
        results.append({'name': name, 'result': outcome, 'steel': s, 'vanilla': v})
    return {'schema': 1, 'scope': 'directory function packs via RCON; no game client',
            'complete_vanilla_parity': False, 'results': results}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('steel', type=Path)
    parser.add_argument('vanilla', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    report = compare(json.loads(args.steel.read_text()), json.loads(args.vanilla.read_text()))
    text = json.dumps(report, indent=2) + '\n'
    args.output.write_text(text)
    print(text)
    if any(row['result'] != 'matched_assertions' for row in report['results']):
        sys.exit(1)


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f'datapack comparison refused: {error}', file=sys.stderr)
        sys.exit(2)
