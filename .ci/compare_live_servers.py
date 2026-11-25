"""Require both real processes to pass the same nonempty scenario inventory.

This is bounded differential evidence, not a complete vanilla parity claim.
"""
import argparse
import json
from pathlib import Path


def compare(steel, vanilla):
    if steel.get('engine') != 'steel' or vanilla.get('engine') != 'vanilla-26.3':
        raise ValueError('Missing or incorrect engine provenance')
    inventories = []
    for report in (steel, vanilla):
        results = report['results']
        names = [row['name'] for row in results]
        if not names or len(names) != len(set(names)):
            raise ValueError('Empty or duplicate scenario inventory')
        failed = [row['name'] for row in results if row.get('passed') is not True]
        if failed:
            raise ValueError('Failed scenarios: ' + ', '.join(failed))
        if names[-1] != 'second_clean_shutdown':
            raise ValueError('Incomplete runtime execution')
        inventories.append(names)
    if inventories[0] != inventories[1]:
        raise ValueError('Engines ran different scenario inventories')
    return {'compared_scenarios': len(inventories[0]), 'names': inventories[0],
            'complete_vanilla_parity': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('steel', type=Path)
    parser.add_argument('vanilla', type=Path)
    args = parser.parse_args()
    print(json.dumps(compare(json.loads(args.steel.read_text()), json.loads(args.vanilla.read_text())), indent=2))
