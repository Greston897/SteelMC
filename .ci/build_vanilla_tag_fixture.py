#!/usr/bin/env python3
"""Resolve vanilla 26.3 tag membership independently of Steel's tag loader.

Input: data/minecraft from misode/mcmeta commit
538b2b167248c648b2198f2c0d56eced10dfc0cf (26.3-data).
Usage: python3 .ci/build_vanilla_tag_fixture.py PATH > steel-core/test_assets/26_3_synced_tags.json
Never generate expectations from Steel registry output.
"""
import json
from pathlib import Path
import sys

SOURCE = "538b2b167248c648b2198f2c0d56eced10dfc0cf"
REGISTRIES = ["block", "item", "fluid", "entity_type", "game_event", "worldgen/biome", "damage_type", "enchantment", "instrument", "painting_variant", "banner_pattern", "dialog", "timeline", "point_of_interest_type", "potion"]


def resolve_registry(root):
    raw = {"minecraft:" + p.relative_to(root).with_suffix("").as_posix(): json.loads(p.read_text())["values"] for p in sorted(root.rglob("*.json"))}
    cache = {}

    def resolve(key, active):
        if key in active:
            raise ValueError(f"cyclic vanilla tag: {key}")
        if key in cache:
            return cache[key]
        values = set()
        for entry in raw[key]:
            required = True
            if isinstance(entry, dict):
                required = entry.get("required", True)
                entry = entry["id"]
            if entry.startswith("#"):
                ref = entry[1:]
                ref = ref if ":" in ref else "minecraft:" + ref
                if ref not in raw and not required:
                    continue
                values.update(resolve(ref, active | {key}))
            else:
                # No optional direct entries occur in the pinned source. Do not
                # invent their existence without an independent registry report.
                if not required:
                    raise ValueError(f"optional direct entry requires registry validation: {entry}")
                values.add(entry if ":" in entry else "minecraft:" + entry)
        cache[key] = sorted(values)
        return cache[key]

    return {key: resolve(key, set()) for key in sorted(raw)}


def main():
    data = Path(sys.argv[1])
    result = {"source_commit": SOURCE, "minecraft_version": "26.3", "registries": {"minecraft:" + name: resolve_registry(data / "tags" / name) for name in REGISTRIES}}
    assert all(result["registries"].values()), "missing vanilla tag directory"
    print(json.dumps(result, sort_keys=True, indent=2))


if __name__ == "__main__":
    main()
