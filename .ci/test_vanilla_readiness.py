"""Readiness accounting regression tests; these are not gameplay tests."""
import copy
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('vanilla_readiness', HERE / 'vanilla_readiness.py')
readiness = importlib.util.module_from_spec(spec)
spec.loader.exec_module(readiness)
spec = importlib.util.spec_from_file_location('record_readiness', HERE / 'record_vanilla_readiness.py')
record = importlib.util.module_from_spec(spec)
spec.loader.exec_module(record)


class ReadinessTests(unittest.TestCase):
    def setUp(self):
        self.ids = ['first', 'second', 'third']
        self.digest = 'a' * 64
        self.evidence = {
            'schema': 1, 'steel_commit': 'b' * 40,
            'run_url': 'https://github.com/Greston897/SteelMC/actions/runs/123',
            'run_conclusion': 'success', 'inventory_sha256': self.digest,
            'reports': {
                'steel': {'engine': 'steel', 'results': [
                    {'name': 'first', 'passed': True}, {'name': 'second', 'passed': False}]},
                'vanilla-26.3': {'engine': 'vanilla-26.3', 'results': [
                    {'name': 'first', 'passed': True}, {'name': 'second', 'passed': True}]},
            },
        }

    def calculate(self, evidence=None):
        return readiness.scenario_results(self.ids, self.evidence if evidence is None else evidence,
                                          self.digest)

    def test_results_do_not_invent_missing_cases(self):
        rows = self.calculate()
        self.assertEqual([r['status'] for r in rows],
                         ['matched_assertions', 'steel_failed', 'unknown'])
        self.assertEqual(readiness.counts(rows), {
            'matched_assertions': 1, 'steel_failed': 1, 'invalid_reference': 0, 'unknown': 1})
        self.assertEqual(readiness.percent(1, 3), 33.33)
        self.assertIsNone(readiness.percent(0, 0))

    def test_vanilla_failure_is_not_steel_failure_or_success(self):
        self.evidence['reports']['vanilla-26.3']['results'][0]['passed'] = False
        self.assertEqual(self.calculate()[0]['status'], 'invalid_reference')

    def test_rejects_inventories_and_results_that_could_hide_failures(self):
        for ids in ([], ['first', 'first'], ['bad name']):
            with self.subTest(ids=ids), self.assertRaises(ValueError):
                readiness.scenario_results(ids, self.evidence, self.digest)
        for extra in ({'name': 'first', 'passed': True},
                      {'name': 'not_in_inventory', 'passed': True},
                      {'name': 'third', 'passed': 1},
                      {'name': 'third'}):
            evidence = copy.deepcopy(self.evidence)
            evidence['reports']['steel']['results'].append(extra)
            with self.subTest(extra=extra), self.assertRaises(ValueError):
                self.calculate(evidence)

    def test_provenance_inventory_and_engine_identity_required(self):
        for key, value in [('steel_commit', 'b' * 7), ('run_url', 'https://example.com/run/123'),
                           ('run_conclusion', 'maybe'), ('inventory_sha256', 'c' * 64)]:
            evidence = copy.deepcopy(self.evidence)
            evidence[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                self.calculate(evidence)
        evidence = copy.deepcopy(self.evidence)
        evidence['reports']['vanilla-26.3']['engine'] = 'steel'
        with self.assertRaises(ValueError):
            self.calculate(evidence)

    def test_previous_dirty_failed_or_unknown_revision_not_current(self):
        sha = self.evidence['steel_commit']
        self.assertTrue(readiness.evidence_is_current(self.evidence, {'head': sha, 'dirty': False}, 'metadata_verified'))
        self.assertFalse(readiness.evidence_is_current(self.evidence, {'head': sha, 'dirty': False}, 'not_checked'))
        for state in ({'head': sha, 'dirty': True}, {'head': 'c' * 40, 'dirty': False},
                      {'head': None, 'dirty': None}):
            self.assertFalse(readiness.evidence_is_current(self.evidence, state, 'metadata_verified'))
        self.evidence['run_conclusion'] = 'failure'
        self.assertFalse(readiness.evidence_is_current(self.evidence, {'head': sha, 'dirty': False}, 'metadata_verified'))

    def test_ci_validation_checks_sha_and_conclusion_not_just_run_url(self):
        import json
        meta = {'status': 'completed', 'conclusion': 'success', 'headSha': 'b' * 40}
        with patch.object(readiness.subprocess, 'check_output', return_value=json.dumps(meta)):
            self.assertEqual(readiness.verify_ci(self.evidence), 'metadata_verified')
        meta['headSha'] = 'c' * 40
        with patch.object(readiness.subprocess, 'check_output', return_value=json.dumps(meta)):
            self.assertEqual(readiness.verify_ci(self.evidence), 'mismatch_or_incomplete')
        with patch.object(readiness.subprocess, 'check_output', side_effect=OSError('offline')):
            self.assertEqual(readiness.verify_ci(self.evidence), 'unavailable')

    def test_import_binds_exact_inventory_and_refuses_incomplete_run(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'inventory.txt'
            path.write_text('v1', encoding='utf-8')
            meta = {'status': 'completed', 'conclusion': 'success', 'headSha': 'b' * 40}
            steel = self.evidence['reports']['steel']
            vanilla = self.evidence['reports']['vanilla-26.3']
            data = record.make_evidence(path, self.ids, steel, vanilla, '123', meta)
            self.assertEqual(data['steel_commit'], meta['headSha'])
            self.assertEqual(readiness.counts(readiness.scenario_results(
                self.ids, data, data['inventory_sha256']))['unknown'], 1)
            path.write_text('v2', encoding='utf-8')
            import hashlib
            with self.assertRaises(ValueError):
                readiness.scenario_results(self.ids, data, hashlib.sha256(path.read_bytes()).hexdigest())
            meta['status'] = 'in_progress'
            with self.assertRaises(ValueError):
                record.make_evidence(path, self.ids, steel, vanilla, '123', meta)
            with self.assertRaises(ValueError):
                record.make_evidence(path, self.ids, steel, vanilla, 'abc', meta)

    def test_manual_tsv_rejects_duplicate_and_missing_fields(self):
        header = '# id\tarea\tmechanic\tsetup\taction\texpected\n'
        row = 'ONE\tarea\tmechanic\tsetup\taction\texpected\n'
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'cases.tsv'
            path.write_text(header + row, encoding='utf-8')
            self.assertEqual(len(readiness.manual_inventory(path)), 1)
            for text in (header + row + row, header + 'ONE\tarea\tmechanic\t\taction\texpected\n'):
                path.write_text(text, encoding='utf-8')
                with self.assertRaises(ValueError):
                    readiness.manual_inventory(path)

    def test_datapack_evidence_preserves_failure_and_blocked_results(self):
        data = readiness.load_json(readiness.DATA / 'datapack-live-ba05896.json')
        result = readiness.datapack_results(data, {'head': None, 'dirty': None}, 'not_checked')
        self.assertEqual(result['total'], 5)
        self.assertEqual(result['counts']['matched_assertions'], 2)
        self.assertEqual(result['counts']['steel_failed'], 1)
        self.assertEqual(result['counts']['blocked_steel'], 2)
        invalid = copy.deepcopy(data)
        invalid['probe_sha256'] = '0' * 64
        with self.assertRaises(ValueError):
            readiness.datapack_results(invalid, {'head': None, 'dirty': None}, 'not_checked')
        invalid = copy.deepcopy(data)
        invalid['reports']['steel']['results'][0]['status'] = 'invented'
        with self.assertRaises(ValueError):
            readiness.datapack_results(invalid, {'head': None, 'dirty': None}, 'not_checked')

    def test_real_saved_inputs_have_matching_inventory_and_explicit_unknown_manual(self):
        import hashlib
        inventory_path = readiness.DATA / 'smoke-inventory.json'
        inventory = readiness.load_json(inventory_path)
        smoke = readiness.load_json(readiness.DATA / 'smoke-812c597.json')
        manual_path = readiness.ROOT / '.ci/manual_mechanics_cases.tsv'
        manual = readiness.manual_inventory(manual_path)
        triggers = {'total': 0, 'gate_enabled': [], 'gate_not_enabled': []}
        report = readiness.build_report(inventory, smoke, manual, {'head': None, 'dirty': None},
                                        triggers, hashlib.sha256(inventory_path.read_bytes()).hexdigest(),
                                        hashlib.sha256(manual_path.read_bytes()).hexdigest())
        self.assertIsNone(report['global_readiness_percent'])
        self.assertEqual(report['smoke']['total'], 38)
        self.assertEqual(report['smoke']['counts']['matched_assertions'], 38)
        self.assertEqual(report['manual']['counts']['unknown'], report['manual']['total'])
        self.assertFalse(report['smoke']['current'])


if __name__ == '__main__':
    unittest.main()
