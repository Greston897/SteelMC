"""Tests for the advancement trigger source inventory, not gameplay parity."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('trigger_audit', Path(__file__).with_name('advancement_trigger_audit.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)

JAVA = '''
public static final Trigger FIRST = register("first", new Trigger());
public static final Trigger SECOND = register("second", new Trigger());
public static final Trigger ALIAS = register("vanilla_alias", new Trigger());
'''
RUST = '''
pub fn trigger_with_context() {
    let block_location = trigger == &vanilla_criteria_triggers::FIRST;
    let used_item = trigger == &vanilla_criteria_triggers::FIRST;
    let changed_dimension = trigger == &vanilla_criteria_triggers::FIRST;
    let slept_in_bed = trigger == &vanilla_criteria_triggers::FIRST;
    let nether_travel = trigger == &vanilla_criteria_triggers::FIRST;
    let unused = trigger == &vanilla_criteria_triggers::SECOND;
    if !block_location && !used_item && !changed_dimension && !slept_in_bed && !nether_travel { return 0; }
    let mut granted = 0;
    // Another reference after the gate must not be treated as an enabled trigger.
    trigger == &vanilla_criteria_triggers::SECOND;
}
'''


class TriggerAuditTests(unittest.TestCase):
    def test_inventory_preserves_registry_names_and_only_counts_gate(self):
        self.assertEqual(audit.inventory(JAVA, RUST), [
            ('FIRST', 'first', True),
            ('SECOND', 'second', False),
            ('ALIAS', 'vanilla_alias', False),
        ])

    def test_rejects_missing_or_duplicate_vanilla_declarations(self):
        for java in ('', JAVA + JAVA.splitlines()[1] + '\n', JAVA.replace('"second"', '"first"')):
            with self.subTest(java=java):
                with self.assertRaises(ValueError):
                    audit.inventory(java, RUST)

    def test_rejects_unknown_or_changed_dispatch(self):
        for rust in (RUST.replace('::FIRST', '::UNKNOWN'), RUST.replace('!nether_travel', '!new_condition'), RUST.replace('::FIRST;', '::FIRST && condition;', 1), ''):
            with self.subTest(rust=rust):
                with self.assertRaises(ValueError):
                    audit.inventory(JAVA, rust)


if __name__ == '__main__':
    unittest.main()
