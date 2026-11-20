import unittest
from build_manual_mechanics_guide import load_cases, render, source_objects
from subsystem_report import count_tests


class ManualPlanTests(unittest.TestCase):
    def test_catalog_has_explicit_actions_and_expectations_across_kernel(self):
        cases = load_cases()
        self.assertGreaterEqual(len(cases), 75)
        self.assertGreaterEqual(len({case['area'] for case in cases}), 15)
        self.assertEqual(len({case['id'] for case in cases}), len(cases))
        self.assertTrue(all(len(case['action']) > 25 and len(case['expected']) > 25 for case in cases))

    def test_object_queue_only_references_existing_cases(self):
        ids = {case['id'] for case in load_cases()}
        objects = source_objects()
        self.assertGreater(len(objects), 400)
        self.assertTrue(all(set(obj['cases']) <= ids for obj in objects))
        text = render()
        self.assertIn('NOT RUN', text)
        self.assertIn('не исчерпывающая спецификация', text)
        self.assertEqual(text, render())

    def test_runtime_counts_never_infer_test_outcomes_from_source(self):
        tests = [{'crate': 'steel_core', 'name': 'world::raid::one', 'status': 'passed'},
                 {'crate': 'steel_core', 'name': 'worldgen::one', 'status': 'failed'},
                 {'crate': 'steel_core', 'name': 'world::heavy', 'status': 'ignored'}]
        self.assertEqual(count_tests(tests, ['steel_core/world::']), {'passed': 1, 'failed': 0, 'ignored': 1})
        self.assertEqual(count_tests([], ['steel_core/entity']), {'passed': 0, 'failed': 0, 'ignored': 0})
