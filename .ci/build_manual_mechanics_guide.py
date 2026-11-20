"""Build a human test plan and an explicit source-object audit queue.

This is a plan, never evidence that a mechanic passed. No coverage percentage
is calculated from source files, test names or the number of manual cases.
"""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parent.parent
CASES = ROOT / '.ci/manual_mechanics_cases.tsv'
TEST = re.compile(r'#\[(?:test|tokio::test)[^\]]*\]\s*(?:async\s+)?fn\s+(\w+)')


def load_cases(path=CASES):
    cases = []
    seen = set()
    for number, line in enumerate(path.read_text().splitlines(), 1):
        if not line or line.startswith('#'):
            continue
        fields = line.split('\t')
        if len(fields) != 6 or any(not field.strip() for field in fields):
            raise ValueError(f'{path}:{number}: expected six nonempty columns')
        if fields[0] in seen:
            raise ValueError(f'duplicate case {fields[0]}')
        seen.add(fields[0])
        cases.append(dict(zip(('id', 'area', 'name', 'setup', 'action', 'expected'), fields)))
    return cases


# Source paths are audit units, not a claim that each file is one mechanic.
FAMILIES = [
    ('Поведение блоков', 'steel-core/src/behavior/blocks', ['BLK11', 'BLK12'],
     {'redstone': ['RED01', 'RED02', 'RED03', 'RED04', 'RED06'],
      'container': ['INV01', 'BLK01', 'BLK02', 'BLK04', 'CHK03'],
      'vegetation': ['BLK11', 'WRL04'], 'portal': ['PHY04'], 'fluid': ['PHY02', 'BLK12'],
      'decoration': ['BLK08', 'BLK09', 'BLK12'], 'building': ['BLK12', 'BLK13']}),
    ('Предметы', 'steel-core/src/behavior/items', ['INV01', 'INV04', 'DATA02'], {}),
    ('Block entities', 'steel-core/src/block_entity/entities', ['CHK03', 'INV01', 'BLK02'], {}),
    ('Мобы', 'steel-core/src/entity/entities/mobs', ['MOB01', 'NET05', 'CHK03'],
     {'bosses': ['BOSS01', 'BOSS02'], 'passive': ['MOB02', 'MOB10', 'MOB11', 'MOB12'],
      'hostile': ['MOB01', 'MOB05', 'MOB06', 'MOB07', 'MOB08', 'MOB09', 'RAID08']}),
    ('Объекты и снаряды', 'steel-core/src/entity/entities/objects', ['NET05', 'COM04', 'CHK03'],
     {'vehicles': ['RED05', 'PHY01', 'INV01'], 'explosives': ['MOB06', 'DATA02']}),
    ('Команды', 'steel-core/src/command/builtins', ['SYS03', 'CMD01', 'CMD02', 'CMD03', 'CMD04'], {}),
]
SPECIFIC = {
    'fishing_rod': ['ITEM01'], 'brush': ['ITEM02'], 'lead': ['ITEM03'], 'name_tag': ['ITEM03'],
    'shears': ['ITEM04'], 'block_transformer': ['ITEM04'], 'mace': ['ITEM05'], 'wind_charge': ['ITEM05'],
    'ender_pearl': ['ITEM06'], 'ender_eye': ['ITEM06'], 'firework_rocket': ['ITEM07'],
    'bucket': ['ITEM08'], 'solid_bucket_item': ['ITEM08'], 'mob_bucket': ['ITEM08'], 'bottle': ['ITEM08'],
    'wolf': ['MOB13'], 'cat': ['MOB13'], 'ocelot': ['MOB13'],
    'sheep': ['MOB14'], 'cow': ['MOB14'], 'mooshroom': ['MOB14'], 'chicken': ['MOB14'],
    'goat': ['MOB15'], 'armadillo': ['MOB15'], 'sniffer': ['MOB15'],
    'frog': ['MOB16'], 'tadpole': ['MOB16'], 'slime': ['MOB16'], 'magma_cube': ['MOB16'],
    'panda': ['MOB17'], 'parrot': ['MOB17'], 'rabbit': ['MOB17'], 'turtle': ['MOB18'],
    'cod': ['MOB19'], 'salmon': ['MOB19'], 'tropical_fish': ['MOB19'], 'pufferfish': ['MOB19'],
    'llama': ['MOB20'], 'trader_llama': ['MOB20'], 'sulfur_cube': ['MOB21'],
    'camel_husk': ['MOB22'], 'skeleton_horse': ['MOB22'], 'zombie_horse': ['MOB22'],
    'blaze': ['MOB23'], 'breeze': ['MOB23'], 'snow_golem': ['MOB23'], 'iron_golem': ['MOB23'],
    'vex': ['MOB24'], 'evoker': ['MOB24', 'RAID08'], 'illusioner': ['MOB24'],
    'wither_boss': ['BOSS01'], 'villager': ['MOB03', 'MOB04'],
    'pillager': ['MOB05', 'RAID04'], 'ravager': ['RAID02'], 'vindicator': ['RAID08'],
}

HOOKS = {
    'random_tick': 'Дайте пройти random ticks; затем повторите с randomTickSpeed=0. Изменение не должно происходить без своего триггера.',
    'tick': 'Вызовите отложенное изменение, выгрузите чанк до его исполнения, загрузите и проверьте однократное завершение.',
    'use_item_on': 'Используйте подходящий и неподходящий предмет по каждой доступной грани; повторите с занятым инвентарём.',
    'use_without_item': 'Нажмите use пустой рукой, с предметом и при приседании; второй игрок проверяет то же состояние.',
    'neighbor_changed': 'Измените соседа с каждой стороны, уберите опору; проверьте, что изменение не зацикливает обновления.',
    'update_shape': 'Поставьте и уберите соседний блок и воду; сверьте итоговые block states и дроп.',
    'entity_inside': 'Проведите сквозь блок игрока, моба, предмет и транспорт; отдельно проверьте стояние без движения.',
    'on_place': 'Поставьте блок, замените его другим и верните; убедитесь, что прежнее состояние не осталось.',
    'on_remove': 'Сломайте, замените командой и взорвите в отдельных опытах; посчитайте предметы и содержимое.',
    'get_drops': 'Добудьте рукой, подходящим инструментом, с silk touch и fortune; сравните предметы и XP.',
    'can_survive': 'Уберите опору, свет или воду, необходимые этому объекту; сравните момент разрушения и дроп.',
    'use_on': 'Используйте предмет по подходящей цели, воздуху и запрещённой цели; сверьте расход и durability.',
    'finish_using_item': 'Полностью используйте предмет; затем прервите использование перед концом; сравните результат и расход.',
    'release_using': 'Отпустите использование сразу, в середине и после полной зарядки; измерьте результат и durability.',
    'save_additional': 'Запишите data get до stop/start и после; отдельно проверьте выгрузку без перезапуска.',
    'load_additional': 'Откройте сохранённый экземпляр; проверьте владельца, варианты, экипировку и рабочие таймеры.',
    'register_goals': 'Проверьте конкуренцию целей: еда/опасность/игрок/атакующий; закройте прямой путь и затем откройте.',
    'hurt_server': 'Нанесите melee, projectile, fire и explosion damage раздельно; сверьте иммунитеты и реакцию.',
}


def source_objects():
    objects = []
    for family, directory, default, by_part in FAMILIES:
        for path in sorted((ROOT / directory).rglob('*.rs')):
            if path.stem in {'mod', 'tests', 'test'} or 'tests' in path.parts or path.stem.endswith('_tests'):
                continue
            text = path.read_text()
            refs = list(default)
            for part, ids in by_part.items():
                if part in path.relative_to(ROOT / directory).parts:
                    refs = ids
            refs = SPECIFIC.get(path.stem, refs)
            hooks = [name for name in HOOKS if re.search(r'\bfn\s+' + name + r'\s*\(', text)]
            objects.append({'family': family, 'path': str(path.relative_to(ROOT)),
                            'name': path.stem, 'cases': refs, 'hooks': hooks,
                            'tests': TEST.findall(text)})
    return objects


def render():
    cases = load_cases()
    objects = source_objects()
    ids = {case['id'] for case in cases}
    assert all(set(obj['cases']) <= ids for obj in objects)
    out = ['# Как проверять механику SteelMC против vanilla 26.3', '',
           'Это рабочая инструкция, а не отчёт о пройденных проверках. Все пункты ниже имеют статус **НЕ ПРОВЕРЕНО этим ручным прогоном**. Даже если часть логики покрыта Rust-тестом, весь описанный сценарий не считается пройденным.', '',
           '## С чего начать', '',
           '1. Два отдельных сервера: официальный vanilla 26.3 и Steel с записанным SHA. Нельзя открывать один мир сразу двумя процессами. Работайте с копиями.',
           '2. Одинаковые seed, difficulty, gamerules, view/simulation distance. Для боевых сценариев survival; creative только для подготовки. Зафиксируйте координаты и время.',
           '3. Сначала выполните опыт на vanilla. Запишите числа и фактическое поведение. Затем повторите на Steel из того же начального состояния. Для случайных исходов нужна серия; один удачный исход не доказывает распределение.',
           '4. Не используйте kill/teleport для проверки естественного поиска пути или добивания: команда меняет проверяемые условия. Tick freeze/step допустимы только там, где одинаково применены на обоих серверах.',
           '5. Для сетевой части нужны два клиента 26.3. Для серверной логики человек не обязателен: эти сценарии также являются очередью для автоматизации в Actions.',
           '6. После каждого опыта сохраните результат. FAIL — наблюдаемое расхождение. BLOCKED — нельзя дойти до опыта, например функция не реализована. NOT RUN — опыт не запускали. Пустой чекбокс не PASS.', '',
           'Шаблон результата:', '', '```text', 'Case / объект:', 'Steel SHA / Vanilla version:',
           'Seed / dimension / xyz / difficulty / gamerules:', 'Подготовка и точные команды:',
           'На vanilla получилось:', 'На Steel получилось:', 'PASS / FAIL / BLOCKED / NOT RUN:',
           'Лог / видео / NBT до и после:', 'Повтор после reconnect / выгрузки / stop-start:', '```', '',
           'Числовые ID блоков между серверами не сравнивайте напрямую: сравнивайте namespaced ID и properties. Команды и NBT-поля уточняйте автодополнением именно 26.3; старые гайды по другой версии не эталон.', '',
           f'## Основные сценарии: {len(cases)}', '',
           'В каждом сценарии повторите опыт после переподключения, выгрузки чанка и штатного перезапуска, если объект сохраняется. Результаты этих трёх вариантов записывайте отдельно.', '']
    area = None
    for case in cases:
        if area != case['area']:
            area = case['area']
            out += [f'## {area}', '']
        out += [f"### {case['id']} — {case['name']}", '',
                f"**Подготовка.** {case['setup']}", '', f"**Действия.** {case['action']}", '',
                f"**Что сравнить.** {case['expected']}", '',
                '**Результат:** NOT RUN. Приложите результат vanilla и Steel, а не только скриншот Steel.', '']
    out += ['## Пообъектная очередь аудита', '',
            f'Ниже {len(objects)} файлов поведения/объектов. Это не {len(objects)} механик: в одном файле может быть несколько объектов, а наследуемая механика может находиться в другом. Включены и файлы с unit-тестами: наличие теста в файле не доказывает весь объект.', '',
            'Имя файла — указатель в код, не гарантированный registry ID. Предмет или сущность выбирайте по регистрации в этом файле. Общие помощники проверяйте через вызывающие объекты. Если указанный семейный сценарий к объекту неприменим, запишите N/A с причиной и добавьте специфический сценарий, а не ставьте PASS.', '',
            'Для каждого объекта нужно: выполнить подходящие связанные сценарии; проверить перечисленные ниже триггеры; сохранить имя объекта и результаты отдельно. Если специальная способность не описана, это пробел плана, который надо внести отдельным case. Полный список всех механик vanilla здесь пока не установлен.', '']
    family = None
    for i, obj in enumerate(objects, 1):
        if family != obj['family']:
            family = obj['family']
            out += [f'## Объекты: {family}', '']
        out += [f"### OBJ{i:04d} — {obj['name']}", '', f"Код: `{obj['path']}`.", '',
                'Сценарии: ' + ', '.join(obj['cases']) + '. Статус объектного сравнения: **NOT RUN**.', '']
        for hook in obj['hooks']:
            out += [f'- `{hook}`: {HOOKS[hook]}']
        if not obj['hooks']:
            out += ['- Проверьте основной сценарий семейства. В этом файле не найден известный генератору hook; это не означает отсутствия специальной механики.']
        out += ['', 'Локальные тестовые функции в файле (не доказательство клиентского прогона): ' +
                (', '.join(f'`{name}`' for name in obj['tests']) or 'не найдены; искать также в соседних tests-модулях и интеграционных тестах') + '.', '']
    out += ['## Чего этот файл не обещает', '',
            '- Это не исчерпывающая спецификация vanilla. Реестры, datapack-driven поведение и наследуемые AI goals требуют дополнительного сопоставления.',
            '- Не переносите серверную механику в «проверить игроками» только потому, что ей нужен объект Player. Большинство логики можно и нужно автоматизировать.',
            '- Известные незавершённые пути (Wither volleys/bossbar, runtime datapacks, raid captain succession и восстановление участников) не становятся реализованными от наличия инструкции.',
            '- Актуальные фактические запуски и остаточные ограничения — в READINESS-26.3-FACTS.md. Поимённый статический индекс — PROGRESS-26.3-INVENTORY.md.', '']
    return '\n'.join(out)


if __name__ == '__main__':
    (ROOT / 'MANUAL-MECHANICS-26.3-RU.md').write_text(render())
    print(f'{len(load_cases())} scenarios; {len(source_objects())} source audit units; NOT RUN by default')
