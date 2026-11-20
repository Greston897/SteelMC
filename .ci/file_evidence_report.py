#!/usr/bin/env python3
"""Join actual headless test outcomes to exact source declarations and hash every Rust file.

A test executing in a file does NOT prove all its production code or dependencies.
No production source gains PASS merely because it was referenced or compiled.
"""
import argparse
import base64
from collections import Counter, defaultdict
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import urllib.request
import zlib

from progress_inventory import TEST
from subsystem_report import AREAS, count_tests

ROOT = Path(__file__).resolve().parents[1]


def masked_code(text):
    return re.sub(r'r(\#*)"[\s\S]*?"\1|"(?:\\.|[^"\\])*"|//[^\n]*|/\*[\s\S]*?\*/',
                  lambda m: ''.join('\n' if c == '\n' else ' ' for c in m[0]), text)


def tracked_sources(root):
    names = subprocess.check_output(['git', 'ls-files', '-z', '*.rs'], cwd=root).decode().split('\0')
    return [name for name in names if name and (root / name).is_file()]


def source_index(root, paths):
    index = defaultdict(list)
    records = {}
    for path in paths:
        text = (root / path).read_text()
        crate = path.split('/')[0].replace('-', '_')
        declarations = []
        for match in TEST.finditer(masked_code(text)):
            declaration = {'path': path, 'line': text.count('\n', 0, match.start()) + 1, 'name': match[1]}
            declarations.append(declaration)
            # Integration targets have their own binary name, not steel_core.
            parts = Path(path).parts
            if len(parts) >= 3 and parts[1] == 'tests':
                target = Path(parts[2]).stem if parts[2].endswith('.rs') else parts[2]
            else:
                target = crate
            index[(target, match[1])].append(declaration)
        records[path] = {'path': path, 'sha256': hashlib.sha256((root / path).read_bytes()).hexdigest(),
                         'loc': len(text.splitlines()), 'declared_tests': declarations,
                         'observed_tests': []}
    return index, records


def build(report, root, sha, run_url, paths=None):
    index, records = source_index(root, paths if paths is not None else tracked_sources(root))
    unmatched = []
    for number, test in enumerate(report['tests']):
        if test['status'] not in ('passed', 'failed', 'ignored'):
            raise ValueError(f'Unknown outcome: {test}')
        crate = test['crate']
        if crate.endswith(':doctest'):
            path = test['name'].split(' - ', 1)[0]
            candidates = [{'path': path, 'line': None}] if path in records else []
        else:
            candidates = index.get((crate, test['name'].rsplit('::', 1)[-1]), [])
        if len(candidates) != 1:
            unmatched.append({'test_index': number, **test, 'candidate_files': sorted({c['path'] for c in candidates}),
                              'reason': 'ambiguous' if candidates else 'no_exact_source_declaration'})
            continue
        candidate = candidates[0]
        records[candidate['path']]['observed_tests'].append({'test_index': number, **test, 'line': candidate['line']})
    for record in records.values():
        counts = Counter(t['status'] for t in record['observed_tests'])
        record['counts'] = {key: counts[key] for key in ('passed', 'failed', 'ignored')}
        if counts['failed']:
            record['evidence'] = 'OWN_TEST_FAILED'
        elif counts['passed']:
            record['evidence'] = 'OWN_TEST_EXECUTED_NOT_WHOLE_FILE_VERIFIED'
        elif counts['ignored']:
            record['evidence'] = 'OWN_TEST_IGNORED_NOT_EXECUTED'
        elif record['declared_tests']:
            record['evidence'] = 'DECLARED_TEST_NOT_MATCHED_TO_RUN'
        else:
            record['evidence'] = 'NO_OWN_EXECUTED_TEST_EVIDENCE'
    rows = []
    for area, area_paths, prefixes in AREAS:
        files = [record for path, record in records.items() if any(path == p or path.startswith(p.rstrip('/') + '/') for p in area_paths)]
        outcomes = count_tests(report['tests'], prefixes)
        required = sum(outcomes.values())
        executed = outcomes['passed'] + outcomes['failed']
        rows.append({'area': area, 'files': len(files),
                     'files_with_own_executed_tests': sum(r['counts']['passed'] + r['counts']['failed'] > 0 for r in files),
                     **outcomes, 'observed_test_denominator': required,
                     'successful_case_percent': round(100 * outcomes['passed'] / required, 2) if required else None,
                     'executed_case_percent': round(100 * executed / required, 2) if required else None})
    totals = Counter(t['status'] for t in report['tests'])
    return {'schema': 1, 'sha': sha, 'run_url': run_url,
            'meaning': 'Observed headless tests and exact source attribution, NOT implementation or behavioral coverage percentages',
            'real_client_cases_included': 0, 'real_client_cases_excluded_from_this_run': 0,
            'ignored_policy': 'All ignored Rust tests here are headless; none excluded as player-only',
            'totals': {key: totals[key] for key in ('passed', 'failed', 'ignored')},
            'rows': rows, 'files': list(records.values()), 'unmatched_observations': unmatched,
            'tests': report['tests']}



def attach_coverage(data, llvm_report, root):
    records = {record['path']: record for record in data['files']}
    matched = set()
    for export in llvm_report.get('data', []):
        for source in export.get('files', []):
            path = Path(source['filename'])
            try:
                name = str(path.relative_to(root)) if path.is_absolute() else str(path)
            except ValueError:
                continue
            if name not in records:
                continue
            if name in matched:
                raise ValueError(f'Duplicate LLVM file summary: {name}')
            matched.add(name)
            summary = source['summary']
            lines = summary['lines']
            if not 0 <= lines['covered'] <= lines['count']:
                raise ValueError(f'Invalid line counters: {name}')
            records[name]['instrumented_lines'] = {'covered': lines['covered'], 'count': lines['count'],
                'percent': round(100 * lines['covered'] / lines['count'], 2) if lines['count'] else None}
    if not matched:
        raise ValueError('No tracked source matched LLVM coverage; refusing an empty coverage report')
    data['coverage_scope'] = 'Instrumented lines executed by headless workspace tests; not behavioral parity'
    for record in records.values():
        record.setdefault('instrumented_lines', None)
    for row, (_, area_paths, _) in zip(data['rows'], AREAS):
        files = [record for path, record in records.items() if any(path == p or path.startswith(p.rstrip('/') + '/') for p in area_paths)]
        measured = [record['instrumented_lines'] for record in files if record['instrumented_lines'] is not None]
        count = sum(item['count'] for item in measured)
        covered = sum(item['covered'] for item in measured)
        row['instrumented_lines'] = {'covered': covered, 'count': count,
            'percent': round(100 * covered / count, 2) if count else None,
            'files_measured': len(measured), 'files_not_in_llvm_export': len(files) - len(measured)}
    return data


def render(data):
    lines = ['# Подсистемы и каждый Rust-файл: фактические автоматические проверки', '',
             f"Код: `{data['sha']}`. Запуск: {data['run_url']}", '',
             'Процент ниже = passed / (passed + failed + ignored) конкретного наблюдаемого набора тестов. '
             'Это не процент реализации Minecraft. Реальные клиенты не участвуют; автоматически создаваемые Player-сущности включены. '
             'Ignored остаются в знаменателе: им не нужны игроки. '
             'Строки «внутри» входят в родительские и не складываются.', '',
             '| Подсистема | Rust-файлы | Файлы с собственным исполненным тестом | PASS | FAIL | Не исполнено | Успех набора |',
             '|---|---:|---:|---:|---:|---:|---:|']
    for row in data['rows']:
        percent = '—' if row['successful_case_percent'] is None else f"{row['successful_case_percent']:.2f}%"
        lines.append(f"| {row['area']} | {row['files']} | {row['files_with_own_executed_tests']} | {row['passed']} | {row['failed']} | {row['ignored']} | {percent} |")
    if 'coverage_scope' in data:
        lines += ['', '## Измеренное исполнение инструментированных строк', '',
                  'Покрытые строки / все инструментированные строки. Файл, отсутствующий в LLVM export, не получает вымышленный процент. '
                  'Это измерение исполнения кода, а не доказательство правильности всех состояний.', '',
                  '| Подсистема | Исполнено / инструментировано | % строк | Файлов в LLVM / вне LLVM |', '|---|---:|---:|---:|']
        for row in data['rows']:
            coverage = row['instrumented_lines']
            percent = '—' if coverage['percent'] is None else f"{coverage['percent']:.2f}%"
            lines.append(f"| {row['area']} | {coverage['covered']} / {coverage['count']} | {percent} | {coverage['files_measured']} / {coverage['files_not_in_llvm_export']} |")
    lines += ['', f"Все результаты: `{json.dumps(data['totals'])}`. Точная привязка к файлу не установлена для {len(data['unmatched_observations'])} наблюдений; они сохранены в JSON, не потеряны и не приписаны вымышленным файлам.", '',
              '## Каждый отслеживаемый Rust-файл', '',
              'Собственный исполненный тест подтверждает только этот тест, не весь файл. '
              'NO_OWN_EXECUTED_TEST_EVIDENCE не означает, что файл не используется: интеграционные тесты могут проверять его косвенно; '
              'без инструментального покрытия это не засчитывается автоматически.', '',
              '| Файл | SHA256 | PASS / FAIL / ignored | Наблюдаемое основание |', '|---|---|---|---|']
    for record in data['files']:
        counts = record['counts']
        coverage = record.get('instrumented_lines')
        detail = '' if 'coverage_scope' not in data else (f"; LLVM lines {coverage['covered']}/{coverage['count']}" if coverage is not None else '; NOT_IN_LLVM_EXPORT')
        lines.append(f"| `{record['path']}` | `{record['sha256']}` | {counts['passed']} / {counts['failed']} / {counts['ignored']} | {record['evidence']}{detail} |")
    return '\n'.join(lines) + '\n'


def publish(data):
    """REST-readable complete evidence, avoiding truncated annotation/log tails."""
    token = os.environ['GITHUB_TOKEN']
    repository = os.environ['GITHUB_REPOSITORY']
    raw = json.dumps(data, ensure_ascii=False, separators=(',', ':')).encode()
    digest = hashlib.sha256(raw).hexdigest()
    encoded = base64.b64encode(zlib.compress(raw, 9)).decode()
    chunks = [encoded[offset:offset + 44000] for offset in range(0, len(encoded), 44000)]
    for index, chunk in enumerate(chunks, 1):
        body = f"FILE_EVIDENCE_V1 {digest} {index}/{len(chunks)}\n```\n{chunk}\n```"
        request = urllib.request.Request(f"https://api.github.com/repos/{repository}/commits/{data['sha']}/comments",
            data=json.dumps({'body': body}).encode(),
            headers={'Authorization': f'Bearer {token}', 'Accept': 'application/vnd.github+json'}, method='POST')
        with urllib.request.urlopen(request) as response:
            if response.status != 201:
                raise RuntimeError(f'Evidence publication failed: {response.status}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--sha', required=True)
    parser.add_argument('--run-url', required=True)
    parser.add_argument('--publish', action='store_true')
    parser.add_argument('--coverage', type=Path)
    args = parser.parse_args()
    data = build(json.loads(args.input.read_text()), ROOT, args.sha, args.run_url)
    if args.coverage:
        attach_coverage(data, json.loads(args.coverage.read_text()), ROOT)
    args.output.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    args.output.with_suffix('.md').write_text(render(data))
    print('::notice title=File evidence subsystem outcomes::' + json.dumps(data['rows'], ensure_ascii=False))
    if args.publish:
        publish(data)


if __name__ == '__main__':
    main()
