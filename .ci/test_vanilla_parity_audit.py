#!/usr/bin/env python3
"""Self-checks for the measured parity audit.

These tests pin the parser rules so the audit cannot silently fabricate a
percentage: they cover name mapping, Java `@Override` extraction, Rust trait
default-body detection, macro-generated impls and the unknown-source path.
"""
from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import vanilla_parity_audit as audit  # noqa: E402

ROOT = audit.ROOT


class TestNameMapping(unittest.TestCase):
    def test_camel_to_snake(self):
        self.assertEqual(audit.camel_to_snake("getStateForPlacement"), "get_state_for_placement")
        self.assertEqual(audit.camel_to_snake("useItemOn"), "use_item_on")
        self.assertEqual(audit.camel_to_snake("onProjectileHit"), "on_projectile_hit")
        self.assertEqual(audit.camel_to_snake("entityInside"), "entity_inside")
        self.assertEqual(audit.camel_to_snake("isPathfindable"), "is_pathfindable")

    def test_special_cases_win_over_generic_conversion(self):
        self.assertEqual(audit.candidate_method("neighborChanged"), "handle_neighbor_changed")
        self.assertEqual(audit.candidate_method("getTicker"), "get_block_entity_ticker")
        self.assertEqual(audit.candidate_method("getStateForPlacement"), "get_state_for_placement")

    def test_unmapped_name_is_still_returned_as_candidate(self):
        # Unknown mappings are reported as `unknown`, never dropped.
        self.assertEqual(audit.candidate_method("someNewVanillaMethod"), "some_new_vanilla_method")


class TempFileMixin:
    def setUp(self):
        self.tmp = ROOT / ".ci" / "_parity_audit_test"
        self.tmp.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        for child in sorted(self.tmp.rglob("*"), reverse=True):
            if child.is_file():
                child.unlink()
            elif child.is_dir():
                child.rmdir()
        if self.tmp.exists():
            self.tmp.rmdir()


class TestJavaParsing(TempFileMixin, unittest.TestCase):
    def test_overrides_extracted(self):
        src = """
        class Foo {
            @Override
            public void tick() {}
            @Override
            protected InteractionResult useItemOn(ItemStack stack) { return null; }
            @Override
            public @Nullable BlockState getStateForPlacement(BlockPlaceContext ctx) { return null; }
            @Override public int getSignal() { return 0; }
        }
        """
        path = self.tmp / "Foo.java"
        path.write_text(src)
        self.assertEqual(
            audit.parse_overrides(path),
            ["tick", "useItemOn", "getStateForPlacement", "getSignal"],
        )

    def test_annotation_on_field_is_not_a_method(self):
        src = """
        class Foo {
            @Override public static final int CONST = 1;
            @SuppressWarnings("x") private int field;
        }
        """
        path = self.tmp / "Foo.java"
        path.write_text(src)
        self.assertEqual(audit.parse_overrides(path), [])

    def test_annotation_arguments_do_not_confuse_the_scan(self):
        src = """
        class Foo {
            @Override @Something(value = 3) public void tick() {}
        }
        """
        path = self.tmp / "Foo.java"
        path.write_text(src)
        self.assertEqual(audit.parse_overrides(path), ["tick"])


class TestRustParsing(TempFileMixin, unittest.TestCase):
    def test_trait_required_and_default_bodies(self):
        src = """
        pub trait FakeBehavior {
            fn required(&self) -> Option<u32>;
            fn defaulted(&self, x: u8) -> bool { x > 0 }
            fn with_arrow(&self) -> Vec<ItemStack> { Vec::new() }
            fn no_return(&self) {}
        }
        """
        path = self.tmp / "fake.rs"
        path.write_text(src)
        methods = audit.parse_trait_methods(path, "FakeBehavior")
        self.assertEqual(
            methods,
            {
                "required": False,
                "defaulted": True,
                "with_arrow": True,
                "no_return": True,
            },
        )

    def test_macro_generated_impl_is_credited(self):
        src = """
        use steel_macros::block_behavior;
        #[block_behavior]
        pub struct DispenserBlock { block: BlockRef }
        macro_rules! funnel_like {
            ($name:ident) => {
                impl BlockBehavior for $name {
                    fn get_state_for_placement(&self) -> Option<BlockStateId> { None }
                    fn tick(&self) {}
                }
            };
        }
        funnel_like!(DispenserBlock);
        """
        path = self.tmp / "dispenser.rs"
        path.write_text(src)
        infos = audit.parse_steel_behavior(path, "block_behavior", "BlockBehavior")
        self.assertEqual(len(infos), 1)
        self.assertTrue(infos[0]["macro_expanded"])
        self.assertIn("get_state_for_placement", infos[0]["methods"])
        self.assertIn("tick", infos[0]["methods"])

    def test_direct_impl_is_parsed(self):
        src = """
        use steel_macros::block_behavior;
        #[block_behavior(class = "CampfireBlock")]
        pub struct CampfireBlock { block: BlockRef }
        impl BlockBehavior for CampfireBlock {
            fn use_item_on(&self) -> InteractionResult { InteractionResult::Success }
        }
        """
        path = self.tmp / "campfire.rs"
        path.write_text(src)
        infos = audit.parse_steel_behavior(path, "block_behavior", "BlockBehavior")
        self.assertEqual(infos[0]["class_name"], "CampfireBlock")
        self.assertEqual(infos[0]["methods"], ["use_item_on"])


class TestAuditDomain(unittest.TestCase):
    def test_missing_class_is_reported_not_hidden(self):
        steel = {
            "Present": {
                "class_name": "Present",
                "struct_name": "Present",
                "file": "x.rs",
                "methods": ["tick"],
                "trait_names": ["BlockBehavior"],
                "macro_expanded": False,
            }
        }
        result = audit.audit_domain("blocks", [{"name": "a", "class": "Present"}], steel, None)
        self.assertEqual(result["missing_behavior_classes"], [])
        self.assertIsNone(result["method_coverage"])

    def test_absent_vanilla_source_never_fabricates_coverage(self):
        steel = {}
        result = audit.audit_domain("blocks", [{"name": "a", "class": "Missing"}], steel, None)
        self.assertEqual(result["method_coverage"], None)
        self.assertEqual(result["missing_behavior_classes"], ["Missing"])

    def test_handled_elsewhere_is_not_missing(self):
        steel = {}
        result = audit.audit_domain("entities", [{"name": "a", "class": "Player"}], steel, None)
        self.assertEqual(result["missing_behavior_classes"], [])
        self.assertEqual(result["handled_elsewhere"], ["Player"])


if __name__ == "__main__":
    unittest.main()
