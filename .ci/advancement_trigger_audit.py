#!/usr/bin/env python3
"""Inventory vanilla trigger names against Steel's conservative dispatcher.

This is a source-level inventory, not a test of predicate correctness or event
wiring. The generated vanilla source must correspond to the target MC version.
"""
import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VANILLA = ROOT / 'minecraft-src/minecraft/src/net/minecraft/advancements/triggers/CriteriaTriggers.java'
STEEL = ROOT / 'steel-core/src/player/advancement/mod.rs'


def inventory(java, rust):
    registered = re.findall(
        r'public\s+static\s+final\s+\w+\s+(\w+)\s*=\s*register\("([a-z0-9_]+)"',
        java,
    )
    if not registered or len(dict(registered)) != len(registered):
        raise ValueError('missing or duplicate vanilla trigger declarations')
    if len({name for _, name in registered}) != len(registered):
        raise ValueError('duplicate vanilla trigger identifiers')
    start = rust.find('pub fn trigger_with_context(')
    end = rust.find('let mut granted = 0;', start)
    if start < 0 or end < 0:
        raise ValueError('cannot locate the supported-trigger gate')
    gate = rust[start:end]
    if 'if !block_location && !used_item && !changed_dimension && !slept_in_bed && !nether_travel' not in gate:
        raise ValueError('dispatcher gate changed; audit must be reviewed')
    enabled = set()
    for variable in ('block_location', 'used_item', 'changed_dimension', 'slept_in_bed', 'nether_travel'):
        assignment = re.search(r'let\s+' + variable + r'\s*=\s*([^;]+);', gate)
        if not assignment:
            raise ValueError(f'cannot locate {variable} trigger expression')
        for term in assignment.group(1).split('||'):
            match = re.fullmatch(r'\s*trigger\s*==\s*&vanilla_criteria_triggers::([A-Z_]+)\s*', term)
            if not match:
                raise ValueError(f'{variable} expression changed; audit must be reviewed')
            enabled.add(match.group(1))
    missing = enabled - dict(registered).keys()
    if missing:
        raise ValueError(f'enabled constants absent from vanilla registry: {sorted(missing)}')
    return [(constant, name, constant in enabled) for constant, name in registered]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--vanilla', type=Path, default=VANILLA)
    parser.add_argument('--steel', type=Path, default=STEEL)
    args = parser.parse_args()
    rows = inventory(args.vanilla.read_text(), args.steel.read_text())
    supported = [name for _, name, enabled in rows if enabled]
    print(f'Vanilla registered triggers: {len(rows)}')
    print(f'Enabled with restricted predicates in this dispatcher: {len(supported)}')
    print('Enabled: ' + ', '.join(supported))
    print('Not enabled in this dispatcher: ' + ', '.join(name for _, name, enabled in rows if not enabled))
    print('Source inventory only: not a measurement of working mechanics or vanilla parity.')


if __name__ == '__main__':
    main()
