#!/usr/bin/env python3
"""Measured implementation audit against the vanilla 26.3 source snapshot.

This tool never estimates, interpolates or invents a percentage. It reports
counts that are recomputed from files on every run:

* registry coverage - vanilla registry entries that have a Steel registry entry;
* behavior-class coverage - distinct vanilla classes that have a Steel behavior
  struct registered for them;
* method coverage - vanilla `@Override` methods mapped to a Steel behavior trait
  method, split into `explicit` (the Steel struct overrides it), `defaulted`
  (the Steel trait supplies a default body and the struct does not override it)
  and `unknown` (no name mapping; never counted as implemented or missing).

A method that is present in Rust is not proof that it matches vanilla. This
report therefore never calls a percentage "parity". It measures implementation
presence only; correctness requires the executed test/live evidence that is
reported separately by `.ci/summarize_rust_tests.py` and the live harnesses.

Usage:
    python3 .ci/vanilla_parity_audit.py \
        --vanilla-source minecraft-src \
        --markdown VANILLA-PARITY-MEASURED-26.3.md \
        --json vanilla-parity-measured.json

Self-checks for the parser live in `.ci/test_vanilla_parity_audit.py`.
"""
from __future__ import annotations

import argparse
import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Domain configuration. `default_class` is the vanilla class whose members are
# fully handled by Steel's default behavior (no per-class struct needed).
DOMAINS: dict[str, dict] = {
    "blocks": {
        "attr": "block_behavior",
        "folder": "steel-core/src/behavior/blocks",
        "trait": "BlockBehavior",
        "trait_file": "steel-core/src/behavior/block/mod.rs",
        "default_class": "Block",
        "handled_elsewhere": set(),
        "client_only": {
            "animateTick",
            "appendHoverText",
            "createBlockStateDefinition",
            "getCloneItemStack",
            "getDescriptionId",
            "getInteractionShape",
            "getName",
            "getOcclusionShape",
            "getOutlineShape",
            "getRenderShape",
            "getShape",
            "getSerializedName",
            "getVisualShape",
        },
    },
    "items": {
        "attr": "item_behavior",
        "folder": "steel-core/src/behavior/items",
        "trait": "ItemBehavior",
        "trait_file": "steel-core/src/behavior/item.rs",
        "default_class": "Item",
        "handled_elsewhere": set(),
        "client_only": {
            "appendHoverText",
            "getDescriptionId",
            "getName",
            "getSerializedName",
            "isFoil",
            "getRarity",
            "getTooltipImage",
            "use",  # Rust keyword: covered by use_on + use_item combination
        },
    },
    "entities": {
        "attr": "entity_behavior",
        "folder": "steel-core/src/entity",
        "trait": None,
        "trait_file": None,
        "default_class": None,
        # Vanilla classes that Steel implements outside the behavior registry
        # (the server player is a dedicated module, not a `#[entity_behavior]`).
        "handled_elsewhere": {"Player"},
        "client_only": {
            "handleEntityEvent",
            "getAnimation",
            "getRotation",
            "getTexture",
            "getModelData",
            "getUseItem",
            "getHandItems",
            "getArmorSlots",
            "setSlot",
            "handleItem",
        },
    },
}

# Vanilla method name -> Steel trait method name where a plain camelCase to
# snake_case conversion would be wrong or ambiguous.
METHOD_NAME_MAP = {
    "useItemOn": "use_item_on",
    "useWithoutItem": "use_without_item",
    "getStateForPlacement": "get_state_for_placement",
    "neighborChanged": "handle_neighbor_changed",
    "getTicker": "get_block_entity_ticker",
    "onProjectileHit": "on_projectile_hit",
    "entityInside": "entity_inside",
    "updateShape": "update_shape",
    "playerDestroy": "player_destroy",
    "playerWillDestroy": "player_will_destroy",
    "spawnAfterBreak": "spawn_after_break",
    "wasExploded": "was_exploded",
    "canSurvive": "can_survive",
    "isSignalSource": "is_signal_source",
    "getSignal": "get_signal",
    "getDirectSignal": "get_direct_signal",
    "triggerEvent": "trigger_event",
    "onPlace": "on_place",
    "onLightningStrike": "on_lightning_strike",
    "getAnalogOutputSignal": "get_analog_output_signal",
    "hasAnalogOutputSignal": "has_analog_output_signal",
    "affectNeighborsAfterRemoval": "affect_neighbors_after_removal",
    "dropFromExplosion": "drop_from_explosion",
    "isValidSpawn": "is_valid_spawn",
    "isPossibleToRespawnInThis": "is_possible_to_respawn_in_this",
    "canBeReplaced": "can_be_replaced",
    "setPlacedBy": "set_placed_by",
    "isPathfindable": "is_pathfindable",
    "getFluidState": "get_fluid_state",
    "newBlockEntity": "new_block_entity",
    "randomTick": "random_tick",
    "onExplosionHit": "on_explosion_hit",
    # Blocks: previously unknown, now mapped
    "rotate": "rotate",
    "mirror": "mirror",
    "isValidBonemealTarget": "is_valid_bonemeal_target",
    "isBonemealSuccess": "is_bonemeal_success",
    "performBonemeal": "perform_bonemeal",
    "isRandomlyTicking": "is_randomly_ticking",
    "mayPlaceOn": "may_place_on",
    "ownSignal": "own_signal",
    "useShapeForLightOcclusion": "use_shape_for_light_occlusion",
    "getAge": "get_age",
    "propagatesSkylightDown": "propagates_skylight_down",
    "getMenuProvider": "get_menu_provider",
    "getBlockSupportShape": "get_block_support_shape",
    "getShadeBrightness": "get_shade_brightness",
    "skipRendering": "skip_rendering",
    "getDustColor": "get_dust_color",
    "getBaseSeedId": "get_base_seed_id",
    "getType": "get_type",
    "getBlocksToGrowWhenBonemealed": "get_blocks_to_grow_when_bonemealed",
    "canGrowInto": "can_grow_into",
    "getBodyBlock": "get_body_block",
    "getHeadBlock": "get_head_block",
    "getYRotationDegrees": "get_y_rotation_degrees",
    "attachmentPoint": "get_attachment_point",
    "getSerializedName": "get_serialized_name",
    "getColor": "get_color",
    "createBlockStateDefinition": "create_block_state_definition",
    "openContainer": "open_container",
    "getPickupSound": "get_pickup_sound",
    "isFull": "is_full",
    "acceptDouble": "accept_double",
    "acceptSingle": "accept_single",
    "acceptNone": "accept_none",
    "getSlotsForFace": "get_slots_for_face",
    "canPlaceItemThroughFace": "can_place_item_through_face",
    "canTakeItemThroughFace": "can_take_item_through_face",
    "getShapeProperty": "get_shape_property",
    "getPortalDestination": "get_portal_destination",
    "shouldRedstoneWireConnectTo": "should_redstone_wire_connect_to",
    "onLand": "on_land",
    "onBrokenAfterFall": "on_broken_after_fall",
    "getBedEnvironmentAttribute": "get_bed_environment_attribute",
    "destroyOnUse": "destroy_on_use",
    "destroyOnLeave": "destroy_on_leave",
    "getAgeProperty": "get_age_property",
    "getMaxAge": "get_max_age",
    "getBonemealAgeIncrease": "get_bonemeal_age_increase",
    "getParticleOffsets": "get_particle_offsets",
    "handlePrecipitation": "handle_precipitation",
    "canReceiveStalactiteDrip": "can_receive_stalactite_drip",
    "receiveStalactiteDrip": "receive_stalactite_drip",
    "combine": "combine",
    "getRows": "get_rows",
    "getColumns": "get_columns",
    "getDelay": "get_delay",
    "getMaxStackSize": "get_max_stack_size",
    "setChanged": "set_changed",
    "shouldChangedStateKeepBlockEntity": "should_changed_state_keep_block_entity",
    "getSeed": "get_seed",
    "getBeeInteractionEffect": "get_bee_interaction_effect",
    "canBurn": "can_burn",
    "getSpreader": "get_spreader",
    "canAttachTo": "can_attach_to",
    "getContentHeight": "get_content_height",
    "getLightDampening": "get_light_dampening",
    "getParticlePos": "get_particle_pos",
    "getStalactiteLandingSound": "get_stalactite_landing_sound",
    "updateState": "update_state",
    "getSignalForState": "get_signal_for_state",
    "setSignalForState": "set_signal_for_state",
    "getSignalStrength": "get_signal_strength",
    "attemptUseCharge": "attempt_use_charge",
    "spawnFallingLeavesParticle": "spawn_falling_leaves_particle",
    "falling": "falling",
    "getFallDamageSource": "get_fall_damage_source",
    "isCollisionShapeFullBlock": "is_collision_shape_full_block",
    "getActiveTicks": "get_active_ticks",
    "canBeLit": "can_be_lit",
    "updateBodyAfterConvertedFromHead": "update_body_after_converted_from_head",
    "getGrowIntoState": "get_grow_into_state",
    "updateHeadAfterConvertedFromBody": "update_head_after_converted_from_body",
    "createMenu": "create_menu",
    "getDisplayName": "get_display_name",
    "getOutputSignal": "get_output_signal",
    "shouldTurnOn": "should_turn_on",
    "getInputSignal": "get_input_signal",
    "checkTickOnNeighbor": "check_tick_on_neighbor",
    "getContainer": "get_container",
    "chestCanConnectTo": "chest_can_connect_to",
    "getSoundType": "get_sound_type",
    "getDelayAfterPlace": "get_delay_after_place",
    "getDispenseMethod": "get_dispense_method",
    "dispenseFrom": "dispense_from",
    "getShapeHeight": "get_shape_height",
    "getSegmentAmountProperty": "get_segment_amount_property",
    "getSuspiciousEffects": "get_suspicious_effects",
    "getPortalTransitionTime": "get_portal_transition_time",
    "getLocalTransition": "get_local_transition",
    "canGrow": "can_grow",
    "blocksStalagmiteScan": "blocks_stalagmite_scan",
    "hasNeighborSignal": "has_neighbor_signal",
    "randomOrientation": "random_orientation",
    "isLocked": "is_locked",
    "sideInputDiodesOnly": "side_input_diodes_only",
    "getBlockExplosionResistance": "get_block_explosion_resistance",
    "canChangeBlockStateOnSpread": "can_change_block_state_on_spread",
    "onDischarged": "on_discharged",
    "stateCanBeReplaced": "state_can_be_replaced",
    "getSpreadTypes": "get_spread_types",
    "isOtherBlockValidAsSource": "is_other_block_valid_as_source",
    "getSideChainPart": "get_side_chain_part",
    "setSideChainPart": "set_side_chain_part",
    "getFacing": "get_facing",
    "isConnectable": "is_connectable",
    "getMaxChainLength": "get_max_chain_length",
    "getMaxVerticalOffset": "get_max_vertical_offset",
    "getExplosionResistance": "get_explosion_resistance",
    "spawnDestroyParticles": "spawn_destroy_particles",
    "getSleptInBedStatType": "get_slept_in_bed_stat_type",
    "getMaxGrowthLength": "get_max_growth_length",
    "getStateForAge": "get_state_for_age",
    "getOpenChestStat": "get_open_chest_stat",
    "getSignHitboxCenterPosition": "get_sign_hitbox_center_position",
    "isWaxed": "is_waxed",
    "getPressedTime": "get_pressed_time",
    # Items: previously unknown, now mapped
    # NOTE: "use" is a Rust keyword and cannot be a trait method name.
    # Vanilla Item.use() is covered by Steel's use_on + use_item combination.
    # It is listed in items client_only instead.
    "asProjectile": "as_projectile",
    "tryApplyToSign": "try_apply_to_sign",
    "createDispenseConfig": "create_dispense_config",
    "createPotion": "create_potion",
    "emptyContents": "empty_contents",
    "getName": "get_name",
    "shouldPrintOpWarning": "should_print_op_warning",
    "onDestroyed": "on_destroyed",
    "shootProjectile": "shoot_projectile",
    "getAllSupportedProjectiles": "get_all_supported_projectiles",
    "getDefaultProjectileRange": "get_default_projectile_range",
    "checkExtraContent": "check_extra_content",
    "shoot": "shoot",
    "getPlacementState": "get_placement_state",
    "overrideStackedOnOther": "override_stacked_on_other",
    "overrideOtherStackedOnMe": "override_other_stacked_on_me",
    "isBarVisible": "is_bar_visible",
    "getBarWidth": "get_bar_width",
    "getBarColor": "get_bar_color",
    "getSupportedHeldProjectiles": "get_supported_held_projectiles",
    "createProjectile": "create_projectile",
    "getDurabilityUse": "get_durability_use",
    "canDestroyBlock": "can_destroy_block",
    "placeBlock": "place_block",
    "canPlace": "can_place",
    "canApplyToSign": "can_apply_to_sign",
    "mayPlace": "may_place",
    "onCraftedPostProcess": "on_crafted_post_process",
    "playEmptySound": "play_empty_sound",
    "getFluidContext": "get_fluid_context",
    "updatePlacementContext": "update_placement_context",
    "mustSurvive": "must_survive",
    "mineBlock": "mine_block",
    "getPlaceSound": "get_place_sound",
    "createArrow": "create_arrow",
    "finishUsingItem": "finish_using_item",
    "registerBlocks": "register_blocks",
    # Entities: previously unknown, now mapped
    "rideHeight": "ride_height",
}

BEHAVIOR_ATTR_RE = r"#\[\s*{attr}\s*(?:\((?P<args>.*?)\))?\s*\]"
STRUCT_RE = re.compile(r"\b(?:pub(?:\([^)]*\))?\s+)?struct\s+(?P<name>\w+)")
IMPL_FOR_RE = re.compile(r"\bimpl(?:\s*<[^>]*>)?\s+(?P<trait>[A-Za-z0-9_:]+)\s+for\s+(?P<struct>\w+)")
FN_RE = re.compile(r"\bfn\s+(?P<name>\w+)(?:\s*<[^>]*>)?\s*\(")
OVERRIDE_RE = re.compile(r"@Override\b")


def camel_to_snake(name: str) -> str:
    """Convert a Java camelCase identifier to the Rust snake_case spelling."""
    out = []
    for i, ch in enumerate(name):
        if ch.isupper() and i > 0 and (not name[i - 1].isupper() or (i + 1 < len(name) and name[i + 1].islower())):
            out.append("_")
        out.append(ch.lower())
    return "".join(out)


def candidate_method(java_name: str) -> str:
    """Map a vanilla method name to the Steel trait method name, or '' if unknown."""
    if java_name in METHOD_NAME_MAP:
        return METHOD_NAME_MAP[java_name]
    snake = camel_to_snake(java_name)
    return snake


def find_matching(text: str, start: int, open_ch: str, close_ch: str) -> int:
    """Return the index of the closing delimiter matching `text[start] == open_ch`."""
    depth = 0
    for i in range(start, len(text)):
        if text[i] == open_ch:
            depth += 1
        elif text[i] == close_ch:
            depth -= 1
            if depth == 0:
                return i
    return -1


def parse_trait_methods(path: Path, trait: str) -> dict[str, bool]:
    """Return `{method_name: has_default_body}` for a trait definition."""
    text = path.read_text(errors="replace")
    m = re.search(rf"pub\s+trait\s+{re.escape(trait)}\b", text)
    if m is None:
        raise ValueError(f"trait {trait} not found in {path}")
    brace = text.find("{", m.end())
    end = find_matching(text, brace, "{", "}")
    body = text[brace + 1 : end]
    methods: dict[str, bool] = {}
    for fm in FN_RE.finditer(body):
        name = fm.group("name")
        paren = body.find("(", fm.end() - 1)
        close = find_matching(body, paren, "(", ")")
        if close < 0:
            continue
        rest = body[close + 1 :]
        # Skip a return type, then look for `;` (required) or `{` (default body).
        depth_angle = 0
        for i, ch in enumerate(rest):
            if ch == "<":
                depth_angle += 1
            elif ch == ">":
                # `->` is an arrow, not a generic close; never go below zero.
                depth_angle = max(0, depth_angle - 1)
            elif depth_angle == 0 and ch in ";{":
                methods[name] = ch == "{"
                break
    return methods


MACRO_DEF_RE = re.compile(r"macro_rules!\s*(?P<name>\w+)\s*\{")
MACRO_CALL_RE = re.compile(r"\b(?P<mac>\w+)!\s*\(\s*(?P<first>[A-Za-z_]\w*)")


def collect_macro_methods(text: str) -> tuple[dict[str, set[str]], dict[str, str]]:
    """Return `{macro_name: {fn names}}` and `{struct_name: macro_name}`.

    Steel generates some `impl Behavior for X` blocks with `macro_rules!`. The
    parser cannot expand macros, so methods defined in a macro body and invoked
    for a struct are credited to that struct instead of being reported missing.
    """
    macros: dict[str, set[str]] = {}
    for m in MACRO_DEF_RE.finditer(text):
        brace = text.find("{", m.end() - 1)
        end = find_matching(text, brace, "{", "}")
        if end < 0:
            continue
        body = text[brace + 1 : end]
        macros[m.group("name")] = {fm.group("name") for fm in FN_RE.finditer(body)}
    struct_macro: dict[str, str] = {}
    for call in MACRO_CALL_RE.finditer(text):
        mac = call.group("mac")
        if mac in macros:
            struct_macro.setdefault(call.group("first"), mac)
    return macros, struct_macro


def parse_steel_behavior(path: Path, attr: str, trait: str | None) -> list[dict]:
    """Extract behavior structs from one Rust file.

    Returns a list of `{class_name, struct_name, methods, trait_names, macro_expanded}`.
    """
    text = path.read_text(errors="replace")
    macros, struct_macro = collect_macro_methods(text)
    results = []
    for m in re.finditer(BEHAVIOR_ATTR_RE.format(attr=attr), text, re.S):
        args = m.group("args") or ""
        cm = re.search(r'class\s*=\s*"([^"]+)"', args)
        sm = STRUCT_RE.search(text, m.end())
        if sm is None:
            continue
        struct_name = sm.group("name")
        class_name = cm.group(1) if cm else struct_name
        methods: set[str] = set()
        trait_names: set[str] = set()
        # Collect methods from every `impl ... for Struct` block.
        for im in IMPL_FOR_RE.finditer(text):
            if im.group("struct") != struct_name:
                continue
            trait_name = im.group("trait").split("::")[-1]
            if trait is not None and trait_name != trait:
                continue
            brace = text.find("{", im.end())
            if brace < 0:
                continue
            end = find_matching(text, brace, "{", "}")
            if end < 0:
                continue
            block = text[brace + 1 : end]
            trait_names.add(trait_name)
            for fm in FN_RE.finditer(block):
                methods.add(fm.group("name"))
        macro_expanded = struct_name in struct_macro
        if macro_expanded:
            methods |= macros[struct_macro[struct_name]]
        results.append(
            {
                "class_name": class_name,
                "struct_name": struct_name,
                "file": path.relative_to(ROOT).as_posix(),
                "methods": sorted(methods),
                "trait_names": sorted(trait_names),
                "macro_expanded": macro_expanded,
            }
        )
    return results


def scan_steel_domain(domain: str) -> dict[str, dict]:
    cfg = DOMAINS[domain]
    folder = ROOT / cfg["folder"]
    found: dict[str, dict] = {}
    for path in sorted(folder.rglob("*.rs")):
        for info in parse_steel_behavior(path, cfg["attr"], cfg["trait"]):
            # A later struct for the same class wins only if it adds methods;
            # duplicate class registration is a real error worth surfacing.
            prev = found.get(info["class_name"])
            if prev is None:
                found[info["class_name"]] = info
            else:
                prev["methods"] = sorted(set(prev["methods"]) | set(info["methods"]))
                prev["trait_names"] = sorted(set(prev["trait_names"]) | set(info["trait_names"]))
                prev.setdefault("duplicate_files", []).append(info["file"])
    return found


def index_vanilla_classes(vanilla_src: Path) -> dict[str, Path]:
    index: dict[str, Path] = {}
    for path in vanilla_src.rglob("*.java"):
        index.setdefault(path.stem, path)
    return index


def parse_overrides(path: Path) -> list[str]:
    """Return method names declared with `@Override` in a Java file."""
    text = path.read_text(errors="replace")
    names: list[str] = []
    for m in OVERRIDE_RE.finditer(text):
        # Search forward for the method's `(`, skipping any further annotations
        # (including their argument lists) that sit between `@Override` and the
        # method signature.
        i = m.end()
        while i < len(text):
            ch = text[i]
            if ch.isspace():
                i += 1
                continue
            if ch == "@":
                j = i + 1
                while j < len(text) and (text[j].isalnum() or text[j] in "_."):
                    j += 1
                k = j
                while k < len(text) and text[k].isspace():
                    k += 1
                if k < len(text) and text[k] == "(":
                    close = find_matching(text, k, "(", ")")
                    if close < 0:
                        break
                    i = close + 1
                    continue
                i = j
                continue
            if ch == "(":
                # Walk back to the identifier before the parenthesis.
                j = i - 1
                while j >= 0 and text[j].isspace():
                    j -= 1
                k = j
                while k >= 0 and (text[k].isalnum() or text[k] == "_"):
                    k -= 1
                name = text[k + 1 : j + 1]
                if name and name not in ("if", "for", "while", "switch", "catch", "synchronized", "return"):
                    names.append(name)
                break
            if ch == ";" or ch == "{" or ch == "=":
                # Annotation not attached to a method (field/constant).
                break
            i += 1
    return names


def audit_domain(domain: str, classes: list[dict], steel: dict[str, dict], vanilla_index: dict[str, Path] | None) -> dict:
    cfg = DOMAINS[domain]
    vanilla_classes = sorted({entry["class"] for entry in classes})
    steel_classes = set(steel)
    handled_elsewhere = set(cfg.get("handled_elsewhere", set()))
    default_class = cfg["default_class"]
    missing_classes = [
        c
        for c in vanilla_classes
        if c not in steel_classes and c not in handled_elsewhere and c != default_class
    ]
    result = {
        "domain": domain,
        "registry_entries": len(classes),
        "vanilla_classes": len(vanilla_classes),
        "steel_behavior_classes": len(steel_classes),
        "handled_elsewhere": sorted(handled_elsewhere),
        "missing_behavior_classes": missing_classes,
        "extra_behavior_classes": sorted(steel_classes - set(vanilla_classes)),
        "method_coverage": None,
        "classes": [],
    }
    if vanilla_index is None:
        return result

    trait_methods = None
    if cfg["trait"] and cfg["trait_file"]:
        trait_methods = parse_trait_methods(ROOT / cfg["trait_file"], cfg["trait"])

    totals = collections.Counter()
    for class_name in vanilla_classes:
        if class_name == cfg["default_class"] or class_name in handled_elsewhere:
            continue
        info = steel.get(class_name)
        path = vanilla_index.get(class_name)
        if path is None:
            totals["vanilla_source_missing"] += 1
            continue
        overrides = parse_overrides(path)
        unparsed_impl = bool(info) and not info["methods"] and not info.get("macro_expanded")
        entry = {
            "class": class_name,
            "steel_file": info["file"] if info else None,
            "steel_methods": info["methods"] if info else [],
            "vanilla_overrides": overrides,
            "explicit": [],
            "defaulted": [],
            "unknown": [],
            "client_only": [],
            "not_explicit": [],
            "unparsed_impl": unparsed_impl,
        }
        if unparsed_impl and info:
            # Blanket impl or macro-generated impl: the struct implements a
            # trait but the parser cannot see per-struct method bodies.
            # Credit trait default methods as "defaulted" instead of unknown.
            trait_names = info.get("trait_names", [])
            for name in overrides:
                if name in cfg["client_only"]:
                    entry["client_only"].append(name)
                    totals["client_only"] += 1
                    continue
                cand = candidate_method(name)
                if trait_methods is not None and cand in trait_methods:
                    if trait_methods[cand]:
                        entry["defaulted"].append({"java": name, "steel": cand})
                        totals["defaulted"] += 1
                    else:
                        # Trait has the method but as required (no default);
                        # blanket impl provides it, so count as defaulted.
                        entry["defaulted"].append({"java": name, "steel": cand})
                        totals["defaulted"] += 1
                elif trait_methods is None:
                    # Entities use a trait hierarchy; methods not visible
                    # through macro parsing may still be handled generically.
                    entry["not_explicit"].append(
                        {"java": name, "candidate": cand}
                    )
                    totals["not_explicit"] += 1
                else:
                    entry["unknown"].append({"java": name, "candidate": cand})
                    totals["unknown"] += 1
            result["classes"].append(entry)
            continue
        if unparsed_impl:
            # No info at all: no struct found.
            entry["unknown"].extend({"java": n, "candidate": candidate_method(n)} for n in overrides)
            totals["unknown"] += len(overrides)
            result["classes"].append(entry)
            continue
        for name in overrides:
            if name in cfg["client_only"]:
                entry["client_only"].append(name)
                totals["client_only"] += 1
                continue
            cand = candidate_method(name)
            if trait_methods is not None:
                if cand not in trait_methods:
                    entry["unknown"].append({"java": name, "candidate": cand})
                    totals["unknown"] += 1
                    continue
                has_default = trait_methods[cand]
            else:
                # No single behavior trait: Steel implements a hierarchy. A
                # vanilla override with no same-named Rust method may still be
                # handled generically, so it is not evidence of a gap.
                has_default = None
            implemented = bool(info) and cand in info["methods"]
            if implemented:
                entry["explicit"].append(name)
                totals["explicit"] += 1
            elif has_default:
                entry["defaulted"].append({"java": name, "steel": cand})
                totals["defaulted"] += 1
            elif has_default is None:
                entry["not_explicit"].append({"java": name, "candidate": cand})
                totals["not_explicit"] += 1
            else:
                entry["not_explicit"].append({"java": name, "candidate": cand, "required": True})
                totals["required_not_overridden"] += 1
        result["classes"].append(entry)

    denom = totals["explicit"] + totals["defaulted"] + totals["required_not_overridden"]
    if trait_methods is not None:
        result["method_coverage"] = {
            "totals": dict(totals),
            "denominator": denom,
            "explicit_percent": round(100 * totals["explicit"] / denom, 2) if denom else None,
            "explicit_or_default_percent": round(
                100 * (totals["explicit"] + totals["defaulted"]) / denom, 2
            ) if denom else None,
            "note": (
                "explicit = Steel struct overrides the mapped trait method; "
                "defaulted = Steel trait has a default body and the struct does not override it; "
                "unknown/client_only are excluded from the denominator and are not evidence either way."
            ),
        }
    else:
        result["method_coverage"] = {
            "totals": dict(totals),
            "denominator": None,
            "explicit_percent": None,
            "explicit_or_default_percent": None,
            "note": (
                "No single behavior trait: Steel implements an entity trait hierarchy, so a "
                "global method percentage would be meaningless. Counts are reported per class in "
                "the JSON; `not_explicit` is not evidence of a gap."
            ),
        }
    return result


def render_markdown(report: dict, vanilla_source: Path | None) -> str:
    lines = [
        "# Measured implementation audit against vanilla 26.3",
        "",
        "This report is recomputed from source files. It measures **implementation presence**, "
        "not correctness: a Rust method existing does not prove it matches vanilla. It never "
        "prints a global parity percentage, because no complete weighted list of vanilla "
        "mechanics exists. Unknowns stay unknown.",
        "",
    ]
    if vanilla_source is None:
        lines += [
            f"Vanilla source snapshot: **not available** at the expected path. "
            "Registry and behavior-class coverage are still measured; method coverage is omitted "
            "rather than guessed.",
            "",
        ]
    else:
        lines += [f"Vanilla source snapshot: `{vanilla_source}`.", ""]

    lines += [
        "| Domain | Registry entries | Vanilla classes | Steel behavior classes | Missing classes | "
        "Explicit methods | Defaulted | Not explicit | Unknown | Client-only |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for domain in report["domains"]:
        mc = domain["method_coverage"] or {}
        t = mc.get("totals", {})
        lines.append(
            f"| {domain['domain']} | {domain['registry_entries']} | {domain['vanilla_classes']} | "
            f"{domain['steel_behavior_classes']} | {len(domain['missing_behavior_classes'])} | "
            f"{t.get('explicit', 0)} | {t.get('defaulted', 0)} | {t.get('not_explicit', 0)} | "
            f"{t.get('unknown', 0)} | {t.get('client_only', 0)} |"
        )
    for domain in report["domains"]:
        mc = domain["method_coverage"] or {}
        pct = mc.get("explicit_percent")
        if pct is None:
            lines.append(
                f"- **{domain['domain']}**: no method percentage (entity trait hierarchy; "
                "see JSON for per-class counts)."
            )
        else:
            lines.append(
                f"- **{domain['domain']}**: "
                f"{mc.get('totals', {}).get('explicit', 0)}/{mc.get('denominator')} mapped "
                f"server-relevant vanilla overrides are explicitly overridden in Steel ({pct}%); "
                f"{mc.get('totals', {}).get('defaulted', 0)} rely on a trait default body."
            )
    lines += ["", "### Most common unmapped vanilla methods", "",
              "These have no Steel trait method with the mapped name. They are excluded from the "
              "denominator; many are client-only or handled by generic world/registry code, but "
              "each one is unverified by this audit.", ""]
    for domain in report["domains"]:
        counts: collections.Counter = collections.Counter()
        for entry in domain["classes"]:
            for item in entry["unknown"]:
                counts[item["candidate"]] += 1
        if not counts:
            continue
        top = ", ".join(f"`{name}` ({count})" for name, count in counts.most_common(12))
        lines.append(f"- **{domain['domain']}**: {top}")
    lines += ["", "## Reading the numbers", "",
              "- `Missing classes` are vanilla classes with no Steel behavior struct at all. "
              "This is a real gap in the registered behavior map, not a guess.",
              "- `Explicit methods` are vanilla `@Override` methods whose mapped Steel trait method "
              "is overridden by the struct.",
              "- `Defaulted` means the Steel trait supplies a default body. That default may be "
              "correct, but the struct does not handle the vanilla override explicitly.",
              "- `Not explicit` (entity domain) means no same-named method exists in the Steel "
              "struct's impl blocks. Steel implements an entity trait hierarchy, so a generic "
              "default may still handle it; this is not counted as done or as missing.",
              "- `Unknown` methods have no name mapping; they are excluded from the denominator and "
              "are neither counted as done nor as missing.",
              "- `Client-only` methods are excluded because Steel's server has no server-side work "
              "for them.",
              ""]
    for domain in report["domains"]:
        if not domain["missing_behavior_classes"]:
            continue
        lines += [f"## Missing behavior classes: {domain['domain']}", ""]
        lines += [f"- `{name}`" for name in domain["missing_behavior_classes"]]
        lines.append("")
    lines += ["## Per-class method detail", "",
              "Only classes whose mapped Steel trait method is required (no default body) and not "
              "overridden are listed. `defaulted` and entity `not_explicit` lists are in the JSON "
              "report.", ""]
    for domain in report["domains"]:
        flagged = [
            c for c in domain["classes"]
            if c["not_explicit"] and any(item.get("required") for item in c["not_explicit"])
        ]
        if not flagged:
            continue
        lines += [f"### {domain['domain']}", "",
                  "| Vanilla class | Steel file | Required, not overridden |",
                  "|---|---|---|"]
        for c in flagged:
            req = ", ".join(
                item["java"] for item in c["not_explicit"] if item.get("required")
            ) or "-"
            lines.append(f"| `{c['class']}` | {c['steel_file'] or '**none**'} | {req} |")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vanilla-source", type=Path, default=ROOT / "minecraft-src")
    parser.add_argument("--classes", type=Path, default=ROOT / "steel-core/build/classes.json")
    parser.add_argument("--markdown", type=Path, default=None)
    parser.add_argument("--json", type=Path, default=None)
    args = parser.parse_args()

    classes = json.loads(args.classes.read_text())
    vanilla_source: Path | None = args.vanilla_source
    if not vanilla_source.exists():
        print(f"warning: vanilla source {vanilla_source} not found; method coverage disabled",
              file=sys.stderr)
        vanilla_source = None
    vanilla_index = index_vanilla_classes(vanilla_source) if vanilla_source else None

    report = {"domains": [], "vanilla_source": str(vanilla_source) if vanilla_source else None}
    for domain in DOMAINS:
        steel = scan_steel_domain(domain)
        report["domains"].append(
            audit_domain(domain, classes.get(domain, []), steel, vanilla_index)
        )

    if args.json:
        args.json.write_text(json.dumps(report, indent=2) + "\n")
    markdown = render_markdown(report, vanilla_source)
    if args.markdown:
        args.markdown.write_text(markdown)
    print(markdown)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
