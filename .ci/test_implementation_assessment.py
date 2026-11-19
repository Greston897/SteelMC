"""Checks accounting and evidence rendering, not gameplay correctness."""
import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('assessment', Path(__file__).with_name('implementation_assessment.py'))
assessment = importlib.util.module_from_spec(spec)
spec.loader.exec_module(assessment)


class AssessmentTests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads((assessment.ROOT / '.ci/implementation_assessment.json').read_text())

    def test_kernel_accounting_and_confirmed_evidence_render(self):
        self.assertEqual(len(self.data['areas']), 15)
        criteria = [c for area in self.data['areas'] for c in area['criteria']]
        self.assertEqual(len(criteria), 60)
        self.assertEqual(sum(c['score'] for c in criteria), 49.5)
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            assessment.main()
        self.assertIn('82.5%', output.getvalue())
        for update in self.data['verified_updates']:
            self.assertIn(str(update['run']), output.getvalue())
            self.assertIn(update['limit'], output.getvalue())

    def test_unconfirmed_result_cannot_enter_verified_ledger(self):
        for result in ['failure', 'queued', 'in_progress']:
            data = copy.deepcopy(self.data)
            data['verified_updates'][0]['result'] = result
            with self.assertRaises(AssertionError):
                assessment.validate_updates(data)

    def test_missing_source_and_duplicate_run_rejected(self):
        data = copy.deepcopy(self.data)
        data['verified_updates'][0]['sources'] = ['no-such-assessment-source']
        with self.assertRaises(AssertionError):
            assessment.validate_updates(data)
        data = copy.deepcopy(self.data)
        data['verified_updates'].append(data['verified_updates'][0])
        with self.assertRaises(AssertionError):
            assessment.validate_updates(data)


if __name__ == '__main__':
    unittest.main()
