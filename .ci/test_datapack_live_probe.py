"""Validate differential harness accounting, not gameplay."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('datapack_probe', HERE / 'datapack_live_probe.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)
spec = importlib.util.spec_from_file_location('compare_datapack_probe', HERE / 'compare_datapack_probe.py')
compare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compare)


class FakeClient:
    def __init__(self, active):
        self.active = active
        self.score = 0

    def command(self, command):
        if command == 'scoreboard players set probe ci 0':
            self.score = 0
        elif command == 'function arena:probe' and self.active:
            self.score = 202
        elif command == 'scoreboard players get probe ci':
            return f'probe has {self.score} [ci]'
        return ''


def report(engine):
    return {'schema': 1, 'engine': engine,
            'results': [{'name': name, 'status': 'passed'} for name in probe.CASE_IDS]}


class DatapackProbeTests(unittest.TestCase):
    def test_pack_format_function_and_score_check(self):
        with tempfile.TemporaryDirectory() as temp:
            pack = Path(temp) / 'pack'
            function = probe.write_pack(pack)
            metadata = json.loads((pack / 'pack.mcmeta').read_text())
            self.assertEqual(metadata['pack']['min_format'], [121, 0])
            self.assertEqual(metadata['pack']['max_format'], [121, 0])
            self.assertEqual(function.read_text(), 'scoreboard players set probe ci 101\n')
        self.assertTrue(probe.has_score('probe has 202 [ci]\n', 202))
        self.assertFalse(probe.has_score('error setting probe has 202 [ci]', 202))
        self.assertFalse(probe.has_score('probe has 2020 [ci]', 202))

    def test_disabled_check_distinguishes_running_from_unavailable(self):
        probe.wait_for_disabled(FakeClient(False), timeout=1, stable_seconds=0)
        with self.assertRaises(AssertionError):
            probe.wait_for_disabled(FakeClient(True), timeout=0.01, stable_seconds=0)
        probe.wait_for_function(FakeClient(True), 202, timeout=1)
        with self.assertRaises(AssertionError):
            probe.wait_for_function(FakeClient(False), 202, timeout=0.01)

    def test_blocked_and_bad_vanilla_never_become_success(self):
        steel, vanilla = report('steel'), report('vanilla-26.3')
        steel['results'][1]['status'] = 'failed'
        steel['results'][2]['status'] = 'blocked'
        vanilla['results'][3]['status'] = 'failed'
        vanilla['results'][4]['status'] = 'blocked'
        outcomes = [r['result'] for r in compare.compare(steel, vanilla)['results']]
        self.assertEqual(outcomes, ['matched_assertions', 'steel_failed', 'blocked_steel',
                                    'invalid_reference', 'blocked_reference'])

    def test_incomplete_mismatched_or_duplicate_cases_are_rejected(self):
        steel, vanilla = report('steel'), report('vanilla-26.3')
        for mutate in ('missing', 'duplicate', 'status', 'wrong_engine'):
            value = copy.deepcopy(steel)
            if mutate == 'missing':
                value['results'].pop()
            elif mutate == 'duplicate':
                value['results'][1]['name'] = value['results'][0]['name']
            elif mutate == 'status':
                value['results'][0]['status'] = 'skipped-as-pass'
            else:
                value['engine'] = 'vanilla-26.3'
            with self.subTest(mutate=mutate), self.assertRaises(ValueError):
                compare.compare(value, vanilla)


if __name__ == '__main__':
    unittest.main()
