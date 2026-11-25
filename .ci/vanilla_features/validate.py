"""Reject captured block references unless they reproduce the existing fixture."""
import gzip
import hashlib
import json
from pathlib import Path
import struct


def main():
    expected = {(c['x'], c['z']): c['stages']['minecraft:features'] for c in
                json.loads(Path('fixture.json').read_text())['dimensions']['minecraft:overworld']['chunks']}
    with gzip.open('reference.bin.gz', 'rb') as handle:
        def integer():
            return struct.unpack('>i', handle.read(4))[0]
        count = integer()
        assert count == 7
        failed = []
        for _ in range(count):
            x, z, sections = integer(), integer(), integer()
            hash_ = hashlib.md5()
            for _ in range(sections):
                marker = handle.read(1)
                assert marker in (b'\0', b'\1')
                data = b'\0' if marker == b'\0' else handle.read(4096 * 4)
                assert len(data) == (1 if marker == b'\0' else 4096 * 4)
                hash_.update(data)
            actual = hash_.hexdigest()
            print(f'CAPTURE {x},{z}: expected={expected[x,z]} actual={actual}', flush=True)
            if actual != expected[x, z]:
                failed.append((x, z))
        assert not handle.read(1)
        assert not failed, f'Untrusted capture: {failed}; expected hashes are NOT changed'


if __name__ == '__main__':
    main()
