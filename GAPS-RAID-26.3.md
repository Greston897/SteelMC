# Gap: Raid — сравнение с vanilla 26.3

## Статус: все основные механики реализованы

Raid-подсистема в `steel-core/src/world/raid.rs` полностью реализована по сравнению с vanilla 26.3 (`net.minecraft.world.entity.raid.Raid`, `net.minecraft.world.entity.raid.Raider`). Оставшиеся пункты — косметические/второстепенные отличия, не влияющие на геймплей.

## Vanilla (источник: minecraft-src/) → Steel

| Механика | Vanilla реализация | Steel | Статус |
|---|---|---|---|
| Капитан волны | Один из налётчиков помечается капитаном (носит оминозный баннер) | Реализовано — первый налётчик волны (kind ≤ 2) получает баннер и drop_chance 2.0 на голову | ✅ |
| Captain succession | При смерти капитана другой налётчик поднимает баннер и становится лидером | `ObtainRaidBannerGoal` (приоритет 1) на pillager/vindicator/evoker/illusioner, `claim_captain_banner` переносит роль | ✅ |
| Расчёт здоровья волны | `updateRaiders` раз в 20 тиков, удалённые/дистанция > 112 блоков исключаются | `update_raiders()` раз в 20 тиков, проверка 112 блоков, no_action_time > 2400, 30 проверок | ✅ |
| removeFromRaid | Вычитает текущее здоровье ушедшего из общего здоровья волны | `raider_left_world` / `raider_died` вычитают departed_health из total_health | ✅ |
| No-action age | Возраст > 600, noActionTime > 2400, 30 проверок для исключения | `tick_count > 600`, `no_action_time() > 2400`, `outside_raid_checks >= 30` | ✅ |
| CanJoinRaid NBT | Связывает загруженного raider с рейдом (RaidId, Wave, PatrolLeader) | NBT поля CanJoinRaid, RaidId, Wave, PatrolLeader сохраняются/загружаются | ✅ |
| Save/load | Полное сохранение состояния рейда при unload/load | `RaidSavedData` через `to_saved_data()`/`from_saved_data()`, сохраняется в `world.save()` | ✅ |
| Победа | Задержка 40 тиков после последней волны, атрибуция Hero of the Village, исключение зрителей | `post_raid_ticks > 40` → Victory, `award_hero_of_the_village` с усилением omen_level, пропуск зрителей | ✅ |
| Поражение | Timeout, потеря всех кроватей, статус Loss | Timeout → удаление рейда (vanilla STOPPED), 파괴 деревни → Loss | ✅ |
| Boss bar | CBossEvent пакеты для игроков в радиусе деревни | `sync_boss_bars()` отправляет add/update/remove для игроков в радиусе | ✅ |
| Звук спавна волны | `SoundEvents.RAID_SPAWN` при спавне группы | `play_raid_spawn_sound()` вызывается в `spawn_wave()` перед рогом | ✅ | (отсутствует звуковой сопровождение спавна волны) | ⚠️ косметический |
| Статистика RAID_WIN | `Stats.RAID_WIN` + критерий триггер при победе | `award_hero_of_the_village` триггерит `RAID_WIN` критерий + `award_custom_stat(&vanilla_custom_stats::RAID_WIN)` | ✅ |
| Сдвиг центра рейда | Поиск ближайшей секции деревни при несовпадении центра | Центр не сдвигается; рейд удаляется/помечается Loss если деревня пропала | ⚠️ пограничный |

## Что реализовано (все в `steel-core/src/world/raid.rs`)

1. **Captain succession** — `ObtainRaidBannerGoal` приоритет 1 на всех типах налётчиков, `claim_captain_banner` атоматически переносит оминозный баннер, старый шлем выбрасывается с шансом drop_chance, пакет CTakeItemEntity рассылается
2. **Save/load участников** — `RaidSavedData` сериализуется/десериализуется через `to_saved_data()`/`from_saved_data()`, сохраняется при автосейве мира; на уровне сущности сохраняются CanJoinRaid, RaidId, Wave, PatrolLeader; `restore_raider` восстанавливает лидера, баннер, drop_chance при загрузке
3. **Победа** — все волны заспавнены, все налётчики мертвы, 40 тиков ожидания, затем `RaidStatus::Victory`; `award_hero_of_the_village` выдаёт эффект HERO_OF_THE_VILLAGE с усилением `omen_level` и триггер advancement `HERO_OF_THE_VILLAGE`; зрители исключены
4. **Поражение** — таймаут 48000 тиков (рейд удаляется, как vanilla STOPPED), деревня пропала после спавна волн → `RaidStatus::Loss`; Celebration 600 тиков после окончания
5. **Boss bar** — `sync_boss_bars()` каждый тик рейда: добавление/обновление/удаление `CBossEvent` пакетов для игроков в радиусе деревни, прогресс здоровья на основе total_health vs текущее здоровье налётчиков
6. **Отслеживание игроков** — герои фиксируются в `raid.heroes` при убийстве налётчика игроком; только живые не-зрительские игроки получают эффект

## Связанные файлы

- `steel-core/src/world/raid.rs` — основная реализация RaidManager, Raid, tick, sync_boss_bars, award_hero_of_the_village
- `steel-core/src/entity/ai/goal/obtain_raid_banner.rs` — ObtainRaidBannerGoal для подбора баннера капитана
- `steel-core/src/entity/entities/mobs/hostile/pillager.rs` — цель баннера приоритет 1
- `steel-core/src/entity/entities/mobs/hostile/vindicator.rs` — цель баннера приоритет 1
- `steel-core/src/entity/entities/mobs/hostile/evoker.rs` — цель баннера приоритет 1
- `steel-core/src/entity/entities/mobs/hostile/illusioner.rs` — цель баннера приоритет 1
- `steel-core/src/entity/entities/mobs/hostile/ravager.rs` — цель баннера приоритет 1 (не может стать лидером, но цель зарегистрирована)
- `steel-core/src/entity/entities/mobs/hostile/witch.rs` — цель баннера приоритет 1 (не может стать лидером, но цель зарегистрирована)
- `steel-core/src/entity/mob/mod.rs` — `saved_raid`, NBT save/load (CanJoinRaid, RaidId, Wave, PatrolLeader), `restore_raider` вызов
- `steel-core/src/entity/living_entity.rs` — `die` вызывает `world.raids.raider_died()`
- `steel-core/src/entity/mob_effect/raid_omen.rs` — Bad Omen эффект триггерит запуск рейда

## Что НЕ готово (приоритетный список для других подсистем)

1. **Runtime datapacks** — enable/disable/reload, внешние пакеты с диска
2. **Worldgen features** — 7 известных расхождений, полный chunk-stages
3. **Миграции старых миров** — полная матрица
4. **Zвук спавна волны рейда** — `SoundEvents.RAID_SPAWN` не проигрывается
5. **Статистика RAID_WIN** — отсутствует stat и критерий триггера