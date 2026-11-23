#!/usr/bin/env python3
"""Count explicitly inventoried 26.3 scenarios, not overall Minecraft parity.

Example: python3 .ci/vanilla_readiness.py --markdown report.md --json report.json
Evidence without the exact same clean Steel revision is historical, never current.
"""
import argparse
import collections
import csv
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / '.ci/vanilla-readiness'
FULL_SHA = re.compile(r'^[0-9a-f]{40}$')
RUN_URL = re.compile(r'^https://github\.com/Greston897/SteelMC/actions/runs/[1-9][0-9]*$')
ID = re.compile(r'^[a-zA-Z0-9_]+$')


def load_json(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def validate_inventory(ids):
    if not isinstance(ids, list) or not ids:
        raise ValueError('scenario inventory must be a nonempty list')
    if any(not isinstance(i, str) or not ID.fullmatch(i) for i in ids):
        raise ValueError('invalid scenario id')
    if len(ids) != len(set(ids)):
        raise ValueError('duplicate scenario id')
    return ids


def manual_inventory(path):
    with Path(path).open(encoding='utf-8', newline='') as source:
        reader = csv.DictReader(source, delimiter='\t')
        if reader.fieldnames != ['# id', 'area', 'mechanic', 'setup', 'action', 'expected']:
            raise ValueError('unexpected manual scenario columns')
        rows = list(reader)
    validate_inventory([row['# id'] for row in rows])
    if any(None in row or not all(isinstance(value, str) and value.strip() for value in row.values()) for row in rows):
        raise ValueError('missing manual scenario detail')
    return rows


def scenario_results(inventory, evidence, inventory_sha256):
    """Compare identical named assertions; missing cases remain unknown.

    This compares assertion outcomes, not all observations or the mechanics
    behind each case. It does not assume a successful CI job covers missing IDs.
    """
    validate_inventory(inventory)
    if evidence.get('schema') != 1:
        raise ValueError('unsupported evidence schema')
    if not isinstance(inventory_sha256, str) or not re.fullmatch(r'[0-9a-f]{64}', inventory_sha256) or evidence.get('inventory_sha256') != inventory_sha256:
        raise ValueError('evidence does not match the exact inventory bytes')
    if not isinstance(evidence.get('steel_commit'), str) or not FULL_SHA.fullmatch(evidence['steel_commit']):
        raise ValueError('evidence needs a full 40-character Steel commit')
    if not isinstance(evidence.get('run_url'), str) or not RUN_URL.fullmatch(evidence['run_url']):
        raise ValueError('evidence needs a canonical repository Actions run URL')
    if evidence.get('run_conclusion') not in ('success', 'failure', 'cancelled', 'in_progress'):
        raise ValueError('invalid run conclusion')
    reports = evidence.get('reports')
    if not isinstance(reports, dict) or set(reports) != {'steel', 'vanilla-26.3'}:
        raise ValueError('both uniquely named engine reports are required')
    parsed = {}
    for engine, report in reports.items():
        if not isinstance(report, dict) or report.get('engine') != engine or not isinstance(report.get('results'), list):
            raise ValueError(f'invalid {engine} report')
        found = {}
        for row in report['results']:
            if not isinstance(row, dict):
                raise ValueError(f'invalid {engine} scenario result')
            name = row.get('name')
            if not isinstance(name, str) or name not in inventory:
                raise ValueError(f'unregistered scenario: {name!r}')
            if name in found:
                raise ValueError(f'duplicate result: {name}')
            if type(row.get('passed')) is not bool:
                raise ValueError(f'non-boolean result: {name}')
            found[name] = row['passed']
        parsed[engine] = found
    rows = []
    for name in inventory:
        v = parsed['vanilla-26.3'].get(name)
        s = parsed['steel'].get(name)
        if v is None or s is None:
            outcome = 'unknown'
        elif not v:
            outcome = 'invalid_reference'
        elif s:
            outcome = 'matched_assertions'
        else:
            outcome = 'steel_failed'
        rows.append({'id': name, 'status': outcome, 'vanilla_passed': v, 'steel_passed': s})
    return rows


def counts(rows):
    found = collections.Counter(row['status'] for row in rows)
    return {key: found[key] for key in ('matched_assertions', 'steel_failed', 'invalid_reference', 'unknown')}


def state(root=ROOT):
    try:
        sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
        dirty = bool(subprocess.check_output(
            ['git', 'status', '--porcelain', '--untracked-files=all'], cwd=root, text=True))
        if not FULL_SHA.fullmatch(sha):
            raise ValueError('invalid git commit')
        return {'head': sha, 'dirty': dirty}
    except (OSError, subprocess.CalledProcessError, ValueError):
        return {'head': None, 'dirty': None}


def verify_ci(evidence):
    """Check run metadata, not authenticity of a separately supplied results file."""
    if not RUN_URL.fullmatch(evidence.get('run_url', '')):
        raise ValueError('invalid run URL')
    run = evidence['run_url'].rsplit('/', 1)[1]
    try:
        raw = subprocess.check_output(
            ['gh', 'run', 'view', run, '--repo', 'Greston897/SteelMC', '--json',
             'status,conclusion,headSha,jobs'], text=True, stderr=subprocess.PIPE)
        data = json.loads(raw)
    except (OSError, subprocess.CalledProcessError, json.JSONDecodeError):
        return 'unavailable'
    if (data.get('status') != 'completed' or data.get('conclusion') != evidence['run_conclusion']
            or data.get('headSha') != evidence['steel_commit']):
        return 'mismatch_or_incomplete'
    return 'metadata_verified'


def evidence_is_current(evidence, revision, ci_verification):
    # Run metadata validation does not authenticate arbitrary input files.
    return (ci_verification == 'metadata_verified' and revision.get('dirty') is False
            and revision.get('head') == evidence['steel_commit']
            and evidence['run_conclusion'] == 'success')


def trigger_inventory(path, rust_path, vanilla_source=None):
    data = load_json(path)
    if data.get('schema') != 1 or data.get('version') != '26.3':
        raise ValueError('unsupported trigger inventory')
    registrations = data.get('registrations')
    if not isinstance(registrations, list) or not registrations:
        raise ValueError('missing vanilla trigger registrations')
    constants = [row['constant'] for row in registrations]
    names = [row['id'] for row in registrations]
    validate_inventory(names)
    if len(constants) != len(set(constants)) or any(not re.fullmatch(r'[A-Z_]+', c) for c in constants):
        raise ValueError('invalid trigger constants')
    java = '\n'.join(f'public static final Trigger {c} = register("{n}", x);'
                     for c, n in zip(constants, names))
    if vanilla_source is not None:
        raw = Path(vanilla_source).read_bytes()
        if hashlib.sha256(raw).hexdigest() != data.get('source_sha256'):
            raise ValueError('vanilla source differs from pinned inventory')
        java = raw.decode('utf-8')
    spec = importlib.util.spec_from_file_location('trigger_audit', ROOT / '.ci/advancement_trigger_audit.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    rows = module.inventory(java, Path(rust_path).read_text(encoding='utf-8'))
    if [(c, n) for c, n, _ in rows] != list(zip(constants, names)):
        raise ValueError('vanilla source registrations differ from pinned inventory')
    return {'total': len(rows), 'gate_enabled': [n for _, n, yes in rows if yes],
            'gate_not_enabled': [n for _, n, yes in rows if not yes],
            'vanilla_source_hash_verified': vanilla_source is not None,
            'vanilla_source_sha256': data['source_sha256'], 'source_run': data['source_generation_run'],
            'steel_source_sha256': hashlib.sha256(Path(rust_path).read_bytes()).hexdigest()}


def datapack_results(evidence, revision, ci_verification):
    if evidence.get('schema') != 1 or not FULL_SHA.fullmatch(evidence.get('steel_commit', '')):
        raise ValueError('invalid datapack evidence provenance')
    if not RUN_URL.fullmatch(evidence.get('run_url', '')):
        raise ValueError('invalid datapack evidence run')
    if evidence.get('run_conclusion') not in ('success', 'failure', 'cancelled'):
        raise ValueError('invalid datapack run conclusion')
    for key, filename in [('probe_sha256', 'datapack_live_probe.py'),
                          ('comparator_sha256', 'compare_datapack_probe.py')]:
        digest = hashlib.sha256((ROOT / '.ci' / filename).read_bytes()).hexdigest()
        if evidence.get(key) != digest:
            raise ValueError(f'datapack evidence does not match {filename}')
    spec = importlib.util.spec_from_file_location('datapack_compare', ROOT / '.ci/compare_datapack_probe.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    paired = evidence['reports']
    comparison = module.compare(paired['steel'], paired['vanilla-26.3'])
    cases = comparison['results']
    counter = collections.Counter(case['result'] for case in cases)
    return {'scope': comparison['scope'], 'total': len(cases),
            'counts': {key: counter[key] for key in ('matched_assertions', 'steel_failed',
                                                      'invalid_reference', 'blocked_reference', 'blocked_steel')},
            'cases': cases, 'evidence_commit': evidence['steel_commit'],
            'run_url': evidence['run_url'], 'run_conclusion': evidence['run_conclusion'],
            'ci_verification': ci_verification,
            'same_clean_revision': ci_verification == 'metadata_verified'
                and revision.get('dirty') is False and revision.get('head') == evidence['steel_commit']}


def percent(n, d):
    return round(100 * n / d, 2) if d else None


def build_report(inventory, smoke, manual, revision, triggers, smoke_digest, manual_digest,
                 manual_evidence=None, smoke_ci='not_checked', manual_ci='not_checked'):
    ids = validate_inventory(inventory['cases'])
    smoke_rows = scenario_results(ids, smoke, smoke_digest)
    manual_ids = [row['# id'] for row in manual]
    validate_inventory(manual_ids)
    if manual_evidence is None:
        manual_rows = [{'id': i, 'status': 'unknown', 'vanilla_passed': None, 'steel_passed': None}
                       for i in manual_ids]
    else:
        manual_rows = scenario_results(manual_ids, manual_evidence, manual_digest)
    manual_metadata = {row['# id']: row for row in manual}
    for row in manual_rows:
        metadata = manual_metadata[row['id']]
        row['area'] = metadata['area']
        row['mechanic'] = metadata['mechanic']
    by_area = {}
    for area in sorted({row['area'] for row in manual_rows}):
        area_rows = [row for row in manual_rows if row['area'] == area]
        by_area[area] = {'total': len(area_rows), 'counts': counts(area_rows)}
    sc = counts(smoke_rows)
    mc = counts(manual_rows)
    return {
        'schema': 1,
        'target': 'Minecraft Java 26.3 server',
        'global_readiness_percent': None,
        'global_reason': 'No complete, non-overlapping inventory of vanilla server behavior exists here.',
        'revision': revision,
        'smoke': {'scope': inventory['scope'], 'source_commit': inventory['source_commit'],
                  'evidence_commit': smoke['steel_commit'], 'run_url': smoke['run_url'],
                  'run_conclusion': smoke['run_conclusion'], 'ci_verification': smoke_ci,
                  'current': evidence_is_current(smoke, revision, smoke_ci),
                  'total': len(ids), 'counts': sc,
                  'matched_percent_of_inventory': percent(sc['matched_assertions'], len(ids)),
                  'cases': smoke_rows},
        'manual': {'scope': 'Ручные сценарии, не полный и не независимый перечень всех vanilla-механик',
                   'total': len(manual_ids), 'counts': mc, 'by_area': by_area,
                   'matched_percent_of_inventory': percent(mc['matched_assertions'], len(manual_ids)),
                   'evidence_commit': manual_evidence['steel_commit'] if manual_evidence else None,
                   'run_url': manual_evidence['run_url'] if manual_evidence else None,
                   'ci_verification': manual_ci,
                   'current': evidence_is_current(manual_evidence, revision, manual_ci) if manual_evidence else False,
                   'cases': manual_rows},
        'advancement_source_inventory': triggers,
    }


def render(report):
    r = report['revision']
    lines = ['# Измерение проверенных сценариев Minecraft 26.3', '',
             '**Общий процент готовности всего vanilla-ядра: неизвестен.** '
             'Полного независимого перечня поведения и исчерпывающих проверок нет.', '',
             f"Рабочий HEAD: `{r['head'] or 'не определён'}`; незакоммиченные изменения: `{r['dirty']}`. "
             'Изменённый checkout не равен проверенному CI коммиту.', '',
             '## Парные проверки серверов', '']
    for key, label in [('smoke', 'Автоматический live smoke'), ('manual', 'Ручной перечень')]:
        item = report[key]
        c = item['counts']
        status = 'текущий чистый коммит' if item['current'] else 'не подтверждено для текущего рабочего состояния'
        lines += [f"### {label}", '', item['scope'] + '.',
                  f"Всего: **{item['total']}**; совпали проверки: **{c['matched_assertions']}**; "
                  f"Steel не прошёл: **{c['steel_failed']}**; эталон не прошёл: **{c['invalid_reference']}**; "
                  f"неизвестно: **{c['unknown']}**.",
                  f"Доля совпавших проверок в этом перечне: **{item['matched_percent_of_inventory']}%**; "
                  f"{status}."]
        if item['run_url']:
            lines.append(f"Доказательство: [{item['evidence_commit']}]({item['run_url']}); "
                         f"проверка метаданных CI: `{item['ci_verification']}`. "
                         'Метаданные CI не подтверждают подлинность произвольно предоставленного файла результатов.')
        if key == 'manual':
            lines += ['', '| Подсистема | Совпало / всего | Неизвестно |', '|---|---:|---:|']
            for area, stats in item['by_area'].items():
                lines.append(f"| {area} | {stats['counts']['matched_assertions']} / {stats['total']} | "
                             f"{stats['counts']['unknown']} |")
            lines += ['', '| ID | Подсистема / сценарий | Результат |', '|---|---|---|']
            lines.extend(f"| `{row['id']}` | {row['area']}: {row['mechanic']} | {row['status']} |"
                         for row in item['cases'])
        else:
            lines += ['', '| Сценарий | Результат |', '|---|---|']
            lines.extend(f"| `{row['id']}` | {row['status']} |" for row in item['cases'])
        lines.append('')
    if 'datapack_live' in report:
        d = report['datapack_live']
        c = d['counts']
        lines += ['## Парный datapack-прогон', '',
                  f"Сценариев: **{d['total']}**; совпали условия: **{c['matched_assertions']}**; "
                  f"не прошли на Steel: **{c['steel_failed']}**; заблокированы на Steel: **{c['blocked_steel']}**; "
                  f"невалидный/заблокированный эталон: **{c['invalid_reference'] + c['blocked_reference']}**.",
                  f"[{d['evidence_commit']}]({d['run_url']}): общий результат CI `{d['run_conclusion']}`, "
                  f"метаданные CI `{d['ci_verification']}`, чистый checkout этого SHA: `{d['same_clean_revision']}`.",
                  'Каталожные функции через RCON, без подключённых клиентов; не полный datapack.', '',
                  '| Сценарий | Результат |', '|---|---|']
        lines.extend(f"| `{case['name']}` | {case['result']} |" for case in d['cases'])
        lines.append('')
    t = report['advancement_source_inventory']
    lines += ['## Инвентаризация advancement triggers', '',
              f"Vanilla зарегистрировала **{t['total']}** типов; "
              f"в проверенном статическом gate Steel присутствуют **{len(t['gate_enabled'])}**, "
              f"остальные **{len(t['gate_not_enabled'])}** в этом gate не разрешены.",
              'Это инвентаризация исходников, не проверка работы, предикатов или процента паритета; '
              '`impossible` специально не является обычным событием. Другая проводка не исключена.',
              'Разрешённые gate: ' + ', '.join(f'`{name}`' for name in t['gate_enabled']) + '.',
              f"SHA-256 vanilla-источника: `{t['vanilla_source_sha256']}`; "
              f"проверен на этом запуске: `{t['vanilla_source_hash_verified']}`. "
              'Без файла используется сохранённая инвентаризация, а не повторное чтение vanilla.', '',
              '## Ограничения', '',
              '- Совпавшие assertions не означают совпадения всех выходов или всех веток механики.',
              '- Эти перечни неполны и пересекаются: их нельзя суммировать или выводить из них общий процент.',
              '- Неизвестное не означает ни работающую, ни сломанную функциональность.',
              '- Нет тестирования официальных игровых клиентов; live smoke использует RCON без игроков.',
              '- JSON-отчёт содержит результаты по каждому ID; подтверждения привязаны к SHA и запуску.', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--smoke-evidence', type=Path, default=DATA / 'smoke-812c597.json')
    parser.add_argument('--manual-evidence', type=Path)
    parser.add_argument('--datapack-evidence', type=Path, default=DATA / 'datapack-live-ba05896.json')
    parser.add_argument('--vanilla-source', type=Path, help='Pinned CriteriaTriggers.java to verify the saved inventory')
    parser.add_argument('--verify-ci', action='store_true', help='Query GitHub to check run SHA and conclusion')
    parser.add_argument('--markdown', type=Path, help='Write human-readable report')
    parser.add_argument('--json', type=Path, help='Write machine-readable report')
    args = parser.parse_args()
    manifest = (ROOT / 'Cargo.toml').read_text(encoding='utf-8')
    package = manifest.split('[workspace.package]', 1)[1].split('\n[', 1)[0]
    version = tomllib.loads('[package]\n' + package)['package']['version']
    if not version.endswith('+mc26.3'):
        raise ValueError(f'workspace target mismatch: {version}')
    inventory_path = DATA / 'smoke-inventory.json'
    manual_path = ROOT / '.ci/manual_mechanics_cases.tsv'
    inventory = load_json(inventory_path)
    smoke = load_json(args.smoke_evidence)
    manual = manual_inventory(manual_path)
    triggers = trigger_inventory(DATA / 'trigger-inventory-26.3.json',
                                 ROOT / 'steel-core/src/player/advancement/mod.rs', args.vanilla_source)
    manual_evidence = load_json(args.manual_evidence) if args.manual_evidence else None
    smoke_ci = verify_ci(smoke) if args.verify_ci else 'not_checked'
    manual_ci = verify_ci(manual_evidence) if args.verify_ci and manual_evidence else 'not_checked'
    revision = state()
    report = build_report(inventory, smoke, manual, revision, triggers,
                          hashlib.sha256(inventory_path.read_bytes()).hexdigest(),
                          hashlib.sha256(manual_path.read_bytes()).hexdigest(),
                          manual_evidence, smoke_ci, manual_ci)
    datapack = load_json(args.datapack_evidence)
    datapack_ci = verify_ci(datapack) if args.verify_ci else 'not_checked'
    report['datapack_live'] = datapack_results(datapack, revision, datapack_ci)
    text = render(report)
    if args.markdown:
        args.markdown.write_text(text, encoding='utf-8')
    else:
        print(text)
    if args.json:
        args.json.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, json.JSONDecodeError) as error:
        print(f'readiness calculation refused: {error}', file=sys.stderr)
        sys.exit(2)
