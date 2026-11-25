#!/usr/bin/env python3
"""Reproducible source inventory, not a test runner or parity percentage.

Usage: python3 .ci/progress_inventory.py > PROGRESS-26.3-INVENTORY.md
Only Rust source in src/ and build/ is counted; generated output is excluded.
"""
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parent.parent
TEST = re.compile(r"#\[(?:test|tokio::test(?:\([^\]]*\))?)\]\s*(?:#\[[^\]]*\]\s*)*(?:async\s+)?fn\s+(\w+)")
MARKER = re.compile(r"\b(?:todo|unimplemented)!\s*\(")


def files(path):
    p = ROOT / path
    candidates = [p] if p.is_file() else sorted(p.rglob("*.rs"))
    return [p for p in candidates if p.is_file() and "generated" not in p.relative_to(ROOT).parts]


def stats(paths):
    fs = sorted({p for path in paths for p in files(path)})
    texts = [p.read_text() for p in fs]
    return fs, sum(len(t.splitlines()) for t in texts), sum(len(TEST.findall(t)) for t in texts), sum(len(MARKER.findall(t)) for t in texts)


def row(name, paths):
    fs, lines, tests, markers = stats(paths)
    print(f"| {name} | {len(fs)} | {lines} | {tests} | {markers} | " + ", ".join(f"`{p}`" for p in paths) + " |")


def main():
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    print("# MC 26.3 — воспроизводимая инвентаризация исходников\n")
    print(f"Код на момент замера: `{sha}`. Генератор: `.ci/progress_inventory.py`.\n")
    print("Это **статический подсчёт**, не результаты запуска. Тесты = функции с `#[test]`/`#[tokio::test]`, включая условно компилируемые; макрогенерируемые, doc-тесты и внешние тестовые таргеты не учтены. Маркеры ищутся лексически (включая комментарии). LOC включает комментарии и пустые строки. Числа не являются процентом реализации.\n")
    print("## Крейты\n\n| Крейт | .rs | LOC | Тестовых функций | todo!/unimplemented! | Пути |\n|---|---:|---:|---:|---:|---|")
    members = tomllib.loads((ROOT / "Cargo.toml").read_text().split("[workspace.package]", 1)[0])["workspace"]["members"]
    for member in members:
        row(member, [f"{member}/src", f"{member}/build"])
    print("\n## Зоны ядра (строки могут пересекаться; не суммировать)\n\n| Зона | .rs | LOC | Тестовых функций | Маркеров | Пути |\n|---|---:|---:|---:|---:|---|")
    for name, paths in [
        ("Реестры, теги, кодеки", ["steel-registry/src", "steel-registry/build"]),
        ("Синхронизация конфигурации", ["steel-core/src/server/registry_cache.rs", "steel-core/src/player/connection", "steel-protocol/src/packets/config"]),
        ("Поведение блоков", ["steel-core/src/behavior/blocks", "steel-core/src/behavior/block"]),
        ("Поведение предметов", ["steel-core/src/behavior/items"]),
        ("Block entities", ["steel-core/src/block_entity"]),
        ("Сущности и AI", ["steel-core/src/entity"]),
        ("Игрок и соединение", ["steel-core/src/player"]),
        ("Инвентарь и меню", ["steel-core/src/inventory"]),
        ("Команды и права", ["steel-core/src/command", "steel-core/src/permission"]),
        ("Мир и тик", ["steel-core/src/world", "steel-core/src/server"]),
        ("Чанки, свет, сохранение", ["steel-core/src/chunk", "steel-core/src/chunk_saver", "steel-core/src/level_data"]),
        ("Worldgen", ["steel-core/src/worldgen", "steel-worldgen/src"]),
        ("Loot, зачарования", ["steel-core/src/loot", "steel-core/src/enchantment_helper.rs", "steel-core/src/enchantment_helper"]),
        ("Физика, жидкости, порталы", ["steel-core/src/physics", "steel-core/src/fluid", "steel-core/src/portal"]),
        ("Протокол", ["steel-protocol/src"]),
    ]:
        row(name, paths)
    print("\n## Объекты и игровые зоны (без заявления о полноте поведения)\n\n| Объекты / зона | .rs | LOC | Тестовых функций | Маркеров | Пути |\n|---|---:|---:|---:|---:|---|")
    for name, paths in [
        ("Мирные мобы", ["steel-core/src/entity/entities/mobs/passive"]),
        ("Нейтральные мобы", ["steel-core/src/entity/entities/mobs/neutral"]),
        ("Враждебные мобы", ["steel-core/src/entity/entities/mobs/hostile"]),
        ("Боссы", ["steel-core/src/entity/entities/mobs/bosses"]),
        ("Транспорт", ["steel-core/src/entity/entities/objects/vehicles"]),
        ("Снаряды", ["steel-core/src/entity/entities/objects/projectiles"]),
        ("Взрывчатка", ["steel-core/src/entity/entities/objects/explosives"]),
        ("Предметы в мире", ["steel-core/src/entity/entities/objects/items"]),
        ("Биомы, измерения, пресеты", ["steel-registry/src/biome.rs", "steel-registry/src/dimension_type.rs", "steel-registry/src/world_preset.rs", "steel-registry/build/biomes.rs", "steel-registry/build/dimension_types.rs", "steel-registry/build/dimensions.rs", "steel-registry/build/world_presets.rs"]),
    ]:
        row(name, paths)
    print("\n## Пофайловый индекс\n\nУказаны имена тестов из исходника, **не статус их прохождения**.\n\n| Файл | LOC | Тесты | Маркеров |\n|---|---:|---|---:|")
    for member in members:
        fs, _, _, _ = stats([f"{member}/src", f"{member}/build"])
        for p in fs:
            text = p.read_text()
            names = TEST.findall(text)
            print(f"| `{p.relative_to(ROOT)}` | {len(text.splitlines())} | " + (", ".join(f"`{n}`" for n in names) or "—") + f" | {len(MARKER.findall(text))} |")


if __name__ == "__main__":
    main()
