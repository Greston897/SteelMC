# Где смотреть состояние и как проверять

0. **Замеренный аудит реализации и реальные запуски на SHA:**
   [MEASURED-STATUS-26.3-RU.md](MEASURED-STATUS-26.3-RU.md) и
   [VANILLA-PARITY-MEASURED-26.3.md](VANILLA-PARITY-MEASURED-26.3.md).
   Инструмент `.ci/vanilla_parity_audit.py` пересчитывает покрытие по
   `steel-core/build/classes.json` и исходникам `minecraft-src/`; самопроверки —
   `.ci/test_vanilla_parity_audit.py`. Неизвестное остаётся неизвестным.
1. **Что реально прошло в Actions:**
   [SUBSYSTEMS-26.3-ACTIONS.md](SUBSYSTEMS-26.3-ACTIONS.md).
   Таблица всех зон, 4802 passed / 0 failed / 7 ignored, SHA и ссылки на run.
2. **Что реализовано, а где ещё частичная реализация:**
   [SUBSYSTEMS-26.3-STATUS-RU.md](SUBSYSTEMS-26.3-STATUS-RU.md).
   60 явно перечисленных крупных возможностей, без выдуманного процента игры.
3. **Как проверять непроверенное:**
   [MANUAL-MECHANICS-26.3-RU.md](MANUAL-MECHANICS-26.3-RU.md).
   107 опытов с шагами и ожидаемыми результатами; затем очередь 547 файлов
   с конкретными триггерами. Сначала vanilla, потом Steel. Результат каждого
   опыта и каждого объекта записывается отдельно.
4. **Что обнаружено при сверке с vanilla 26.3:**
   [VANILLA-26.3-GAPS-RU.md](VANILLA-26.3-GAPS-RU.md).
5. **Где в исходниках явно написано про ограничения:**
   [SOURCE-LIMITATIONS-26.3-RU.md](SOURCE-LIMITATIONS-26.3-RU.md).
   57 файлов-кандидатов, с исходными фрагментами. Это очередь аудита;
   шаблонный комментарий не доказывает актуального отсутствия кода.
6. **Поимённые тесты из исходников и размеры всех подсистем:**
   [PROGRESS-26.3-INVENTORY.md](PROGRESS-26.3-INVENTORY.md).
   Статический индекс, не подмена результатов запуска.

Непройденные инструкции имеют статус NOT RUN. Известные отсутствующие пути
не объявляются готовыми: ещё есть captain succession, полное восстановление
рейдеров, специальные AI, Wither ranged/bossbar, runtime datapacks и расхождения
полного worldgen/Flint. Все механики vanilla за этот проход не закончены.
