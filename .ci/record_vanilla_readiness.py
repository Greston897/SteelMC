#!/usr/bin/env python3
"""Package two result files with verified GitHub Actions run metadata.

This checks scenario IDs and metadata, but cannot prove arbitrary local result
files came from the run: obtain them from the run artifacts/comments yourself.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('vanilla_readiness', HERE / 'vanilla_readiness.py')
readiness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(readiness)


def make_evidence(inventory_path, ids, steel, vanilla, run_id, metadata):
    if not run_id.isdecimal() or int(run_id) <= 0:
        raise ValueError('invalid Actions run ID')
    if metadata.get('status') != 'completed':
        raise ValueError('Actions run is not complete')
    data = {
        'schema': 1, 'kind': 'user_supplied_results_with_checked_run_metadata',
        'steel_commit': metadata.get('headSha'),
        'run_url': f'https://github.com/Greston897/SteelMC/actions/runs/{run_id}',
        'run_conclusion': metadata.get('conclusion'),
        'inventory_sha256': hashlib.sha256(Path(inventory_path).read_bytes()).hexdigest(),
        'origin': 'Local input files; GitHub run metadata checked, file origin must be verified separately.',
        'reports': {'steel': steel, 'vanilla-26.3': vanilla},
    }
    readiness.scenario_results(ids, data, data['inventory_sha256'])
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--inventory', choices=('smoke', 'manual'), required=True)
    parser.add_argument('--run', required=True, help='GitHub Actions numerical run ID')
    parser.add_argument('--steel', required=True, type=Path, help='Steel results.json from that run')
    parser.add_argument('--vanilla', required=True, type=Path, help='Vanilla results.json from that run')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    if args.inventory == 'smoke':
        path = readiness.DATA / 'smoke-inventory.json'
        ids = readiness.load_json(path)['cases']
    else:
        path = readiness.ROOT / '.ci/manual_mechanics_cases.tsv'
        ids = [row['# id'] for row in readiness.manual_inventory(path)]
    if not args.run.isdecimal() or int(args.run) <= 0:
        raise ValueError('invalid Actions run ID')
    metadata = json.loads(subprocess.check_output(
        ['gh', 'run', 'view', args.run, '--repo', 'Greston897/SteelMC', '--json',
         'status,conclusion,headSha'], text=True))
    evidence = make_evidence(path, ids, readiness.load_json(args.steel),
                             readiness.load_json(args.vanilla), args.run, metadata)
    args.output.write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Wrote {args.output}. File provenance is not authenticated by run metadata.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f'readiness evidence refused: {error}', file=sys.stderr)
        sys.exit(2)
