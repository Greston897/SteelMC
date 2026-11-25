"""Tests of the trust gate; generated fixture data is never used as a game golden."""
import gzip
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

VALIDATOR = Path(__file__).with_name('validate.py').resolve()


class ReferenceValidationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.raw = bytearray(struct.pack('>i', 7))
        chunks = []
        for x in range(7):
            # Exercise both empty-section shortcut and full block-ID byte order.
            blocks = b''.join(struct.pack('>i', (x * 4096 + i) % 50000) for i in range(4096))
            hash_ = hashlib.md5(b'\0' + blocks).hexdigest()
            chunks.append({'x': x, 'z': -x, 'stages': {'minecraft:features': hash_}})
            self.raw.extend(struct.pack('>iii', x, -x, 2) + b'\0\1' + blocks)
        self.fixture = {'dimensions': {'minecraft:overworld': {'chunks': chunks}}}

    def run_capture(self, raw=None):
        (self.root / 'fixture.json').write_text(json.dumps(self.fixture))
        path = self.root / 'reference.bin.gz'
        path.write_bytes(gzip.compress(self.raw if raw is None else raw))
        fixture_before = (self.root / 'fixture.json').read_bytes()
        result = subprocess.run([sys.executable, str(VALIDATOR)], cwd=self.root,
                                capture_output=True, text=True, check=False)
        self.assertEqual((self.root / 'fixture.json').read_bytes(), fixture_before)
        return result

    def test_exact_block_data_and_section_shortcuts_match_all_seven_hashes(self):
        result = self.run_capture()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.count('CAPTURE '), 7)

    def test_one_changed_block_rejects_capture_without_rewriting_expected_hashes(self):
        raw = self.raw.copy()
        raw[-1] ^= 1
        result = self.run_capture(raw)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Untrusted capture', result.stderr)

    def test_truncated_block_data_is_not_accepted(self):
        self.assertNotEqual(self.run_capture(self.raw[:-1]).returncode, 0)

    def test_trailing_bytes_are_not_accepted(self):
        self.assertNotEqual(self.run_capture(self.raw + b'extra').returncode, 0)

    def test_wrong_chunk_count_is_not_accepted(self):
        self.assertNotEqual(self.run_capture(struct.pack('>i', 6) + self.raw[4:]).returncode, 0)

    def test_invalid_section_marker_is_not_accepted(self):
        raw = self.raw.copy()
        raw[16] = 2
        self.assertNotEqual(self.run_capture(raw).returncode, 0)


if __name__ == '__main__':
    unittest.main()
