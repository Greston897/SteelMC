#!/usr/bin/env python3
"""Render an explicitly subjective implementation checklist, never test parity."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

def main():
    data = json.loads((ROOT / '.ci/implementation_assessment.json').read_text())
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
    print('## Открытые ошибки отдельно от оценки\n\n'
          '- Overworld FEATURES: известное расхождение хешей не исправлено.\n'
          '- Полный Flint остаётся красным; исправление отдельных механик не закрывает весь набор.\n'
          '- Для новых исправлений окончательные расширенные результаты нужно сверять с Actions.\n'
          '- Оценка не даёт гарантии TPS, производительности, сетевой совместимости или отсутствия потери данных.\n')

if __name__ == '__main__':
    main()
