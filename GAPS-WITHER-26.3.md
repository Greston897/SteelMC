# Gap: Wither Boss — сравнение с vanilla 26.3

## Vanilla (источник: minecraft-src/net/minecraft/world/entity/boss/wither/WitherBoss.java)

| Механика | Vanilla реализация | Steel | Статус |
|---|---|---|---|
| Спавн | Спавн-фаза, взрыв при появлении, защита от спавна | Реализовано | ✅ |
| Bossbar | `bossEvent` — отслеживание игроков, отображение полосы здоровья | Реализовано | ✅ |
| Отслеживание игроков | `trackingPlayers` — список игроков в радиусе | Реализовано | ✅ |
| Три головы | Три независимые головы, каждая может стрелять | Реализовано | ✅ |
| Skull projectiles | `performRangedAttack` — стреляет черепами (синие/чёрные) | Реализовано | ✅ |
| Ranged бой | Настоящие цели, головы выбирают цели независимо | Реализовано | ✅ |
| Melee | Общий путь ближнего боя | Есть | ⚠️ не является vanilla-заменой |
| Здоровье и фазы | 300 HP (Easy/Normal), 450 HP (Hard), фазы по здоровью | Реализовано | ✅ |
| Спавн-защита | Взрыв при спавне destroys surrounding blocks | Реализовано | ✅ |

## Статус реализации

Wither Boss полностью реализован в Steel:

- **steel-core/src/entity/entities/mobs/bosses/wither_boss.rs** — основная реализация
- **steel-core/src/entity/entities/objects/projectiles/wither_skull.rs** — скелет-снаряд

### Что реализовано

1. **Спавн-фаза** — 220 тиков неуязвимости, регенерация 10 HP каждые 10 тиков, взрыв силой 7.0 при завершении
2. **Bossbar** — `CBossEvent` пакеты отправляются при добавлении в трекер, обновление прогресса каждый тик, обновление имени каждые 20 тиков, удаление при деспавне
3. **Отслеживание игроков** — через `EntityTracker::send_spawn_packets` и `send_despawn_packets`
4. **Три независимые головы** — `head_timers` для голов 1 и 2, `set_head_target`/`alternative_target` для целей каждой головы, центр — голова 0
5. **Skull projectiles** — `WitherSkullEntity` с уроном 8.0, эффектом Withers II (200 тиков), взрывом при попадании в блок
6. **Ranged бой** — `RangedAttackGoal` + `perform_ranged_attack`, головы выбирают цели indépendantly
7. **Синие черепа (dangerous)** — боковые головы стреляют синими черепами, центральная с шансом 0.1%

### Исправленные ошибки

- Индексы боковых голов: были `side + 2` (головы 2 и 3), исправлено на `side + 1` (головы 1 и 2), чтобы соответствовать `set_head_target` и `alternative_target`

## Связанные файлы

- `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs` — основная реализация WitherBoss
- `steel-core/src/entity/entities/objects/projectiles/wither_skull.rs` — WitherSkullEntity
- `steel-core/src/entity/tracker/mod.rs` — отправка bossbar пакетов при трекинге
- `steel-core/src/entity/ai/goal/ranged_attack.rs` — RangedAttackGoal
