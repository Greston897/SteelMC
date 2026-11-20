#!/usr/bin/env python3
"""Source wiring inventory, NOT a runtime or behavioral-parity test.

Generated classes are compared with behavior annotations. Goal declarations
are checked for construction/reference sites outside their own file and test
modules. References are evidence to inspect, never an automatic PASS.
"""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / 'steel-core/src'


def location(path, text, offset):
    return f'{path.relative_to(ROOT)}:{text.count(chr(10), 0, offset) + 1}'


def inventory():
    sources = {p: p.read_text() for p in SRC.rglob('*.rs')}
    production = {p: text.split('#[cfg(test)]', 1)[0] for p, text in sources.items()}
    classes = json.loads((ROOT / 'steel-core/build/classes.json').read_text())
    objects = []
    for category, attribute in [('entities', 'entity_behavior'), ('blocks', 'block_behavior'), ('items', 'item_behavior')]:
        discovered = {}
        pattern = re.compile(r'#\[' + attribute + r'\s*\(\s*class\s*=\s*"([^"]+)"')
        for path, text in production.items():
            for match in pattern.finditer(text):
                discovered.setdefault(match[1], []).append(location(path, text, match.start()))
        for entry in classes.get(category, []):
            sites = discovered.get(entry['class'], [])
            objects.append({'category': category, 'name': entry['name'], 'class': entry['class'],
                            'implementation_sites': sites,
                            'status': 'ANNOTATION_FOUND_REQUIRES_CALL_PATH_AUDIT' if sites else 'NO_EXACT_ANNOTATION_CHECK_GENERATOR_INHERITANCE_OR_FALLBACK'})
    goals = []
    goal_root = SRC / 'entity/ai/goal'
    for path in sorted(goal_root.glob('*.rs')):
        if path.name == 'selector.rs':
            continue
        text = production[path]
        for match in re.finditer(r'\bstruct\s+(\w*Goal)\b', text):
            name = match[1]
            refs = []
            for other, body in production.items():
                if other in (path, goal_root / 'mod.rs'):
                    continue
                for reference in re.finditer(r'\b' + re.escape(name) + r'\b', body):
                    refs.append(location(other, body, reference.start()))
            goals.append({'name': name, 'implementation': location(path, text, match.start()),
                          'production_references': refs,
                          'status': 'SOURCE_REFERENCES_ONLY_NOT_RUNTIME_PASS' if refs else 'NO_PRODUCTION_CONSUMER_FOUND'})
    return {'scope': 'generated object class inputs and declared shared AI goals; NOT every vanilla mechanic',
            'objects': objects, 'goals': goals}


def main():
    data = inventory()
    (ROOT / '.ci/wiring-inventory.json').write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    lines = ['# Инвентаризация подключения исходников', '',
             '**Не доказательство поведенческого паритета.** Список охватывает входы генератора объектов и общие AI goals, а не каждую механику vanilla. '
             'Аннотация, импорт и ссылка не означают, что поведение исполняется правильно. Отсутствие точной аннотации может означать наследование через генератор; это не автоматически отсутствующая механика.', '',
             'Воспроизведение: `python3 .ci/wiring_inventory.py`.', '', '## Общие AI goals', '',
             '| Goal | Реализация | Производственные ссылки (без собственного файла и тестов) |', '|---|---|---|']
    for goal in data['goals']:
        refs = ', '.join(f'`{r}`' for r in goal['production_references']) or '**NO_PRODUCTION_CONSUMER_FOUND**'
        lines.append(f"| {goal['name']} | `{goal['implementation']}` | {refs} |")
    for category in ['entities', 'blocks', 'items']:
        lines += ['', f'## {category}: входы генератора', '', '| Объект | Vanilla class | Точная аннотация реализации |', '|---|---|---|']
        for obj in data['objects']:
            if obj['category'] != category:
                continue
            sites = ', '.join(f'`{site}`' for site in obj['implementation_sites']) or 'Нет точной аннотации: проверить наследование/генератор/fallback'
            lines.append(f"| {obj['name']} | {obj['class']} | {sites} |")
    (ROOT / 'WIRING-INVENTORY-26.3-RU.md').write_text('\n'.join(lines) + '\n')
    print(f"{len(data['objects'])} generator inputs; {len(data['goals'])} shared goals")
    print('Goals without external production references: ' + ', '.join(g['name'] for g in data['goals'] if not g['production_references']))


if __name__ == '__main__':
    main()
