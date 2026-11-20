# Открытые ограничения: что прямо отмечено в коде

Это **кандидаты на аудит**, а не автоматически подтверждённые 100% отсутствующие функции. Комментарий может устареть, слово placeholder может относиться к тестовой фикстуре. Поэтому ниже оставлен исходный текст с координатами, а не выдуман точный процент готовности.

Найдено 57 файлов steel-core с маркерами отклонения/незавершённости. Нулевое число todo! не означало отсутствия недописанных механик.

Для каждого пункта: прочитайте вызывающий путь; проверьте, не появилась ли реализация после комментария; выполните указанный сценарий сначала на vanilla 26.3; запишите CONFIRMED GAP / STALE COMMENT / TEST-ONLY / NEEDS AUDIT. Пока все пункты — NEEDS AUDIT, кроме отдельно подтверждённых в VANILLA-26.3-GAPS-RU.md.

## GAP001 — `steel-core/src/behavior/block/context.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 21–36:

```rust
///
/// `NoEntity` is distinct from `Unimplemented`: some Vanilla blocks, notably the moving-piston
/// placeholder, intentionally return no entity from normal block creation even though their
/// state accepts an explicitly-created block entity.
pub enum BlockEntityCreation {
    /// The block factory created its entity.
    Created(SharedBlockEntity),
    /// The implemented Vanilla factory intentionally created no entity.
    NoEntity,
    /// Steel has not implemented this block's factory yet.
    Unimplemented,
}

impl BlockEntityCreation {
    /// Converts an optional registered implementation into a factory result.
    #[must_use]
```

## GAP002 — `steel-core/src/behavior/blocks/vegetation/turtle_egg_block.rs`

Статус: **NEEDS AUDIT**. Сценарии: BLK11, WRL04.

Строки 386–394:

```rust

        // Final advance hatches the egg and removes the block. Spawning the baby
        // turtle is stubbed until the Turtle entity lands, so only the removal is
        // asserted here.
        behavior.random_tick(world.get_block_state(pos), world, pos);
        assert!(world.get_block_state(pos).is_air());
    }

    #[test]
```

## GAP003 — `steel-core/src/block_entity/entities/unimplemented.rs`

Статус: **NEEDS AUDIT**. Сценарии: CHK03, INV01, BLK02.

Строки 15–23:

```rust
}

/// Steel-specific fallback for block entity types whose runtime behavior is not implemented yet.
///
/// Vanilla has concrete classes for every block entity type. Steel uses this only to preserve
/// worldgen and disk NBT until the corresponding typed implementation is added.
pub struct UnimplementedBlockEntity {
    base: BlockEntityBase,
    state: SyncMutex<UnimplementedBlockEntityState>,
```

## GAP004 — `steel-core/src/chunk/chunk_pyramid.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 225–234:

```rust

impl ChunkStep {
    /// A placeholder step used for array initialization.
    const PLACEHOLDER: Self = Self {
        target_status: ChunkStatus::Empty,
        direct_dependencies: ChunkDependencies::EMPTY,
        accumulated_dependencies: ChunkDependencies::EMPTY,
        block_state_write_radius: -1,
        task: noop_task,
    };
```

Строки 275–283:

```rust
    const fn new() -> Self {
        Self {
            steps: [ChunkStep::PLACEHOLDER; STATUS_COUNT],
            count: 0,
        }
    }

    const fn step(
        mut self,
```

## GAP005 — `steel-core/src/chunk_saver/format.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 1088–1096:

```rust
        projection: i8,
    },
    /// Empty placeholder element.
    Empty,
    /// Placed feature element.
    Feature {
        /// Feature identifier.
        feature: Identifier,
        /// Projection mode: 0 = rigid, 1 = terrain matching.
```

## GAP006 — `steel-core/src/chunk_saver/ram_only.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 16–24:

```rust
///
/// Loading a world from another storage backend and cloning it for easy world
/// handling is not wired yet; this storage only serves generated or in-memory
/// worlds.
pub struct RamOnlyStorage {
    /// This saves every chunk, and it saves the changes in the world to make it possible to run the server fully in memory
    saved_chunks: AsyncRwLock<FxHashMap<ChunkPos, SimpleRAMChunk>>,
}

```

## GAP007 — `steel-core/src/command/builtins/data.rs`

Статус: **NEEDS AUDIT**. Сценарии: SYS03, CMD01, CMD02, CMD03, CMD04.

Строки 277–285:

```rust
        "steel.commands.data.unsupported_write",
        format!(
            "Writing {what} NBT is not supported yet; the saved form only carries the fields Steel syncs"
        ),
        Vec::new(),
    ));
    Ok(0)
}

```

## GAP008 — `steel-core/src/command/builtins/datapack.rs`

Статус: **NEEDS AUDIT**. Сценарии: SYS03, CMD01, CMD02, CMD03, CMD04.

Строки 83–90:

```rust
    context.source().send_failure(translated(
        "commands.datapack.unknown",
        "Steel loads one built-in data pack; loading packs from disk is not supported yet"
            .to_owned(),
        Vec::new(),
    ));
    Ok(0)
}
```

## GAP009 — `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs`

Статус: **NEEDS AUDIT**. Сценарии: BOSS01.

Строки 4–13:

```rust
//! its invulnerability timer runs.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the boss bar and the three skull volleys are not implemented; the spawn
//!   phase (220 protected ticks, escalating regeneration, closing blast) and the
//!   synced target/invulnerability fields are.
use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
```

## GAP010 — `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the cave spider's poison bite is not applied yet (no per-hit effect hook
//!   on `MeleeAttackGoal`), so a bite only deals its 2.0 attack damage.

use std::iter::empty;
use std::sync::Weak;

```

## GAP011 — `steel-core/src/entity/entities/mobs/hostile/creaking.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 3–11:

```rust
//! Pale-garden mob. While the creaking heart that spawned it is intact the
//! creaking is invulnerable; breaking the heart kills it. Vanilla's "dissolve in
//! daylight / freeze when watched" rules are not implemented.
//!
//! The AI is the shared `Monster` skeleton (float, melee attack, random stroll,
//! look goals) plus the standard hostile target selector; attributes and
//! dimensions come from the extracted `entities.json`.
//!
use std::iter::empty;
```

## GAP012 — `steel-core/src/entity/entities/mobs/hostile/creeper.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! powered). Losing the target or the line of sight deflates it again.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the vanilla `SwellGoal` is folded into the entity: the swell counter is
//!   advanced from `ai_step` and navigation is cancelled from
//!   `custom_server_ai_step`, which runs after the goal selectors — the visible
//!   behaviour (approach, stop, swell, deflate, explode) matches vanilla;
//! * `is_ignited` is synchronized but no igniter path sets it yet, and there is
//!   no lightning hook in the entity trait, so charged creepers cannot yet be
```

## GAP013 — `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 4–13:

```rust
//! differs in attributes, size and the mining-fatigue curse.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the laser beam and the mining-fatigue curse are not implemented.

use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
use rand::Rng;
```

## GAP014 — `steel-core/src/entity/entities/mobs/hostile/enderman.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 6–14:

```rust
//! behavior is vanilla (Endermen are not daylight-burning mobs).
//!
//! Deviations (documented in `STATE-26.3-RU.md`): natural spawning and the
//! block pickup/place goals are separate follow-up systems.

use std::iter::empty;
use std::sync::{Arc, Weak};

use glam::DVec3;
```

## GAP015 — `steel-core/src/entity/entities/mobs/hostile/hoglin.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * hoglins in vanilla flee warped fungus and only attack players that are not
//!   in the Nether-safe set; Steel keeps the plain hostile selector.

use std::iter::empty;
use std::sync::Weak;

```

## GAP016 — `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB23.

Строки 3–12:

```rust
//! A neutral village golem that retaliates when attacked.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * poppy offering and the crack overlay are not implemented; village defence is
//!   modelled as vanilla's `#minecraft:raiders` target selector.

use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
```

## GAP017 — `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB16.

Строки 9–23:

```rust
//! Steel keeps the shared formulas in [`super::cube_mob`].
//!
//! Deviations (documented in `STATE-26.3-RU.md`):
//! * natural spawning (nether wastes, basalt deltas, fortress, bastion, soul sand
//!   valley) is not wired: the entity type is registered without a spawn rule and
//!   only spawners and commands create it;
//! * the hopping goals are collapsed into `custom_server_ai_step` instead of a
//!   `MagmaCubeKeepOnJumpingGoal` / `SlimeRandomDirectionGoal` pair;
//! * `AbstractCubeMob.push` contact damage against iron golems is not wired,
//!   because Steel has no entity-push hook a mob can react to. The player path
//!   (`playerTouch`) is wired through [`Entity::player_touch`];
//! * lava particles and the squish animation are client-side in vanilla.

use std::iter::empty;
use std::sync::atomic::{AtomicI32, Ordering};
```

## GAP018 — `steel-core/src/entity/entities/mobs/hostile/piglin.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * vanilla piglins are neutral unless provoked or the player wears no gold
//!   armor, and barter with gold ingots; Steel registers them as always hostile
//!   and has no bartering.

use std::iter::empty;
use std::sync::Weak;
```

## GAP019 — `steel-core/src/entity/entities/mobs/hostile/pillager.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB05, RAID04.

Строки 5–13:

```rust
//! driven by the `minecraft:burn_in_daylight` entity type tag.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * raid patrols, raid waves, and natural spawning are separate systems; shots
//!   use the shared mob-arrow helper rather than the player crossbow pipeline.

use std::iter::empty;
use std::sync::{Arc, Weak};

```

## GAP020 — `steel-core/src/entity/entities/mobs/hostile/slime.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB16.

Строки 9–21:

```rust
//! `AbstractCubeMob`. Steel keeps the shared formulas in [`super::cube_mob`].
//!
//! Deviations (documented in `STATE-26.3-RU.md`):
//! * the hopping goals are collapsed into `custom_server_ai_step` instead of a
//!   `SlimeKeepOnJumpingGoal` / `SlimeRandomDirectionGoal` pair, so a slime
//!   always hops along its current facing instead of picking a random direction;
//! * `AbstractCubeMob.push` contact damage against iron golems is not wired,
//!   because Steel has no entity-push hook a mob can react to. The player path
//!   (`playerTouch`) is wired through [`Entity::player_touch`];
//! * the squish animation is client-side state in vanilla and is not synced.

use std::iter::empty;
use std::sync::atomic::{AtomicI32, Ordering};
```

## GAP021 — `steel-core/src/entity/entities/mobs/hostile/spider.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * vanilla spiders stop being hostile in bright light (they only retaliate);
//!   Steel keeps the plain hostile target selector, so they always attack.

use std::iter::empty;
use std::sync::Weak;

```

## GAP022 — `steel-core/src/entity/entities/mobs/hostile/vindicator.rs`

Статус: **NEEDS AUDIT**. Сценарии: RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the vindicator does not spawn with its iron axe and does not break doors.

use std::iter::empty;
use std::sync::Weak;

use glam::DVec3;
```

## GAP023 — `steel-core/src/entity/entities/mobs/hostile/warden.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 2–10:

```rust
//!
//! Deep-dark melee horror. It pulses vanilla's darkness effect onto nearby
//! players while it is angry; the shrieker summoning rules are not implemented.
//!
//! The AI is the shared `Monster` skeleton (float, melee attack, random stroll,
//! look goals) plus the standard hostile target selector; attributes and
//! dimensions come from the extracted `entities.json`.
//!
use std::iter::empty;
```

## GAP024 — `steel-core/src/entity/entities/mobs/hostile/zoglin.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * vanilla zoglins attack every living entity except other zoglins and creepers;
//!   Steel's target selector only considers players.

use std::iter::empty;
use std::sync::Weak;

```

## GAP025 — `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 9–17:

```rust
//! a villager that keeps the profession and fires `cured_zombie_villager`.
//!
//! Documented deviations from vanilla 26.3 (see `STATE-26.3-RU.md`):
//! * the profession-specific texture is client-side and not part of Steel.
//! * gossip and the reputation discount are not modelled; the cured villager
//!   starts with vanilla's converted-villager experience of zero.
use std::iter::empty;
use std::sync::Weak;

```

## GAP026 — `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB01, MOB05, MOB06, MOB07, MOB08, MOB09, RAID08.

Строки 7–15:

```rust
//! `minecraft:burn_in_daylight` entity type tag, not by this file.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * zombified piglins are neutral in vanilla: they only retaliate after being
//!   hit. Steel registers the plain hostile target selector for now, so they
//!   attack players on sight.

use std::iter::empty;
use std::sync::Weak;
```

## GAP027 — `steel-core/src/entity/entities/mobs/passive/allay.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 3–14:

```rust
//! A small flying helper that picks up items and carries a single stack, exactly
//! as vanilla's `Allay` does. Duplication and note-block dancing are documented
//! deviations.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * duplication (juke-box dancing) is not implemented; the dancing and
//!   `can_duplicate` synced flags are stored and synced as vanilla does.
//! * the carried stack is not rendered in-hand; it lives in the synced data.

use std::sync::{Arc, Weak};

use glam::DVec3;
```

## GAP028 — `steel-core/src/entity/entities/mobs/passive/armadillo.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB15.

Строки 4–15:

```rust
//! placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the armadillo rolling state machine (`ArmadilloState`) is not implemented;
//!   its synced enum is stored but never updated.
//! * scute shedding is not implemented.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
```

## GAP029 — `steel-core/src/entity/entities/mobs/passive/bee.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 4–14:

```rust
//! and look goals, plus the vanilla `checkAnimalSpawnRules` placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the species-specific goals (climbing, ramming, sliding, digging, ...) are
//!   not implemented yet; the shared animal goal set is used instead.
//! * food/tempt interaction uses the vanilla `#minecraft:goat_food`-style tags
//!   only where Steel already generates them; other species only breed through
//!   the generic `Animal::mob_interact_animal` path.

use std::sync::Weak;

```

## GAP030 — `steel-core/src/entity/entities/mobs/passive/camel.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 8–17:

```rust
//! `getJumpCooldown`, and the dash flag clears on landing.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * sitting poses and the two-passenger saddle are not implemented; the
//!   `last_pose_change_tick` synced field is kept.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
```

## GAP031 — `steel-core/src/entity/entities/mobs/passive/camel_husk.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB22.

Строки 4–13:

```rust
//! this implementation mirrors the camel and only differs in the entity type.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * dashing, sitting poses and two-passenger saddling are not implemented.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
```

## GAP032 — `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB19.

Строки 7–20:

```rust
//! `boundingBox.expand(0.3)` and the puff sounds.
//!
//! Deviations (documented):
//! * vanilla `InflateGoal` uses a `NOT_SCARY_FOR_PUFFERFISH` entity-type tag and
//!   `createNonAttackable().ignoreDistanceScalingFactor().ignoreVisibility()`;
//!   Steel approximates it with "a non-spectator, non-creative player within
//!   8 blocks" because the tag pipeline is not wired for this predicate yet.
//! * `playerTouch` (`onPlayerCollision`) is not wired: Steel has no
//!   `Entity.onPlayerCollision` hook for this entity class yet.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
```

## GAP033 — `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB19.

Строки 6–19:

```rust
//! `initialize`.
//!
//! Deviations (deliberate, documented):
//! * vanilla recalculates the dimensions when the tracked variant changes
//!   (`onSyncedDataUpdated`); Steel derives them on demand in
//!   `dimensions_for_pose`, which is observationally equivalent because the
//!   variant is the only input.
//! * the `SchoolingFish` leader/group bookkeeping is not wired yet: it needs an
//!   entity query (`world.get_entities_in_aabb_matching`) plus a
//!   `FollowFlockLeaderGoal`; `has_self_control` therefore stays `true`.

use std::sync::{Arc, Weak};

use glam::DVec3;
```

## GAP034 — `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB19.

Строки 8–22:

```rust
//! reused for the `Variant` entity NBT key and the `BucketVariantTag` bucket key.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * vanilla rolls a spawn variant out of a fixed list of 22 "common" packed
//!   variants (`TropicalFish.COMMON_VARIANTS`). That list is not part of the
//!   extracted data, so Steel rolls a uniform pattern and two uniform colours;
//!   the resulting packed value keeps the vanilla layout, only the distribution
//!   differs.
//! * `AbstractSchoolingFish` leader/group bookkeeping is not wired yet (same as
//!   cod and salmon); tropical fish use the default `getMaxGroupSize() == 1`,
//!   so no cod-shaped group refresh is skipped.
//! * Java edition exposes a single hitbox for both pattern bases (0.5 x 0.4 per
//!   the extracted `entities.json` and the wiki hitbox table), so the large
//!   patterns do not grow the box.

```

## GAP035 — `steel-core/src/entity/entities/mobs/passive/goat.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB15.

Строки 4–14:

```rust
//! and look goals, plus the vanilla `checkAnimalSpawnRules` placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the species-specific goals (climbing, ramming, sliding, digging, ...) are
//!   not implemented yet; the shared animal goal set is used instead.
//! * food/tempt interaction uses the vanilla `#minecraft:goat_food`-style tags
//!   only where Steel already generates them; other species only breed through
//!   the generic `Animal::mob_interact_animal` path.

use std::sync::Weak;

```

## GAP036 — `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 4–12:

```rust
//! predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * species-specific goals (taming, sitting, leashing, cube fusion) are not
//!   implemented; the shared animal goal set is used instead.
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
```

## GAP037 — `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 5–13:

```rust
//! implemented; jumping, galloping and the chest inventory are not.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * jumping, galloping, the chest inventory and breeding stats are not
//!   implemented.

use std::sync::Weak;

use glam::DVec3;
```

## GAP038 — `steel-core/src/entity/entities/mobs/passive/horse/horse.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 3–21:

```rust
//! Built on the shared `Animal` skeleton with the vanilla `AbstractHorse` flags
//! (`id_flags`) mirrored into the generated entity data. Horse-specific
//! mechanics are intentionally not wired yet.
//!
//! Riding works: a saddled horse is steered by the rider's look direction and
//! moves at its own `MOVEMENT_SPEED`, and taming by mounting is in.
//!
//! The rider jump works too: `START_RIDING_JUMP` stores the charge, the first
//! grounded tick applies `AbstractHorse.executeRidersJump`, and the horse emits
//! `entity.horse.jump`.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * galloping and breeding stats are not implemented.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
```

## GAP039 — `steel-core/src/entity/entities/mobs/passive/horse/llama.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB20.

Строки 5–13:

```rust
//! implemented; jumping, galloping and the chest inventory are not.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * jumping, galloping, the chest inventory and breeding stats are not
//!   implemented.

use std::sync::Weak;

use glam::DVec3;
```

## GAP040 — `steel-core/src/entity/entities/mobs/passive/horse/mule.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 5–13:

```rust
//! implemented; jumping, galloping and the chest inventory are not.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * jumping, galloping, the chest inventory and breeding stats are not
//!   implemented.

use std::sync::Weak;

use glam::DVec3;
```

## GAP041 — `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB22.

Строки 3–15:

```rust
//! Built on the shared `Animal` skeleton with the vanilla `AbstractHorse` flags
//! (`id_flags`) mirrored into the generated entity data. Horse-specific
//! mechanics are intentionally not wired yet.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the `AbstractHorse` behaviour (saddling, jumping, galloping, chest
//!   inventory, breeding stats) is not implemented; only the flag byte is kept.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
```

## GAP042 — `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB20.

Строки 5–14:

```rust
//! type it is registered for.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the trader's despawn rules and spit AI are not implemented; the llama is
//!   spawned and leashed by `WanderingTraderSpawner` like vanilla.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
```

## GAP043 — `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB22.

Строки 3–15:

```rust
//! Built on the shared `Animal` skeleton with the vanilla `AbstractHorse` flags
//! (`id_flags`) mirrored into the generated entity data. Horse-specific
//! mechanics are intentionally not wired yet.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the `AbstractHorse` behaviour (saddling, jumping, galloping, chest
//!   inventory, breeding stats) is not implemented; only the flag byte is kept.

use std::sync::Weak;

use glam::DVec3;
use simdnbt::{borrow, owned};
use steel_macros::entity_behavior;
```

## GAP044 — `steel-core/src/entity/entities/mobs/passive/panda.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB17.

Строки 4–14:

```rust
//! and look goals, plus the vanilla `checkAnimalSpawnRules` placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the species-specific goals (climbing, ramming, sliding, digging, ...) are
//!   not implemented yet; the shared animal goal set is used instead.
//! * food/tempt interaction uses the vanilla `#minecraft:goat_food`-style tags
//!   only where Steel already generates them; other species only breed through
//!   the generic `Animal::mob_interact_animal` path.

use std::sync::Weak;

```

## GAP045 — `steel-core/src/entity/entities/mobs/passive/sniffer.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB15.

Строки 4–14:

```rust
//! and look goals, plus the vanilla `checkAnimalSpawnRules` placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the species-specific goals (climbing, ramming, sliding, digging, ...) are
//!   not implemented yet; the shared animal goal set is used instead.
//! * food/tempt interaction uses the vanilla `#minecraft:goat_food`-style tags
//!   only where Steel already generates them; other species only breed through
//!   the generic `Animal::mob_interact_animal` path.

use std::sync::Weak;

```

## GAP046 — `steel-core/src/entity/entities/mobs/passive/strider.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 4–14:

```rust
//! and look goals, plus the vanilla `checkAnimalSpawnRules` placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the species-specific goals (climbing, ramming, sliding, digging, ...) are
//!   not implemented yet; the shared animal goal set is used instead.
//! * food/tempt interaction uses the vanilla `#minecraft:goat_food`-style tags
//!   only where Steel already generates them; other species only breed through
//!   the generic `Animal::mob_interact_animal` path.

use std::sync::Weak;

```

## GAP047 — `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB21.

Строки 4–12:

```rust
//! predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * species-specific goals (taming, sitting, leashing, cube fusion) are not
//!   implemented; the shared animal goal set is used instead.
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
```

## GAP048 — `steel-core/src/entity/entities/mobs/passive/turtle.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB18.

Строки 4–14:

```rust
//! and look goals, plus the vanilla `checkAnimalSpawnRules` placement predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the species-specific goals (climbing, ramming, sliding, digging, ...) are
//!   not implemented yet; the shared animal goal set is used instead.
//! * food/tempt interaction uses the vanilla `#minecraft:goat_food`-style tags
//!   only where Steel already generates them; other species only breed through
//!   the generic `Animal::mob_interact_animal` path.

use std::sync::Weak;

```

## GAP049 — `steel-core/src/entity/entities/mobs/passive/villager.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB03, MOB04.

Строки 1–9:

```rust
//! Vanilla `net.minecraft.world.entity.npc.villager.Villager`.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * trading covers the novice tier of every profession; levelling, restocking
//!   and gossip-driven prices are not modelled, so the offer list is fixed for
//!   the villager's lifetime.
//! * the `villager_data` synced component is stored as-is.
//! * no natural spawning: vanilla villages place villagers through structure
//!   generation and the wandering trader arrives with the trader event, both of
```

## GAP050 — `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB02, MOB10, MOB11, MOB12.

Строки 4–12:

```rust
//! llama escorts are spawned by `WanderingTraderSpawner` and leashed here.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the trade list is fixed at spawn from the wandering trader table.
//! * no natural spawning: vanilla villages place villagers through structure
//!   generation and the wandering trader arrives with the trader event, both of
//!   which Steel does not run yet.
use std::sync::{Arc, Weak};

```

## GAP051 — `steel-core/src/entity/entities/mobs/passive/wolf.rs`

Статус: **NEEDS AUDIT**. Сценарии: MOB13.

Строки 4–12:

```rust
//! predicate.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * species-specific goals (taming, sitting, leashing, cube fusion) are not
//!   implemented; the shared animal goal set is used instead.
use std::sync::Weak;

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
```

## GAP052 — `steel-core/src/entity/entities/objects/decoration/armor_stand.rs`

Статус: **NEEDS AUDIT**. Сценарии: NET05, COM04, CHK03.

Строки 7–15:

```rust
//! `minecraft:armor_stand` item itself, and never awards experience.
//!
//! Documented deviations from vanilla 26.3 (see `STATE-26.3-RU.md`):
//! * a right click only fills an *empty* slot. Vanilla replaces the item already
//!   in the target slot and hands it to the player; Steel's player inventory has
//!   no "place swapped item into hand" primitive yet, so replacing would delete
//!   the old item. Filling empty slots keeps the same gameplay without silent
//!   item loss;
//! * marker stands keep their vanilla behaviour for interaction (they are not
```

## GAP053 — `steel-core/src/entity/entities/objects/decoration/mannequin.rs`

Статус: **NEEDS AUDIT**. Сценарии: NET05, COM04, CHK03.

Строки 6–19:

```rust
//! optional description component rendered in the UI.
//!
//! Documented deviations from vanilla 26.3 (see `STATE-26.3-RU.md`):
//! * mannequins do not render and cannot be equipped through interaction yet;
//!   the equipment slots live in `LivingEntityBase` and are saved with the
//!   entity, but the interaction path is not implemented.
//! * a mannequin always keeps the default `HumanoidArm::Right` and the default
//!   profile; NBT round-trips of `profile` are not implemented because
//!   `ResolvableProfile` has no NBT codec in Steel yet.
//! * mannequins never take damage in vanilla; Steel keeps the base
//!   `LivingEntity` damage path, so they are destroyable like other decorations.

use std::sync::{Arc, Weak};

```

## GAP054 — `steel-core/src/entity/entities/objects/projectiles/shulker_bullet.rs`

Статус: **NEEDS AUDIT**. Сценарии: NET05, COM04, CHK03.

Строки 4–13:

```rust
//! damage on contact and applies levitation.
//!
//! Deviations (deliberate, documented in `STATE-26.3-RU.md`):
//! * the homing flight is not implemented; Steel's shared throwable tick flies
//!   straight and applies gravity.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
```

## GAP055 — `steel-core/src/entity/patrol_spawner.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 8–16:

```rust
//! the ominous banner.
//!
//! Deviations from vanilla, both documented in `STATE-26.3-RU.md`:
//!
//! * the "have monsters been peaceful long enough" check is the player's play
//!   time, because Steel has no per-world `inhabitedTime` yet and vanilla's
//!   answer for a fresh world is the same;
//! * the village check uses the `meeting` point of interest inside a 5×5×5
//!   section box, which is the block a vanilla village is anchored on.
```

## GAP056 — `steel-core/src/player/mod.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 1711–1719:

```rust
                    // reported instead of being silently dropped.
                    log::warn!(
                        "test instance block action {:?} at {:?} is not implemented",
                        packet.action,
                        packet.pos
                    );
                }
                TestInstanceBlockAction::Init
                | TestInstanceBlockAction::Query
```

## GAP057 — `steel-core/src/server/tests.rs`

Статус: **NEEDS AUDIT**. Сценарии: выберите профильный сценарий в MANUAL-MECHANICS-26.3-RU.md и заведите отдельный case для специального пути.

Строки 4660–4685:

```rust
        runtime.block_on(async {
            let save_root = test_storage_root("offline-key-request");
            let services_stub = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("services key stub should bind");
            let endpoint = format!(
                "http://{}/publickeys",
                services_stub
                    .local_addr()
                    .expect("services key stub should report its address")
            );

            let server = offline_server(runtime, &save_root, &endpoint)
                .await
                .expect("an offline server should start");

            timeout(SERVICE_KEY_CONNECT_TIMEOUT, services_stub.accept())
                .await
                .expect("an offline server should request the services keys")
                .expect("services key stub should accept the request");

            shutdown_server(&server, &save_root).await;
        });
    });
}

```

Полнота поиска не гарантируется: механика может быть пропущена вообще без комментария. Реестр объектов и собственные тесты не заменяют дифференциальную проверку с эталоном.
