import tempfile
from pathlib import Path
import unittest
from file_evidence_report import attach_coverage, build, masked_code, render


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
                            [{'crate': 'steel_core', 'name': 'world::alias::same', 'status': 'passed'}])
        self.assertEqual(len(data['unmatched_observations']), 1)
        self.assertEqual(data['unmatched_observations'][0]['reason'], 'ambiguous')
        self.assertTrue(all(not f['observed_tests'] for f in data['files']))

    def test_duplicate_leaf_resolves_only_with_matching_compiler_module_prefix(self):
        data = self.fixture({'steel-core/src/world/one.rs': '#[test] fn same() {}',
                             'steel-core/src/world/two.rs': '#[test] fn same() {}'},
                            [{'crate': 'steel_core', 'name': 'world::two::tests::same', 'status': 'passed'}])
        self.assertEqual(data['unmatched_observations'], [])
        self.assertEqual(data['files'][0]['counts']['passed'], 0)
        self.assertEqual(data['files'][1]['counts']['passed'], 1)

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

    def test_instrumented_line_counters_do_not_turn_missing_files_into_passes(self):
        data = self.fixture({'steel-core/src/world/one.rs': 'pub fn one() {}',
                             'steel-core/src/world/two.rs': 'pub fn two() {}'}, [])
        llvm = {'data': [{'files': [{'filename': '/repo/steel-core/src/world/one.rs',
                    'summary': {'lines': {'covered': 3, 'count': 4}}}]}]}
        attach_coverage(data, llvm, Path('/repo'))
        row = next(row for row in data['rows'] if row['area'] == 'Мир и тики')
        self.assertEqual(row['instrumented_lines']['percent'], 75)
        self.assertEqual(row['instrumented_lines']['files_not_in_llvm_export'], 1)
        self.assertIsNone(data['files'][1]['instrumented_lines'])
        self.assertIn('NOT_IN_LLVM_EXPORT', render(data))
        self.assertEqual(data['files'][0]['evidence'], 'NO_OWN_EXECUTED_TEST_EVIDENCE')

    def test_empty_coverage_export_is_not_a_green_zero_case_run(self):
        data = self.fixture({'steel-core/src/lib.rs': ''}, [])
        with self.assertRaises(ValueError):
            attach_coverage(data, {'data': []}, Path('/repo'))

    def test_expected_panic_suffix_is_not_part_of_the_function_name(self):
        data = self.fixture({'steel-core/src/lib.rs': '#[test] #[should_panic] fn reject() {}'},
                            [{'crate': 'steel_core', 'name': 'tests::reject - should panic', 'status': 'passed'}])
        self.assertEqual(data['unmatched_observations'], [])
        self.assertEqual(data['files'][0]['counts']['passed'], 1)

    def test_explicit_path_imported_build_tests_use_the_integration_target(self):
        data = self.fixture({'steel-worldgen/build/density/types.rs': '#[test] fn rarity() {}',
                             'steel-worldgen/tests/density_codegen.rs': '#[path = "../build/density/types.rs"] mod density;'},
                            [{'crate': 'density_codegen', 'name': 'density::tests::rarity', 'status': 'passed'}])
        self.assertEqual(data['unmatched_observations'], [])
        self.assertEqual(data['files'][0]['counts']['passed'], 1)

    def test_unknown_status_cannot_be_published(self):
        with self.assertRaises(ValueError):
            self.fixture({}, [{'crate': 'steel_core', 'name': 'x', 'status': 'assumed_pass'}])


if __name__ == '__main__':
    unittest.main()
