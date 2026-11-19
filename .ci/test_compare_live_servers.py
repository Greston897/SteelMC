import copy
import unittest
from compare_live_servers import compare


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.steel = {'engine': 'steel', 'results': [
            {'name': 'boot_real_server', 'passed': True},
            {'name': 'second_clean_shutdown', 'passed': True}]}
        self.vanilla = copy.deepcopy(self.steel)
        self.vanilla['engine'] = 'vanilla-26.3'

    def test_equal_complete_runs_pass_without_claiming_global_parity(self):
        result = compare(self.steel, self.vanilla)
        self.assertEqual(result['compared_scenarios'], 2)
        self.assertFalse(result['complete_vanilla_parity'])

    def test_failures_truncation_and_duplicate_results_cannot_pass(self):
        for rows in [[], self.steel['results'][:1],
                     self.steel['results'] + self.steel['results'],
                     [{'name': 'second_clean_shutdown', 'passed': False}],
                     [{'name': 'second_clean_shutdown', 'passed': 'true'}]]:
            with self.subTest(rows=rows), self.assertRaises(ValueError):
                compare(self.steel, {'engine': 'vanilla-26.3', 'results': rows})

    def test_different_scenarios_and_provenance_rejected(self):
        self.vanilla['results'][0]['name'] = 'different_test'
        with self.assertRaises(ValueError):
            compare(self.steel, self.vanilla)
        with self.assertRaises(ValueError):
            compare(self.steel, self.steel)
