#!/usr/bin/env python3
"""Render an explicitly subjective implementation checklist, never test parity."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

def validate_updates(data):
    """Only confirmed, source-backed runs belong in the verified ledger."""
    seen = set()
    for update in data.get('verified_updates', []):
        assert update['result'] == 'success'
        assert isinstance(update['run'], int) and update['run'] > 0
        assert update['run'] not in seen
        seen.add(update['run'])
        assert 7 <= len(update['commit']) <= 40
        assert all(c in '0123456789abcdef' for c in update['commit'])
        assert update['limit'] and update['sources']
        for source in update['sources']:
            assert (ROOT / source).exists(), source

def main():
    data = json.loads((ROOT / '.ci/implementation_assessment.json').read_text())
    validate_updates(data)
    print('# Реализация всего ядра — укрупнённая инженерная оценка\n')
    print('Это **оценка по коду, не процент vanilla-паритета и не доля прошедших тестов**. '
          'Охвачены все 15 основных подсистем, но только 60 явно перечисленных возможностей. '
          'Не выполнен аудит каждого метода и каждой механики Minecraft.\n')
    print('Шкала: **1** — основной исполняемый путь реализован; **0,5** — частично, '
          'известны ограничения либо полнота не установлена; **0** — возможность не '
          'засчитана. Вес каждого пункта одинаковый. Это условная инженерная шкала: '
          '0,5 не означает, что измерена ровно половина исходников. Изменение списка '
          'или весов меняет итог. Не читать показатель как «осталось ровно столько-то работы».\n')
    print('| Подсистема | Баллы / 4 | Оценка по чеклисту |\n|---|---:|---:|')
    total = 0
    count = 0
    for area in data['areas']:
        criteria = area['criteria']
        assert len(criteria) == 4
        for criterion in criteria:
            assert criterion['score'] in (0, .5, 1)
            assert (ROOT / criterion['source']).exists(), criterion['source']
        points = sum(c['score'] for c in criteria)
        total += points
        count += len(criteria)
        print(f"| {area['name']} | {points:g} / 4 | {points / 4:.1%} |")
    print(f'| **Итого** | **{total:g} / {count}** | **{total/count:.1%}** |\n')
    print('**Граница достоверности:** наличие основного пути не доказывает отсутствие '
          'багов или всех исключений. Это воспроизводимый расчёт из ручных оценок '
          '`.ci/implementation_assessment.json`, а не автоматически измеренная '
          'готовность всего Minecraft. Точное общее значение по всем механикам неизвестно.\n')
    print('## Основания каждого балла\n')
    for area in data['areas']:
        print(f"### {area['name']}\n")
        for c in area['criteria']:
            print(f"- **{c['score']:g}** — {c['capability']}: `{c['source']}`.")
        print()
    print('## Подтверждённые дополнения после исходного аудита\n')
    print('Дополнения включены в основания оценки, но не повышают баллы широких '
          'критериев «все механизмы»: непроверенные ветви остаются. '
          'Success ниже относится к конкретному SHA и workflow, не к текущему HEAD автоматически.\n')
    for update in data.get('verified_updates', []):
        print(f"- **{update['area']} — {update['capability']}**: `{update['commit']}`, "
              f"[{update['workflow']} {update['run']}](https://github.com/Greston897/SteelMC/actions/runs/{update['run']}) — success.")
        print('  Источники: ' + ', '.join(f'`{s}`' for s in update['sources']) + '.')
        print(f"  Граница проверки: {update['limit']}\n")
    print('## Открытые вопросы отдельно от оценки\n\n'
          '- Известное расхождение Overworld/features исправлено в проверенной выборке '
          '2500 чанков; более широкий паритет не доказан.\n'
          '- Flint и другие проверки нужно оценивать по конкретному запуску: '
          'ранее обнаружены периодические отказы, а отдельный успех их не исключает.\n'
          '- Для новых исправлений окончательные расширенные результаты нужно сверять с Actions.\n'
          '- Оценка не даёт гарантии TPS, производительности, сетевой совместимости или отсутствия потери данных.\n')

if __name__ == '__main__':
    main()
