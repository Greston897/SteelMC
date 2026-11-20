"""Report source inventory next to observed test outcomes, without parity scores."""
import argparse
import json
from pathlib import Path
from progress_inventory import stats

# Source/test groups overlap only where explicitly stated in the resulting file.
AREAS = [
    ('Инфраструктура', ['steel-core/src/bootstrap.rs', 'steel-core/src/server', 'steel/src'], ['steel_core/server', 'steel/']),
    ('Мир и тики', ['steel-core/src/world'], ['steel_core/world']),
    ('Чанки и свет', ['steel-core/src/chunk'], ['steel_core/chunk']),
    ('Хранение и миграции', ['steel-core/src/chunk_saver', 'steel-core/src/level_data'], ['steel_core/chunk_saver', 'steel_core/level_data']),
    ('Worldgen', ['steel-core/src/worldgen', 'steel-worldgen/src'], ['steel_core/worldgen', 'steel_worldgen/']),
    ('Блоки', ['steel-core/src/behavior/blocks', 'steel-core/src/behavior/block'], ['steel_core/behavior::blocks', 'steel_core/behavior::block::']),
    ('Предметы', ['steel-core/src/behavior/items'], ['steel_core/behavior::items']),
    ('Block entities', ['steel-core/src/block_entity'], ['steel_core/block_entity']),
    ('Сущности и AI', ['steel-core/src/entity'], ['steel_core/entity']),
    ('Игрок и соединение', ['steel-core/src/player'], ['steel_core/player']),
    ('Инвентарь и меню', ['steel-core/src/inventory'], ['steel_core/inventory']),
    ('Команды и права', ['steel-core/src/command', 'steel-core/src/permission'], ['steel_core/command', 'steel_core/permission']),
    ('Физика, жидкости, порталы', ['steel-core/src/physics', 'steel-core/src/fluid', 'steel-core/src/portal'], ['steel_core/physics', 'steel_core/fluid', 'steel_core/portal']),
    ('Loot и зачарования', ['steel-core/src/loot', 'steel-core/src/enchantment_helper.rs', 'steel-core/src/enchantment_helper'], ['steel_core/loot', 'steel_core/enchantment_helper']),
    ('Протокол', ['steel-protocol/src'], ['steel_protocol/']),
    ('Реестры и кодеки', ['steel-registry/src', 'steel-registry/build'], ['steel_registry/']),
    ('Вход и сессии', ['steel-login/src'], ['steel_login/']),
    ('Утилиты и математика', ['steel-utils/src', 'steel-math/src', 'steel-crypto/src'], ['steel_utils/', 'steel_math/', 'steel_crypto/']),
    ('Flint adapter', ['steel-flint/src'], ['steel_flint/']),
]


def count_tests(tests, prefixes):
    counts = dict.fromkeys(('passed', 'failed', 'ignored'), 0)
    for test in tests:
        key = test['crate'] + '/' + test['name']
        if any((key.startswith(prefix) if prefix.endswith(('/', '::')) else key.startswith(prefix + '::')) for prefix in prefixes):
            counts[test['status']] += 1
    return counts


def build_report(report, sha, run_url):
    rows = []
    for name, paths, prefixes in AREAS:
        files, loc, tests, _ = stats(paths)
        rows.append({'area': name, 'files': len(files), 'loc': loc, 'static_test_functions': tests,
                     **count_tests(report['tests'], prefixes)})
    matched = [prefix for _, _, prefixes in AREAS for prefix in prefixes]
    other = [test for test in report['tests']
             if not any(((test['crate'] + '/' + test['name']).startswith(prefix) if prefix.endswith(('/', '::')) else (test['crate'] + '/' + test['name']).startswith(prefix + '::')) for prefix in matched)]
    lines = ['# Подсистемы: код и фактически исполненные тесты', '', f'Код: `{sha}`. Run: {run_url}', '',
             'Это наблюдаемые результаты одного полного cargo test. Passed — число тестовых случаев, не число механик. Static — функции в исходниках; ignored не исполнены. Процент реализации и процент покрытия механик **не установлены**: полного согласованного списка vanilla-механик и ветвей нет.', '',
             'Server-строка включает инфраструктурные пути, взаимодействующие с миром; строки не следует складывать как независимые механики. Интеграционные и doctest-сценарии, не сопоставленные по префиксу, приведены отдельным числом, не теряются.', '',
             '| Подсистема | Rust-файлы | LOC | Static tests | Passed | Failed | Ignored |',
             '|---|---:|---:|---:|---:|---:|---:|']
    for row in rows:
        lines.append('| ' + ' | '.join(str(row[key]) for key in ('area', 'files', 'loc', 'static_test_functions', 'passed', 'failed', 'ignored')) + ' |')
    lines += ['', f"Все наблюдаемые результаты (включая прочие): `{json.dumps(report['totals'], ensure_ascii=False)}`.",
              f'Вне префиксов таблицы: {len(other)} результатов. Полные имена — automated-mechanics-tests.json.', '',
              'Наличие исходников не равно завершённой реализации. Известные пробелы и ручные сценарии: READINESS-26.3-FACTS.md и MANUAL-MECHANICS-26.3-RU.md.', '']
    return '\n'.join(lines), rows


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sha', required=True)
    parser.add_argument('--run-url', required=True)
    args = parser.parse_args()
    report = json.loads(args.input.read_text())
    markdown, rows = build_report(report, args.sha, args.run_url)
    args.output.write_text(markdown)
    print('::notice title=Subsystem observed counts::' + json.dumps(rows, ensure_ascii=False))
