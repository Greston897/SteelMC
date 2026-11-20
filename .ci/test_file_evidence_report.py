import tempfile
from pathlib import Path
import unittest
from file_evidence_report import build, masked_code, render


class FileEvidenceTests(unittest.TestCase):
    def fixture(self, files, observations):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        for name, text in files.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        return build({'tests': observations}, root, 'a' * 40, 'https://example.invalid/run', list(files))

    def test_no_source_or_import_reference_can_become_a_runtime_pass(self):
        data = self.fixture({'steel-core/src/world/one.rs': 'pub fn tick() {}',
                             'steel-core/src/world/two.rs': 'use super::one;'}, [])
        self.assertEqual(len(data['files']), 2)
        self.assertTrue(all(f['evidence'] == 'NO_OWN_EXECUTED_TEST_EVIDENCE' for f in data['files']))
        self.assertEqual(data['rows'][1]['successful_case_percent'], None)

    def test_synthetic_players_are_not_excluded_and_headless_ignored_remain_in_denominator(self):
        data = self.fixture({'steel-core/src/player/tests.rs': '#[test] fn automatic_player() {}\n#[test]\n#[ignore = "slow"] fn slow() {}'},
                            [{'crate': 'steel_core', 'name': 'player::tests::automatic_player', 'status': 'passed'},
                             {'crate': 'steel_core', 'name': 'player::tests::slow', 'status': 'ignored'}])
        row = next(row for row in data['rows'] if row['area'] == 'Игрок и соединение')
        self.assertEqual(row['successful_case_percent'], 50)
        self.assertEqual(row['observed_test_denominator'], 2)
        self.assertEqual(data['real_client_cases_included'], 0)
        self.assertEqual(data['files'][0]['counts'], {'passed': 1, 'failed': 0, 'ignored': 1})

    def test_ambiguous_names_are_never_assigned_arbitrarily(self):
        data = self.fixture({'steel-core/src/world/one.rs': '#[test] fn same() {}',
                             'steel-core/src/world/two.rs': '#[test] fn same() {}'},
                            [{'crate': 'steel_core', 'name': 'world::one::same', 'status': 'passed'}])
        self.assertEqual(len(data['unmatched_observations']), 1)
        self.assertEqual(data['unmatched_observations'][0]['reason'], 'ambiguous')
        self.assertTrue(all(not f['observed_tests'] for f in data['files']))

    def test_exact_doctest_path_and_integration_binary_are_attributed(self):
        data = self.fixture({'steel-core/tests/biome_hashes.rs': '#[test] fn check_biome() {}',
                             'steel-utils/src/lib.rs': 'pub fn library() {}'},
                            [{'crate': 'biome_hashes', 'name': 'check_biome', 'status': 'ignored'},
                             {'crate': 'steel_utils:doctest', 'name': 'steel-utils/src/lib.rs - (line 1)', 'status': 'passed'}])
        self.assertEqual(data['unmatched_observations'], [])
        self.assertEqual(data['files'][0]['evidence'], 'OWN_TEST_IGNORED_NOT_EXECUTED')
        self.assertIn('NO_OWN_EXECUTED_TEST_EVIDENCE', render(self.fixture({'steel-core/src/lib.rs': ''}, [])))

    def test_comments_and_string_examples_are_not_test_declarations(self):
        source = '// #[test] fn comment() {}\nconst SAMPLE: &str = r#"#[test] fn example() {}"#;\n#[test] fn real() {}'
        masked = masked_code(source)
        self.assertNotIn('comment', masked)
        self.assertNotIn('example', masked)
        self.assertIn('real', masked)
        data = self.fixture({'steel-core/src/lib.rs': source}, [])
        self.assertEqual([t['name'] for t in data['files'][0]['declared_tests']], ['real'])

    def test_unknown_status_cannot_be_published(self):
        with self.assertRaises(ValueError):
            self.fixture({}, [{'crate': 'steel_core', 'name': 'x', 'status': 'assumed_pass'}])


if __name__ == '__main__':
    unittest.main()
