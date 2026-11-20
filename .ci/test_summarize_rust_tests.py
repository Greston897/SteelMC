import unittest
from summarize_rust_tests import summarize


class RustReportTests(unittest.TestCase):
    def test_every_outcome_keeps_crate_module_and_name(self):
        report = summarize('''Running `/tmp/target/debug/deps/steel_core-abc123`
\x1b[32mtest player::combat::critical ... ok\x1b[0m
test world::raid::roundtrip ... FAILED
test world::heavy ... ignored, expensive
Doc-tests steel_utils
test steel-utils/src/lib.rs - (line 2) ... ok
''')
        self.assertEqual(report['totals'], {'passed': 2, 'failed': 1, 'ignored': 1})
        self.assertEqual(report['groups']['steel_core/player'], {'passed': 1})
        self.assertEqual(report['tests'][-1]['crate'], 'steel_utils:doctest')
        self.assertFalse(report['complete_vanilla_parity'])

    def test_failed_compilation_or_headings_are_not_test_results(self):
        for log in ['', 'error: could not compile steel-core', 'test result: ok. 99 passed']:
            with self.subTest(log=log), self.assertRaises(ValueError):
                summarize(log)

    def test_unknown_crate_is_explicit_not_invented(self):
        self.assertEqual(summarize('test world::save ... ok')['tests'][0]['crate'], 'unknown')


    def test_standard_cargo_runner_headers(self):
        for header in [
            'Running unittests src/lib.rs (target/debug/deps/steel_core-0123456789abcdef)',
            'Running unittests src/lib.rs (target/debug/deps/steel_core/0123456789abcdef/steel_core)',
            'Running `/tmp/target/debug/deps/steel_core/0123456789abcdef/steel_core`',
            'Running `/tmp/custom/steel_core-0123456789abcdef`',
            'Running `CARGO=/bin/cargo /tmp/custom/steel_core-0123456789abcdef`',
        ]:
            with self.subTest(header=header):
                report = summarize(header + '\ntest world::raid::case ... ok')
                self.assertEqual(report['tests'][0]['crate'], 'steel_core')
