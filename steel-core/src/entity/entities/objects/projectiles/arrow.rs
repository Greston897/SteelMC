//! Arrow entities (`Arrow` and `SpectralArrow`).
//!
//! Ports vanilla `AbstractArrow` / `Arrow` / `SpectralArrow` on top of the Steel
//! `Projectile` trait: gravity `0.05`, drag `0.99` in air / `0.6` in water,
//! sticking into blocks with a `life` counter of `1200` ticks, critical hits,
//! piercing, pickup rules and the vanilla hit sounds. Spectral arrows share the
//! whole implementation and additionally make the hit entity glow.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_protocol::packets::game::SoundSource;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::data_components::{PotionContents, vanilla_components};
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::{sound_events, vanilla_damage_types, vanilla_items};
use steel_utils::locks::SyncMutex;
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::nbt::NbtValueInput as _;
use steel_utils::{DowncastType, DowncastTypeKey};

use crate::behavior::MOB_EFFECT_BEHAVIORS;
use crate::entity::LivingEntity;
use crate::entity::damage::DamageSource;
use crate::entity::potion_contents::to_runtime_instance_icon_from_visibility;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, Player, Projectile, ProjectileBase, RemovalReason,
    SharedEntity,
};
use crate::world::{ClipHitResult, World};
use steel_utils::types::GameType;

/// Vanilla `AbstractArrow.MAX_LIFE_TICKS`: a stuck arrow despawns after a minute.
pub const MAX_LIFE_TICKS: i32 = 1200;

/// Vanilla `AbstractArrow.getGravity()`.
pub const ARROW_GRAVITY: f64 = 0.05;

/// Vanilla `AbstractArrow.DRAG_IN_AIR`.
pub const ARROW_DRAG_IN_AIR: f64 = 0.99;

/// Vanilla water drag for arrows.
pub const ARROW_DRAG_IN_WATER: f64 = 0.6;

/// Vanilla `AbstractArrow` default damage for regular arrows.
pub const DEFAULT_ARROW_DAMAGE: f64 = 2.0;

/// Vanilla spectral arrow damage.
pub const SPECTRAL_ARROW_DAMAGE: f64 = 4.0;

/// Vanilla `PickupPermission`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowPickup {
    /// Cannot be picked up at all.
    Disallowed,
    /// Survival players can pick the arrow up.
    Allowed,
    /// Only creative players can pick the arrow up.
    CreativeOnly,
}

impl ArrowPickup {
    /// Vanilla ordinals: `DISALLOWED`, `ALLOWED`, `CREATIVE_ONLY`.
    #[must_use]
    pub const fn value(self) -> i8 {
        match self {
            Self::Disallowed => 0,
            Self::Allowed => 1,
            Self::CreativeOnly => 2,
        }
    }

    /// Reads the vanilla `PickupPermission` ordinal.
    #[must_use]
    pub const fn from_value(value: i8) -> Self {
        match value {
            1 => Self::Allowed,
            2 => Self::CreativeOnly,
            _ => Self::Disallowed,
        }
    }
}

/// Mutable state of a single arrow.
#[derive(Debug, Clone)]
pub struct ArrowState {
    damage: f64,
    critical: bool,
    pierce_level: u8,
    in_ground: bool,
    life: i32,
    pickup: ArrowPickup,
    item: ItemStack,
    weapon: Option<ItemStack>,
    shot_from_crossbow: bool,
    pierced_entities: Vec<i32>,
}

impl ArrowState {
    /// A fresh arrow carrying `item`.
    #[must_use]
    pub const fn new(item: ItemStack, damage: f64) -> Self {
        Self {
            damage,
            critical: false,
            pierce_level: 0,
            in_ground: false,
            life: 0,
            pickup: ArrowPickup::Disallowed,
            item,
            weapon: None,
            shot_from_crossbow: false,
            pierced_entities: Vec::new(),
        }
    }
}

/// Shared arrow fields for both arrow entity types.
pub struct ArrowHandle {
    /// Mutable arrow state.
    pub state: SyncMutex<ArrowState>,
    /// Set once the arrow removed itself, guarding pickup races.
    pub removed: AtomicBool,
    /// Whether this is a spectral arrow (marks hit entities as glowing).
    pub spectral: bool,
}

impl ArrowHandle {
    /// Creates the handle for a freshly spawned arrow.
    #[must_use]
    pub const fn new(item: ItemStack, damage: f64, spectral: bool) -> Self {
        Self {
            state: SyncMutex::new(ArrowState::new(item, damage)),
            removed: AtomicBool::new(false),
            spectral,
        }
    }

    /// Whether this handle belongs to a spectral arrow.
    #[must_use]
    pub const fn spectral(&self) -> bool {
        self.spectral
    }
}

/// Shared vanilla `AbstractArrow` behaviour implemented by both arrow entities.
pub trait AbstractArrow: Projectile + Sized {
    /// Returns this arrow's shared state handle.
    fn arrow_handle(&self) -> &ArrowHandle;

    /// The item this arrow renders and drops.
    fn arrow_item(&self) -> ItemStack {
        self.arrow_handle().state.lock().item.clone()
    }

    /// Sets the rendered/dropped item.
    fn set_arrow_item(&self, item: ItemStack) {
        self.arrow_handle().state.lock().item = item;
    }

    /// Vanilla `AbstractArrow#getPickupItem`.
    fn pickup_item(&self) -> ItemStack {
        self.arrow_item().copy_with_count(1)
    }

    /// Base damage of this arrow.
    fn arrow_damage(&self) -> f64 {
        self.arrow_handle().state.lock().damage
    }

    /// Sets the base damage (vanilla `setDamage`).
    fn set_arrow_damage(&self, damage: f64) {
        self.arrow_handle().state.lock().damage = damage;
    }

    /// Whether this arrow is critical.
    fn is_critical(&self) -> bool {
        self.arrow_handle().state.lock().critical
    }

    /// Sets the critical flag (vanilla `setCritical`).
    fn set_critical(&self, critical: bool) {
        self.arrow_handle().state.lock().critical = critical;
    }

    /// Number of entities this arrow can pierce.
    fn pierce_level(&self) -> u8 {
        self.arrow_handle().state.lock().pierce_level
    }

    /// Sets the pierce level (vanilla `setPierceLevel`).
    fn set_pierce_level(&self, level: u8) {
        self.arrow_handle().state.lock().pierce_level = level;
    }

    /// Whether the arrow is stuck in a block.
    fn is_in_ground(&self) -> bool {
        self.arrow_handle().state.lock().in_ground
    }

    /// Sets the in-ground flag.
    fn set_in_ground(&self, in_ground: bool) {
        self.arrow_handle().state.lock().in_ground = in_ground;
    }

    /// Pickup permission of this arrow.
    fn pickup(&self) -> ArrowPickup {
        self.arrow_handle().state.lock().pickup
    }

    /// Sets the pickup permission (vanilla `setPickupPermission`).
    fn set_pickup(&self, pickup: ArrowPickup) {
        self.arrow_handle().state.lock().pickup = pickup;
    }

    /// Whether this arrow was shot from a crossbow.
    fn shot_from_crossbow(&self) -> bool {
        self.arrow_handle().state.lock().shot_from_crossbow
    }

    /// Marks this arrow as crossbow-shot.
    fn set_shot_from_crossbow(&self) {
        self.arrow_handle().state.lock().shot_from_crossbow = true;
    }

    /// The weapon stack that shot this arrow, if any.
    fn weapon(&self) -> Option<ItemStack> {
        self.arrow_handle().state.lock().weapon.clone()
    }

    /// Records the weapon that shot this arrow.
    fn set_weapon(&self, weapon: ItemStack) {
        self.arrow_handle().state.lock().weapon = Some(weapon);
    }

    /// Vanilla `AbstractArrow#age`: counts stuck ticks and despawns at the limit.
    fn age_arrow(&self) {
        let mut state = self.arrow_handle().state.lock();
        state.life += 1;
        let expired = state.life >= MAX_LIFE_TICKS;
        drop(state);
        if expired {
            self.set_removed(RemovalReason::Discarded);
        }
    }

    /// Vanilla `AbstractArrow#knockback`: pushes the hit living entity away.
    fn arrow_knockback(&self, living: &dyn LivingEntity) {
        let velocity = self.velocity();
        let horizontal = (velocity.x * velocity.x + velocity.z * velocity.z).sqrt();
        if horizontal < 1.0e-6 {
            return;
        }
        living.knockback(0.6, velocity.x / horizontal, velocity.z / horizontal);
    }

    /// Vanilla `AbstractArrow#tick`.
    fn abstract_arrow_tick(self: Arc<Self>) {
        self.set_old_position_to_current();
        self.base().set_old_rotation_to_current();

        // Stuck arrows only age out.
        if self.is_in_ground() {
            self.age_arrow();
            return;
        }

        // A block that appeared inside the arrow stops it (vanilla `isInsideBlock`).
        if let Some(world) = self.level() {
            let pos = self.block_position();
            if !world.get_block_state(pos).is_air() {
                self.set_velocity(DVec3::ZERO);
                self.set_in_ground(true);
                return;
            }
        }

        let hit = self.get_hit_result_on_move_vector();
        let new_position = match &hit {
            Some(result) => result.location(),
            None => self.position() + self.velocity(),
        };

        if let Err(error) = self.try_set_position(new_position) {
            log::debug!("failed to advance arrow {}: {error}", self.id());
            self.set_removed(RemovalReason::Discarded);
            return;
        }

        self.update_rotation();
        self.projectile_base_tick();

        // Vanilla `applyDrag` then `applyGravity`.
        let drag = if self.is_in_water() {
            ARROW_DRAG_IN_WATER
        } else {
            ARROW_DRAG_IN_AIR
        };
        self.set_velocity(self.velocity() * drag);
        if !self.is_in_ground() {
            self.apply_gravity();
        }

        if let Some(result) = hit
            && self.is_alive()
            && !self.is_world_change_pending()
        {
            self.hit_target_or_deflect_self(&result);
        }
    }

    /// Vanilla `AbstractArrow#onEntityHit`.
    fn abstract_arrow_on_hit_entity(self: Arc<Self>, entity: &SharedEntity) {
        let Some(world) = entity.level() else {
            return;
        };

        let speed = self.velocity().length() as f32;
        let mut raw_damage = ((f64::from(speed) * self.arrow_damage())
            .clamp(0.0, f64::from(i32::MAX)))
        .ceil() as i32;

        // Vanilla piercing bookkeeping: `pierced.size() >= pierceLevel + 1`).
        if self.pierce_level() > 0 {
            let mut state = self.arrow_handle().state.lock();
            let limit = usize::from(state.pierce_level) + 1;
            if state.pierced_entities.len() >= limit {
                drop(state);
                self.set_removed(RemovalReason::Discarded);
                return;
            }
            state.pierced_entities.push(entity.id());
        }

        if self.is_critical() {
            let bonus = rand::random_range(0..raw_damage / 2 + 2);
            raw_damage = raw_damage.saturating_add(bonus);
        }

        let mut damage = DamageSource::environment(&vanilla_damage_types::ARROW)
            .with_direct_entity(Arc::<Self>::clone(&self));
        if let Some(owner) = self.get_owner() {
            damage = damage.with_causing_entity(owner);
        }

        let hit_player = entity.as_player().is_some();
        if !entity.hurt(&world, &damage, raw_damage as f32) {
            return;
        }

        // Spectral arrows mark what they hit.
        if self.arrow_handle().spectral() {
            entity.base().set_glowing(true);
        }

        if let Some(living) = entity.as_living_entity()
            && self.pierce_level() == 0
        {
            self.arrow_knockback(living);
        }

        let sound = if hit_player {
            &sound_events::ENTITY_ARROW_HIT_PLAYER
        } else {
            &sound_events::ENTITY_ARROW_HIT
        };
        self.play_sound(sound, 1.0, 1.2);

        // Vanilla `Arrow#doPostHurtEffects`: tipped arrows apply their potion
        // contents with an eighth of the configured duration (minimum 1 tick).
        if !self.arrow_handle().spectral() {
            self.apply_tipped_arrow_effects(world.as_ref(), entity);
        }

        // Arrows without piercing stop at the entity they hit.
        if self.pierce_level() == 0 {
            self.set_velocity(self.velocity() * 0.0);
            self.set_removed(RemovalReason::Discarded);
        }
    }

    /// Vanilla `Arrow#doPostHurtEffects`: applies the tipped arrow's potion
    /// contents to the entity that was hit.
    ///
    /// Instantaneous effects are applied through their effect behavior;
    /// everything else gets `duration / 8` ticks with a minimum of one tick.
    fn apply_tipped_arrow_effects(self: &Arc<Self>, world: &World, entity: &SharedEntity) {
        let item = self.arrow_item();
        let contents =
            item.get_or_default(vanilla_components::POTION_CONTENTS, PotionContents::empty());
        let effects = contents.all_effects();
        if effects.is_empty() {
            return;
        }
        let Some(living) = entity.as_living_entity() else {
            return;
        };
        if !living.is_affected_by_potions() {
            return;
        }

        let owner = self.get_owner();
        let this: SharedEntity = Arc::<Self>::clone(self);
        for effect in &effects {
            let behavior = MOB_EFFECT_BEHAVIORS.get_behavior(effect.effect());
            if let Some(instantaneous) = behavior.as_instantaneous() {
                instantaneous.apply_instantaneous(
                    world,
                    living,
                    effect.amplifier(),
                    Some(&this),
                    owner.as_ref(),
                    1.0,
                );
                continue;
            }

            let duration = effect.map_duration(|base_duration| (base_duration / 8).max(1));
            let scaled_effect = to_runtime_instance_icon_from_visibility(effect, duration);
            living.add_mob_effect_with_source(scaled_effect, Some(this.as_ref()));
        }
    }

    /// Vanilla `AbstractArrow#onBlockHit`: the arrow sticks into the block.
    fn abstract_arrow_on_hit_block(self: Arc<Self>, hit: &ClipHitResult) {
        // Keep the block behavior dispatch (`super.onHitBlock`).
        self.projectile_on_hit_block(hit);

        self.set_in_ground(true);
        self.set_velocity(DVec3::ZERO);
        self.set_arrow_item(self.pickup_item());
        self.play_sound(&sound_events::ENTITY_ARROW_HIT, 1.0, 1.2);
    }

    /// Vanilla `AbstractArrow#onPlayerCollision` pickup handling.
    fn abstract_arrow_player_touch(self: Arc<Self>, player: &Arc<Player>) {
        if !self.is_in_ground() || self.is_removed() {
            return;
        }

        match self.pickup() {
            ArrowPickup::Disallowed => return,
            ArrowPickup::CreativeOnly if player.game_mode() != GameType::Creative => {
                return;
            }
            ArrowPickup::Allowed | ArrowPickup::CreativeOnly => {}
        }

        if self.arrow_handle().removed.swap(true, Ordering::AcqRel) {
            return;
        }

        let mut stack = self.pickup_item();
        player.add_item_with_sound(&mut stack);
        if stack.is_empty() {
            self.play_sound(&sound_events::ENTITY_ARROW_HIT_PLAYER, 0.2, 1.0);
        }
        self.set_removed(RemovalReason::Discarded);
    }

    /// Saves the vanilla arrow NBT fields.
    fn save_arrow(&self, nbt: &mut NbtCompound) {
        let state = self.arrow_handle().state.lock();
        nbt.insert("damage", state.damage);
        nbt.insert("crit", i8::from(state.critical));
        nbt.insert("PierceLevel", state.pierce_level as i8);
        nbt.insert("inGround", i8::from(state.in_ground));
        nbt.insert("life", state.life as i16);
        nbt.insert("pickup", state.pickup.value());
        nbt.insert("ShotFromCrossbow", i8::from(state.shot_from_crossbow));
        if !state.item.is_empty() {
            nbt.insert("item", state.item.to_nbt_tag_ref());
        }
        if let Some(weapon) = &state.weapon {
            nbt.insert("weapon", weapon.to_nbt_tag_ref());
        }
    }

    /// Loads the vanilla arrow NBT fields.
    fn load_arrow(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        let mut state = self.arrow_handle().state.lock();
        if let Some(damage) = nbt.get("damage").and_then(|tag| tag.codec_f64()) {
            state.damage = damage;
        }
        if let Some(crit) = nbt.byte_value("crit") {
            state.critical = crit != 0;
        }
        if let Some(pierce) = nbt.byte_value("PierceLevel") {
            state.pierce_level = pierce.max(0) as u8;
        }
        if let Some(in_ground) = nbt.byte_value("inGround") {
            state.in_ground = in_ground != 0;
        }
        if let Some(life) = nbt.short_value("life") {
            state.life = i32::from(life);
        }
        if let Some(pickup) = nbt.byte_value("pickup") {
            state.pickup = ArrowPickup::from_value(pickup);
        }
        if let Some(crossbow) = nbt.byte_value("ShotFromCrossbow") {
            state.shot_from_crossbow = crossbow != 0;
        }
        if let Some(item) = nbt.compound("item")
            && let Some(stack) = ItemStack::from_borrowed_compound(&item)
        {
            state.item = stack;
        }
        if let Some(weapon) = nbt.compound("weapon")
            && let Some(stack) = ItemStack::from_borrowed_compound(&weapon)
        {
            state.weapon = Some(stack);
        }
    }
}

/// Boilerplate shared by both arrow entity types.
macro_rules! arrow_entity_impl {
    ($name:ident) => {
        impl Entity for $name {
            fn base(&self) -> &EntityBase {
                &self.base
            }

            fn entity_type(&self) -> EntityTypeRef {
                self.entity_type
            }

            fn tick(self: Arc<Self>) {
                self.abstract_arrow_tick();
            }

            fn get_default_gravity(&self) -> f64 {
                ARROW_GRAVITY
            }

            fn sound_source(&self) -> SoundSource {
                SoundSource::Neutral
            }

            fn spawn_data(&self) -> i32 {
                self.get_owner().map_or(0, |owner| owner.id())
            }

            fn restore_owner_reference(&self, owner: &SharedEntity) {
                self.cache_owner_entity(owner);
            }

            fn projectile_owner_uuid(&self) -> Option<uuid::Uuid> {
                self.owner_uuid()
            }

            fn projectile_owner(&self) -> Option<SharedEntity> {
                self.get_owner()
            }

            fn player_touch(self: Arc<Self>, player: &Arc<Player>) {
                self.abstract_arrow_player_touch(player);
            }

            fn save_additional(&self, nbt: &mut NbtCompound) {
                self.save_projectile(nbt);
                self.save_arrow(nbt);
            }

            fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
                self.load_projectile(nbt);
                self.load_arrow(nbt);
            }
        }

        impl Projectile for $name {
            fn projectile_base(&self) -> &ProjectileBase {
                &self.projectile_base
            }

            fn on_hit_entity(self: Arc<Self>, entity: &SharedEntity, _location: DVec3) {
                self.abstract_arrow_on_hit_entity(entity);
            }

            fn on_hit_block(self: Arc<Self>, hit: &ClipHitResult) {
                self.abstract_arrow_on_hit_block(hit);
            }
        }

        impl AbstractArrow for $name {
            fn arrow_handle(&self) -> &ArrowHandle {
                &self.arrow
            }
        }
    };
}

/// A regular arrow (`Arrow`), also used for tipped arrows.
#[entity_behavior(class = "Arrow")]
pub struct ArrowEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
    arrow: ArrowHandle,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ArrowEntity`.
unsafe impl DowncastType for ArrowEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/arrow");
}

impl ArrowEntity {
    /// Creates a fresh arrow.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
            arrow: ArrowHandle::new(
                ItemStack::new(&vanilla_items::ARROW),
                DEFAULT_ARROW_DAMAGE,
                false,
            ),
        }
    }

    /// Creates an arrow from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
            arrow: ArrowHandle::new(
                ItemStack::new(&vanilla_items::ARROW),
                DEFAULT_ARROW_DAMAGE,
                false,
            ),
        }
    }
}

arrow_entity_impl!(ArrowEntity);

/// A spectral arrow (`SpectralArrow`): glows the entity it hits.
#[entity_behavior(class = "SpectralArrow")]
pub struct SpectralArrowEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
    arrow: ArrowHandle,
}

// SAFETY: This key is owned by Steel and uniquely identifies `SpectralArrowEntity`.
unsafe impl DowncastType for SpectralArrowEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/spectral_arrow");
}

impl SpectralArrowEntity {
    /// Creates a fresh spectral arrow.
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
            arrow: ArrowHandle::new(
                ItemStack::new(&vanilla_items::SPECTRAL_ARROW),
                SPECTRAL_ARROW_DAMAGE,
                true,
            ),
        }
    }

    /// Creates a spectral arrow from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
            arrow: ArrowHandle::new(
                ItemStack::new(&vanilla_items::SPECTRAL_ARROW),
                SPECTRAL_ARROW_DAMAGE,
                true,
            ),
        }
    }
}

arrow_entity_impl!(SpectralArrowEntity);

/// Vanilla `ThrownTrident` base damage.
pub const TRIDENT_DAMAGE: f64 = 8.0;

/// Ticks between loyalty return steps (vanilla moves the trident every tick;
/// Steel steps it a fixed distance to stay deterministic and testable).
pub const LOYALTY_RETURN_DISTANCE: f64 = 1.0;

/// A thrown trident (`ThrownTrident`).
///
/// Modelled on vanilla's `ThrownTrident extends AbstractArrow`, so it shares the
/// arrow flight/pickup machinery and adds:
/// * 8.0 base damage,
/// * a persisted `Loyalty` level that makes the trident fly back to its owner
///   once it has landed or hit something (`DealtDamage`),
/// * pickup allowed for survival players, creative-only when thrown in creative.
///
/// Differences from vanilla, all deliberate:
/// * Vanilla uses the `riptide` enchantment for the spin-attack throw; Steel has
///   no riptide movement yet, so a charged throw always throws the trident.
/// * The return trip moves the trident a fixed distance per tick instead of
///   vanilla's per-tick teleport-with-particles.
#[entity_behavior(class = "ThrownTrident")]
pub struct ThrownTridentEntity {
    base: EntityBase,
    entity_type: EntityTypeRef,
    projectile_base: ProjectileBase,
    arrow: ArrowHandle,
    /// Persisted `Loyalty` enchantment level.
    loyalty: SyncMutex<i32>,
    /// Persisted vanilla `DealtDamage` flag; armed loyalty only returns after it.
    dealt_damage: AtomicBool,
}

// SAFETY: This key is owned by Steel and uniquely identifies `ThrownTridentEntity`.
unsafe impl DowncastType for ThrownTridentEntity {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:entity/thrown_trident");
}

impl ThrownTridentEntity {
    /// Creates a bare trident (the shape the entity loader builds).
    #[must_use]
    pub fn new(entity_type: EntityTypeRef, id: i32, position: DVec3, world: Weak<World>) -> Self {
        Self {
            base: EntityBase::new(id, position, entity_type.dimensions, world),
            entity_type,
            projectile_base: ProjectileBase::new(),
            arrow: ArrowHandle::new(
                ItemStack::new(&vanilla_items::TRIDENT),
                TRIDENT_DAMAGE,
                false,
            ),
            loyalty: SyncMutex::new(0),
            dealt_damage: AtomicBool::new(false),
        }
    }

    /// Creates a trident ready to be thrown, carrying `item` and its `Loyalty`.
    #[must_use]
    pub fn with_throw(
        entity_type: EntityTypeRef,
        id: i32,
        position: DVec3,
        world: Weak<World>,
        item: ItemStack,
        loyalty: i32,
    ) -> Self {
        let entity = Self::new(entity_type, id, position, world);
        entity.set_arrow_item(item);
        *entity.loyalty.lock() = loyalty;
        entity
    }

    /// Creates a trident from saved base data.
    #[must_use]
    pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
        Self {
            base: EntityBase::from_load(load, entity_type.dimensions),
            entity_type,
            projectile_base: ProjectileBase::new(),
            arrow: ArrowHandle::new(
                ItemStack::new(&vanilla_items::TRIDENT),
                TRIDENT_DAMAGE,
                false,
            ),
            loyalty: SyncMutex::new(0),
            dealt_damage: AtomicBool::new(false),
        }
    }

    /// Current `Loyalty` level.
    #[must_use]
    pub fn loyalty(&self) -> i32 {
        *self.loyalty.lock()
    }

    /// Whether the trident already hurt or hit something (`DealtDamage`).
    #[must_use]
    pub fn dealt_damage(&self) -> bool {
        self.dealt_damage.load(Ordering::Acquire)
    }

    /// Marks the trident as having hit something, arming loyalty.
    pub fn mark_dealt_damage(&self) {
        self.dealt_damage.store(true, Ordering::Release);
    }

    /// Vanilla `ThrownTrident.tick` return leg: fly back to the owner once the
    /// trident landed (or is mid-return) and loyalty is present.
    ///
    /// Returns `true` when the trident is being returned (so the regular arrow
    /// tick must be skipped), `false` to run the normal flight tick.
    fn tick_loyalty(self: &Arc<Self>) -> bool {
        if self.loyalty() <= 0 || !self.dealt_damage() {
            return false;
        }

        let Some(world) = self.level() else {
            return false;
        };
        let Some(owner) = self.get_owner() else {
            return false;
        };

        // Vanilla ignores collisions while returning.
        self.base.set_no_physics(true);

        let target =
            owner.position() + DVec3::new(0.0, owner.get_eye_y() - owner.position().y, 0.0);
        let delta = target - self.position();
        if delta.length_squared() <= 1.0 {
            // Vanilla `tryReturnToOwner`: insert the trident into the owner's
            // inventory (or drop it) and discard the entity.
            let mut stack = self.pickup_item();
            if let Some(player) = owner.as_player() {
                player.add_item_with_sound(&mut stack);
            } else {
                world.drop_item_stack(self.block_position(), stack);
            }
            self.set_removed(RemovalReason::Discarded);
            return true;
        }

        let step = delta.normalize_or_zero() * LOYALTY_RETURN_DISTANCE;
        if let Err(error) = self.try_set_position(self.position() + step) {
            log::debug!("failed to move returning trident: {error}");
        }
        self.play_sound(&sound_events::ITEM_TRIDENT_RETURN, 1.0, 1.0);
        true
    }

    /// Saves the trident-only NBT fields on top of the shared arrow state.
    fn save_trident(&self, nbt: &mut NbtCompound) {
        nbt.insert("Loyalty", *self.loyalty.lock() as i8);
        nbt.insert("DealtDamage", i8::from(self.dealt_damage()));
    }

    /// Loads the trident-only NBT fields.
    fn load_trident(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        *self.loyalty.lock() = i32::from(nbt.byte_value("Loyalty").unwrap_or(0));
        self.dealt_damage.store(
            nbt.byte_value("DealtDamage")
                .is_some_and(|value| value != 0),
            Ordering::Release,
        );
    }
}

impl Entity for ThrownTridentEntity {
    fn base(&self) -> &EntityBase {
        &self.base
    }

    fn entity_type(&self) -> EntityTypeRef {
        self.entity_type
    }

    fn tick(self: Arc<Self>) {
        if self.tick_loyalty() {
            return;
        }
        self.abstract_arrow_tick();
    }

    fn get_default_gravity(&self) -> f64 {
        ARROW_GRAVITY
    }

    fn sound_source(&self) -> SoundSource {
        SoundSource::Players
    }

    fn spawn_data(&self) -> i32 {
        self.get_owner().map_or(0, |owner| owner.id())
    }

    fn restore_owner_reference(&self, owner: &SharedEntity) {
        self.cache_owner_entity(owner);
    }

    fn projectile_owner_uuid(&self) -> Option<uuid::Uuid> {
        self.owner_uuid()
    }

    fn projectile_owner(&self) -> Option<SharedEntity> {
        self.get_owner()
    }

    fn player_touch(self: Arc<Self>, player: &Arc<Player>) {
        self.abstract_arrow_player_touch(player);
    }

    fn save_additional(&self, nbt: &mut NbtCompound) {
        self.save_projectile(nbt);
        self.save_arrow(nbt);
        self.save_trident(nbt);
    }

    fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
        self.load_projectile(nbt);
        self.load_arrow(nbt);
        self.load_trident(nbt);
    }
}

impl Projectile for ThrownTridentEntity {
    fn projectile_base(&self) -> &ProjectileBase {
        &self.projectile_base
    }

    fn on_hit_entity(self: Arc<Self>, entity: &SharedEntity, _location: DVec3) {
        // `abstract_arrow_on_hit_entity` consumes the handle, so keep one for
        // the loyalty bookkeeping afterwards.
        let handle = Arc::clone(&self);
        handle.abstract_arrow_on_hit_entity(entity);
        self.mark_dealt_damage();
    }

    fn on_hit_block(self: Arc<Self>, hit: &ClipHitResult) {
        self.abstract_arrow_on_hit_block(hit);
    }
}

impl AbstractArrow for ThrownTridentEntity {
    fn arrow_handle(&self) -> &ArrowHandle {
        &self.arrow
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Weak;

    use glam::DVec3;
    use simdnbt::borrow::{NbtCompound as BorrowedNbtCompoundView, read_compound};
    use simdnbt::owned::NbtCompound;
    use std::io::Cursor;
    use steel_registry::{init_vanilla_registry, vanilla_entities};

    use super::{AbstractArrow, ArrowEntity, ArrowPickup, MAX_LIFE_TICKS, SpectralArrowEntity};
    use crate::entity::Entity as _;
    use crate::world::ClipHitResult;
    use crate::world::World;
    use std::sync::Arc;

    fn arrow(id: i32) -> ArrowEntity {
        ArrowEntity::new(
            &vanilla_entities::ARROW,
            id,
            DVec3::new(0.5, 64.0, 0.5),
            Weak::<World>::new(),
        )
    }

    fn spectral(id: i32) -> SpectralArrowEntity {
        SpectralArrowEntity::new(
            &vanilla_entities::SPECTRAL_ARROW,
            id,
            DVec3::new(0.5, 64.0, 0.5),
            Weak::<World>::new(),
        )
    }

    #[test]
    fn pickup_permissions_round_trip_through_their_vanilla_ordinals() {
        for pickup in [
            ArrowPickup::Disallowed,
            ArrowPickup::Allowed,
            ArrowPickup::CreativeOnly,
        ] {
            assert_eq!(ArrowPickup::from_value(pickup.value()), pickup);
        }
    }

    #[test]
    fn stuck_arrows_despawn_after_the_vanilla_life_time() {
        init_vanilla_registry();
        let arrow = arrow(1);
        arrow.set_in_ground(true);

        for _ in 0..MAX_LIFE_TICKS - 1 {
            arrow.age_arrow();
            assert!(
                !arrow.is_removed(),
                "arrows live for {MAX_LIFE_TICKS} ticks"
            );
        }

        arrow.age_arrow();
        assert!(arrow.is_removed());
    }

    #[test]
    fn hitting_a_block_sticks_the_arrow() {
        init_vanilla_registry();
        let arrow = Arc::new(arrow(2));
        arrow.set_velocity(DVec3::new(0.0, -1.0, 0.0));

        let hit = ClipHitResult {
            location: DVec3::new(0.5, 64.0, 0.5),
            direction: steel_utils::Direction::Up,
            block_pos: steel_utils::BlockPos::new(0, 63, 0),
            miss: false,
            inside: false,
            world_border_hit: false,
        };
        Arc::clone(&arrow).abstract_arrow_on_hit_block(&hit);

        assert!(arrow.is_in_ground());
        assert_eq!(arrow.velocity(), DVec3::ZERO);
        assert_eq!(arrow.pickup_item().count(), 1);
    }

    #[test]
    fn spectral_arrows_are_flagged_and_regular_arrows_are_not() {
        init_vanilla_registry();
        assert!(spectral(3).arrow_handle().spectral());
        assert!(!arrow(4).arrow_handle().spectral());
        assert_eq!(spectral(5).arrow_damage(), 4.0);
        assert_eq!(arrow(6).arrow_damage(), 2.0);
    }

    #[test]
    fn pierce_level_survives_saving_and_loading() {
        init_vanilla_registry();
        let shot = arrow(7);
        shot.set_pierce_level(3);
        shot.set_critical(true);
        shot.set_in_ground(true);
        shot.set_pickup(ArrowPickup::Allowed);
        shot.set_shot_from_crossbow();

        let mut nbt = NbtCompound::new();
        shot.save_arrow(&mut nbt);
        let mut bytes = Vec::new();
        nbt.write(&mut bytes);
        let mut cursor = Cursor::new(bytes.as_slice());
        let borrowed = read_compound(&mut cursor).expect("test NBT should reborrow");
        let view: BorrowedNbtCompoundView<'_, '_> = (&borrowed).into();

        let fresh = arrow(8);
        fresh.load_arrow(view);

        assert_eq!(fresh.pierce_level(), 3);
        assert!(fresh.is_critical());
        assert!(fresh.is_in_ground());
        assert_eq!(fresh.pickup(), ArrowPickup::Allowed);
        assert!(fresh.shot_from_crossbow());
    }
}
