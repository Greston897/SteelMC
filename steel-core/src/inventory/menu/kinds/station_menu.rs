//! Vanilla crafting station menus: loom, cartography table, smithing table,
//! and stonecutter.
//!
//! Each menu keeps its input slots in a [`SimpleContainer`] drained back to the
//! player on close and drives a virtual [`ResultContainer`] through a dedicated
//! [`ResultHandler`] that consumes ingredients, plays the station sound, and
//! recomputes the result slot (including multi-stack shift-click takes).

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use steel_protocol::packets::game::SoundSource;
use steel_registry::banner_pattern::BannerPatternRef;
use steel_registry::data_components::components::{
    ArmorTrim, BannerPatternLayer, BannerPatternLayers, MapPostProcessing,
};
use steel_registry::data_components::vanilla_components::{
    BANNER_PATTERNS, DYE, MAP_ID, MAP_POST_PROCESSING, PROVIDES_BANNER_PATTERNS,
    PROVIDES_TRIM_MATERIAL, TRIM,
};
use steel_registry::item_stack::ItemStack;
use steel_registry::items::ItemRef;
use steel_registry::recipe::{
    SmithingRecipe, SmithingRecipeInput, StonecuttingRecipe, vanilla_recipe_types,
};
use steel_registry::vanilla_banner_pattern_tags::BannerPatternTag;
use steel_registry::vanilla_item_tags::ItemTag;
use steel_registry::{
    REGISTRY, RegistryHolder, RegistryHolderSet, TaggedRegistryExt as _, level_events,
    sound_events, vanilla_blocks, vanilla_items, vanilla_menu_types,
};
use steel_utils::locks::{IntoShared, Shared, SyncMutex};
use steel_utils::{BlockPos, DowncastType, DowncastTypeKey};

use crate::inventory::container::{ResultContainer, SimpleContainer};
use crate::inventory::prelude::*;
use crate::inventory::slots::ResultHandler;
use crate::player::player_inventory::PlayerInventory;
use crate::world::World;
use crate::world::map_state::MAX_SCALE;

/// Vanilla `LoomMenu` maximum survival pattern count.
const MAX_LOOM_BANNER_LAYERS: usize = 6;

/// Shared marker kept for callers referring to generic station menus.
pub struct StationKind {
    /// Input container backing the menu's own slots.
    pub input: Shared<SimpleContainer>,
}

impl StationKind {
    /// Creates the kind for a given input container.
    #[must_use]
    pub const fn new(input: Shared<SimpleContainer>) -> Self {
        Self { input }
    }
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for StationKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/station");
}

impl MenuKind for StationKind {}

// ---------------------------------------------------------------------------
// Stonecutter (`Vanilla StonecutterMenu`)
// ---------------------------------------------------------------------------

struct StonecutterResultHandler {
    input: Shared<SimpleContainer>,
    result: Shared<ResultContainer>,
    selected: Arc<AtomicI32>,
    pos: BlockPos,
    world: Arc<World>,
}

impl StonecutterResultHandler {
    fn matching_recipes(
        input: &ItemStack,
    ) -> Vec<(&'static StonecuttingRecipe, steel_utils::Identifier)> {
        if input.is_empty() {
            return Vec::new();
        }
        // Keep the exact iteration order sent in `CUpdateRecipes.stonecutter` so
        // client button indices map 1:1 to server recipes.
        let mut out = Vec::new();
        for recipe in REGISTRY.recipes.iter() {
            if recipe.recipe_type().key.path.as_ref() != "stonecutting" {
                continue;
            }
            if let Some(data) = recipe.downcast_data::<StonecuttingRecipe>()
                && data.ingredient.test(input)
            {
                out.push((data, recipe.key().clone()));
            }
        }
        out
    }

    fn result_for(
        input: &ItemStack,
        selected: i32,
    ) -> Option<(ItemStack, steel_utils::Identifier)> {
        let index = usize::try_from(selected).ok()?;
        let matches = Self::matching_recipes(input);
        let (recipe, key) = matches.get(index)?;
        let stack = recipe.result.create();
        (!stack.is_empty()).then(|| (stack, key.clone()))
    }

    fn refresh(&self, guard: &mut ContainerLockGuard) {
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return;
        };
        let selected = self.selected.load(Ordering::Relaxed);
        let next = Self::result_for(input_c.get_item(0), selected)
            .map_or_else(ItemStack::empty, |(stack, _)| stack);
        result_c.set_item(0, next);
    }
}

impl ResultHandler for StonecutterResultHandler {
    fn result_container(&self) -> ContainerRef {
        ContainerRef::from(Arc::clone(&self.result))
    }

    fn dependencies(&self) -> Vec<ContainerRef> {
        vec![ContainerRef::from(Arc::clone(&self.input))]
    }

    fn update_result(&self, guard: &mut ContainerLockGuard) {
        self.refresh(guard);
    }

    fn on_result_taken(
        &self,
        guard: &mut ContainerLockGuard,
        player: &Player,
    ) -> Option<ItemStack> {
        let selected = self.selected.load(Ordering::Relaxed);
        if let Some(input_c) = guard.get(ContainerId::from_arc(&self.input))
            && let Some((_, recipe_key)) = Self::result_for(input_c.get_item(0), selected)
        {
            let _ = player.recipe_book.lock().add(&recipe_key);
        }

        if let Some(input_c) = guard.get_mut(ContainerId::from_arc(&self.input)) {
            let _ = input_c.remove_item(0, 1);
            if input_c.get_item(0).is_empty() {
                self.selected.store(-1, Ordering::Relaxed);
            }
        }

        // Refill the output when items remain so shift-clicking cuts the whole stack.
        self.refresh(guard);

        self.world.play_sound(
            &sound_events::UI_STONECUTTER_TAKE_RESULT,
            SoundSource::Blocks,
            self.pos,
            1.0,
            1.0,
            None,
        );
        None
    }

    fn is_result_valid(&self, guard: &ContainerLockGuard, _player: &Player) -> bool {
        let Some(input_c) = guard.get(ContainerId::from_arc(&self.input)) else {
            return false;
        };
        let selected = self.selected.load(Ordering::Relaxed);
        Self::result_for(input_c.get_item(0), selected).is_some()
    }
}

/// Stonecutter menu state (`StonecutterMenu`).
pub struct StonecutterKind {
    input: Shared<SimpleContainer>,
    result: Shared<ResultContainer>,
    selected: Arc<AtomicI32>,
    selected_slot: DataSlot,
    last_item: Option<ItemRef>,
    pos: BlockPos,
    world: Arc<World>,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for StonecutterKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/stonecutter");
}

impl MenuKind for StonecutterKind {
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        still_valid_block(&self.world, self.pos, &vanilla_blocks::STONECUTTER, player)
    }

    fn removed(&mut self, _behavior: &mut MenuBehavior, _player: &Player) {
        self.result.lock().set_item(0, ItemStack::empty());
    }

    fn slots_changed(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return;
        };
        let input = input_c.get_item(0);
        let current_item = (!input.is_empty()).then(|| input.item());
        if current_item == self.last_item {
            let selected = self.selected.load(Ordering::Relaxed);
            self.selected_slot
                .set(behavior, i16::try_from(selected).unwrap_or(-1));
            let next = StonecutterResultHandler::result_for(input, selected)
                .map_or_else(ItemStack::empty, |(stack, _)| stack);
            result_c.set_item(0, next);
        } else {
            // Vanilla `StonecutterMenu.slotsChanged`: only reset selection when
            // the item type changes, keeping selection while cutting a stack.
            self.last_item = current_item;
            self.selected.store(-1, Ordering::Relaxed);
            self.selected_slot.set(behavior, -1);
            result_c.set_item(0, ItemStack::empty());
        }
    }

    fn click_menu_button(
        &mut self,
        behavior: &mut MenuBehavior,
        button_id: i32,
        _player: &Player,
    ) -> bool {
        let current = self.selected.load(Ordering::Relaxed);
        if button_id == current || button_id < 0 {
            return false;
        }
        let mut guard = ContainerLockGuard::lock_all(&[
            ContainerRef::from(&self.input),
            ContainerRef::from(&self.result),
        ]);
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return false;
        };
        if let Some((stack, _)) =
            StonecutterResultHandler::result_for(input_c.get_item(0), button_id)
        {
            self.selected.store(button_id, Ordering::Relaxed);
            self.selected_slot
                .set(behavior, i16::try_from(button_id).unwrap_or(-1));
            result_c.set_item(0, stack);
        }
        true
    }
}

/// Builds the vanilla stonecutter menu (input, result, player inventory).
#[must_use]
pub fn stonecutter(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    pos: BlockPos,
    world: &Arc<World>,
) -> Menu {
    let input = SimpleContainer::new(1).into_shared();
    let result = ResultContainer::new().into_shared();
    let selected = Arc::new(AtomicI32::new(-1));

    let mut builder = MenuBuilder::new(&vanilla_menu_types::STONECUTTER, container_id);
    let selected_slot = builder.data_slot(-1);
    let input_section = builder.section_all(&input);
    let result_section = builder.crafting_result_slot(StonecutterResultHandler {
        input: Arc::clone(&input),
        result: Arc::clone(&result),
        selected: Arc::clone(&selected),
        pos,
        world: Arc::clone(world),
    });
    let player = builder.player_inventory(&inventory);

    builder.route_with_remainder_policy(
        result_section,
        player.all(),
        FillDirection::Backward,
        FakeResultRemainderPolicy::Discard,
    );
    builder.route(input_section, player.all(), FillDirection::Forward);
    builder.route(player.all(), input_section, FillDirection::Forward);
    builder.drain(input_section);

    builder.build(StonecutterKind {
        input,
        result,
        selected,
        selected_slot,
        last_item: None,
        pos,
        world: Arc::clone(world),
    })
}

// ---------------------------------------------------------------------------
// Smithing Table (`Vanilla SmithingMenu`)
// ---------------------------------------------------------------------------

fn assemble_smithing(
    template: &ItemStack,
    base: &ItemStack,
    addition: &ItemStack,
) -> Option<(ItemStack, steel_utils::Identifier)> {
    let recipe_input = SmithingRecipeInput::new(template.clone(), base.clone(), addition.clone());
    let recipe = REGISTRY
        .recipes
        .matching(&vanilla_recipe_types::SMITHING, &recipe_input)
        .next()?;

    let result = match recipe.data() {
        SmithingRecipe::Transform(transform) => transform
            .result
            .apply(transform.result.count(), base.components_patch()),
        SmithingRecipe::Trim(trim) => {
            let material = addition.get(PROVIDES_TRIM_MATERIAL)?.material().clone();
            let pattern = RegistryHolder::reference(trim.pattern);
            let new_trim = ArmorTrim::new(material, pattern);
            if base.get(TRIM) == Some(&new_trim) {
                return None;
            }
            let mut trimmed = base.copy_with_count(1);
            trimmed.set(TRIM, new_trim);
            trimmed
        }
    };

    (!result.is_empty()).then(|| (result, recipe.key().clone()))
}

struct SmithingResultHandler {
    input: Shared<SimpleContainer>,
    result: Shared<ResultContainer>,
    last_recipe: Arc<SyncMutex<Option<steel_utils::Identifier>>>,
    pos: BlockPos,
    world: Arc<World>,
}

impl SmithingResultHandler {
    fn refresh(&self, guard: &mut ContainerLockGuard) {
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return;
        };
        let assembled = assemble_smithing(
            input_c.get_item(0),
            input_c.get_item(1),
            input_c.get_item(2),
        );
        if let Some((stack, key)) = assembled {
            *self.last_recipe.lock() = Some(key);
            result_c.set_item(0, stack);
        } else {
            *self.last_recipe.lock() = None;
            result_c.set_item(0, ItemStack::empty());
        }
    }
}

impl ResultHandler for SmithingResultHandler {
    fn result_container(&self) -> ContainerRef {
        ContainerRef::from(Arc::clone(&self.result))
    }

    fn dependencies(&self) -> Vec<ContainerRef> {
        vec![ContainerRef::from(Arc::clone(&self.input))]
    }

    fn update_result(&self, guard: &mut ContainerLockGuard) {
        self.refresh(guard);
    }

    fn on_result_taken(
        &self,
        guard: &mut ContainerLockGuard,
        player: &Player,
    ) -> Option<ItemStack> {
        if let Some(key) = self.last_recipe.lock().take() {
            let _ = player.recipe_book.lock().add(&key);
        }
        if let Some(input_c) = guard.get_mut(ContainerId::from_arc(&self.input)) {
            let _ = input_c.remove_item(0, 1);
            let _ = input_c.remove_item(1, 1);
            let _ = input_c.remove_item(2, 1);
        }
        self.refresh(guard);
        self.world
            .level_event(level_events::SOUND_SMITHING_TABLE_USED, self.pos, 0, None);
        None
    }

    fn is_result_valid(&self, guard: &ContainerLockGuard, _player: &Player) -> bool {
        let Some(input_c) = guard.get(ContainerId::from_arc(&self.input)) else {
            return false;
        };
        assemble_smithing(
            input_c.get_item(0),
            input_c.get_item(1),
            input_c.get_item(2),
        )
        .is_some()
    }
}

/// Smithing menu state (`SmithingMenu`).
pub struct SmithingKind {
    handler: Arc<SmithingResultHandler>,
    result: Shared<ResultContainer>,
    pos: BlockPos,
    world: Arc<World>,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for SmithingKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/smithing");
}

impl MenuKind for SmithingKind {
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        still_valid_block(
            &self.world,
            self.pos,
            &vanilla_blocks::SMITHING_TABLE,
            player,
        )
    }

    fn removed(&mut self, _behavior: &mut MenuBehavior, _player: &Player) {
        self.result.lock().set_item(0, ItemStack::empty());
    }

    fn slots_changed(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        self.handler.refresh(guard);
    }
}

/// Builds the vanilla smithing table menu (template, base, addition, result).
#[must_use]
pub fn smithing(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    pos: BlockPos,
    world: &Arc<World>,
) -> Menu {
    let input = SimpleContainer::new(3).into_shared();
    let result = ResultContainer::new().into_shared();
    let handler = Arc::new(SmithingResultHandler {
        input: Arc::clone(&input),
        result: Arc::clone(&result),
        last_recipe: Arc::new(SyncMutex::new(None)),
        pos,
        world: Arc::clone(world),
    });

    let mut builder = MenuBuilder::new(&vanilla_menu_types::SMITHING, container_id);
    let input_section = builder.section_all(&input);
    let result_section = builder.crafting_result_slot(Arc::clone(&handler));
    let player = builder.player_inventory(&inventory);

    builder.route_with_remainder_policy(
        result_section,
        player.all(),
        FillDirection::Backward,
        FakeResultRemainderPolicy::Discard,
    );
    builder.route(input_section, player.all(), FillDirection::Forward);
    builder.route(player.all(), input_section, FillDirection::Forward);
    builder.drain(input_section);

    builder.build(SmithingKind {
        handler,
        result,
        pos,
        world: Arc::clone(world),
    })
}

// ---------------------------------------------------------------------------
// Cartography Table (`Vanilla CartographyTableMenu`)
// ---------------------------------------------------------------------------

fn assemble_cartography(map_stack: &ItemStack, additional: &ItemStack, world: &World) -> ItemStack {
    if map_stack.is_empty() || additional.is_empty() {
        return ItemStack::empty();
    }
    let Some(map_id) = map_stack.get(MAP_ID).map(|id| id.id()) else {
        return ItemStack::empty();
    };
    let (locked, scale) = {
        let maps = world.map_states.lock();
        let Some(state) = maps.get(map_id) else {
            return ItemStack::empty();
        };
        (state.locked(), state.scale())
    };

    if additional.is(&vanilla_items::PAPER) && !locked && scale < MAX_SCALE {
        let mut out = map_stack.copy_with_count(1);
        out.set(MAP_POST_PROCESSING, MapPostProcessing::Scale);
        out
    } else if additional.is(&vanilla_items::GLASS_PANE) && !locked {
        let mut out = map_stack.copy_with_count(1);
        out.set(MAP_POST_PROCESSING, MapPostProcessing::Lock);
        out
    } else if additional.is(&vanilla_items::MAP) {
        map_stack.copy_with_count(2)
    } else {
        ItemStack::empty()
    }
}

struct CartographyResultHandler {
    input: Shared<SimpleContainer>,
    result: Shared<ResultContainer>,
    pos: BlockPos,
    world: Arc<World>,
}

impl CartographyResultHandler {
    fn refresh(&self, guard: &mut ContainerLockGuard) {
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return;
        };
        let next = assemble_cartography(input_c.get_item(0), input_c.get_item(1), &self.world);
        result_c.set_item(0, next);
    }
}

impl ResultHandler for CartographyResultHandler {
    fn result_container(&self) -> ContainerRef {
        ContainerRef::from(Arc::clone(&self.result))
    }

    fn dependencies(&self) -> Vec<ContainerRef> {
        vec![ContainerRef::from(Arc::clone(&self.input))]
    }

    fn update_result(&self, guard: &mut ContainerLockGuard) {
        self.refresh(guard);
    }

    fn on_result_taken(
        &self,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) -> Option<ItemStack> {
        if let Some(input_c) = guard.get_mut(ContainerId::from_arc(&self.input)) {
            let _ = input_c.remove_item(0, 1);
            let _ = input_c.remove_item(1, 1);
        }
        self.refresh(guard);
        self.world.play_sound(
            &sound_events::UI_CARTOGRAPHY_TABLE_TAKE_RESULT,
            SoundSource::Blocks,
            self.pos,
            1.0,
            1.0,
            None,
        );
        None
    }

    fn is_result_valid(&self, guard: &ContainerLockGuard, _player: &Player) -> bool {
        let Some(input_c) = guard.get(ContainerId::from_arc(&self.input)) else {
            return false;
        };
        !assemble_cartography(input_c.get_item(0), input_c.get_item(1), &self.world).is_empty()
    }
}

/// Cartography table menu state (`CartographyTableMenu`).
pub struct CartographyTableKind {
    handler: Arc<CartographyResultHandler>,
    result: Shared<ResultContainer>,
    pos: BlockPos,
    world: Arc<World>,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for CartographyTableKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/cartography_table");
}

impl MenuKind for CartographyTableKind {
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        still_valid_block(
            &self.world,
            self.pos,
            &vanilla_blocks::CARTOGRAPHY_TABLE,
            player,
        )
    }

    fn removed(&mut self, _behavior: &mut MenuBehavior, _player: &Player) {
        self.result.lock().set_item(0, ItemStack::empty());
    }

    fn slots_changed(
        &mut self,
        _behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        self.handler.refresh(guard);
    }
}

/// Builds the vanilla cartography table menu (map, modifier, result).
#[must_use]
pub fn cartography_table(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    pos: BlockPos,
    world: &Arc<World>,
) -> Menu {
    let input = SimpleContainer::new(2).into_shared();
    let result = ResultContainer::new().into_shared();
    let handler = Arc::new(CartographyResultHandler {
        input: Arc::clone(&input),
        result: Arc::clone(&result),
        pos,
        world: Arc::clone(world),
    });

    let mut builder = MenuBuilder::new(&vanilla_menu_types::CARTOGRAPHY_TABLE, container_id);
    let input_section = builder.section_all_with(
        &input,
        SectionKind::restricted(|slot, stack| match slot {
            0 => stack.get(MAP_ID).is_some(),
            1 => {
                stack.is(&vanilla_items::PAPER)
                    || stack.is(&vanilla_items::MAP)
                    || stack.is(&vanilla_items::GLASS_PANE)
            }
            _ => false,
        }),
    );
    let result_section = builder.crafting_result_slot(Arc::clone(&handler));
    let player = builder.player_inventory(&inventory);

    builder.route_with_remainder_policy(
        result_section,
        player.all(),
        FillDirection::Backward,
        FakeResultRemainderPolicy::Discard,
    );
    builder.route(input_section, player.all(), FillDirection::Forward);
    builder.route(player.all(), input_section, FillDirection::Forward);
    builder.drain(input_section);

    builder.build(CartographyTableKind {
        handler,
        result,
        pos,
        world: Arc::clone(world),
    })
}

// ---------------------------------------------------------------------------
// Loom (`Vanilla LoomMenu`)
// ---------------------------------------------------------------------------

fn selectable_banner_patterns(pattern_stack: &ItemStack) -> Vec<BannerPatternRef> {
    if pattern_stack.is_empty() {
        return REGISTRY
            .banner_patterns
            .get_tag(&BannerPatternTag::NO_ITEM_REQUIRED)
            .unwrap_or_default();
    }
    match pattern_stack.get(PROVIDES_BANNER_PATTERNS) {
        Some(RegistryHolderSet::Tag(tag)) => {
            REGISTRY.banner_patterns.get_tag(tag).unwrap_or_default()
        }
        Some(RegistryHolderSet::Direct(list)) => list.clone(),
        None => Vec::new(),
    }
}

fn assemble_loom(banner: &ItemStack, dye: &ItemStack, pattern: BannerPatternRef) -> ItemStack {
    if banner.is_empty() || !banner.item().has_tag(&ItemTag::BANNERS) {
        return ItemStack::empty();
    }
    let Some(color) = dye.get(DYE).copied() else {
        return ItemStack::empty();
    };
    let mut layers = banner
        .get(BANNER_PATTERNS)
        .map_or_else(Vec::new, |existing| existing.layers().to_vec());
    if layers.len() >= MAX_LOOM_BANNER_LAYERS {
        return ItemStack::empty();
    }
    layers.push(BannerPatternLayer::new(
        RegistryHolder::reference(pattern),
        color,
    ));
    let mut out = banner.copy_with_count(1);
    out.set(BANNER_PATTERNS, BannerPatternLayers::new(layers));
    out
}

struct LoomResultHandler {
    input: Shared<SimpleContainer>,
    result: Shared<ResultContainer>,
    selected: Arc<AtomicI32>,
    pos: BlockPos,
    world: Arc<World>,
}

impl LoomResultHandler {
    fn current_output(&self, input_c: &dyn Container) -> ItemStack {
        let banner = input_c.get_item(0);
        let dye = input_c.get_item(1);
        let pattern_item = input_c.get_item(2);
        if banner.is_empty() || dye.is_empty() {
            return ItemStack::empty();
        }
        let Ok(index) = usize::try_from(self.selected.load(Ordering::Relaxed)) else {
            return ItemStack::empty();
        };
        let patterns = selectable_banner_patterns(pattern_item);
        let Some(&pattern) = patterns.get(index) else {
            return ItemStack::empty();
        };
        assemble_loom(banner, dye, pattern)
    }

    fn refresh(&self, guard: &mut ContainerLockGuard) {
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return;
        };
        let next = self.current_output(input_c);
        result_c.set_item(0, next);
    }
}

impl ResultHandler for LoomResultHandler {
    fn result_container(&self) -> ContainerRef {
        ContainerRef::from(Arc::clone(&self.result))
    }

    fn dependencies(&self) -> Vec<ContainerRef> {
        vec![ContainerRef::from(Arc::clone(&self.input))]
    }

    fn update_result(&self, guard: &mut ContainerLockGuard) {
        self.refresh(guard);
    }

    fn on_result_taken(
        &self,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) -> Option<ItemStack> {
        if let Some(input_c) = guard.get_mut(ContainerId::from_arc(&self.input)) {
            // Vanilla `LoomMenu`: consumes 1 banner + 1 dye; the banner pattern
            // item in slot 2 is reusable and is never consumed.
            let _ = input_c.remove_item(0, 1);
            let _ = input_c.remove_item(1, 1);
            if input_c.get_item(0).is_empty() || input_c.get_item(1).is_empty() {
                self.selected.store(-1, Ordering::Relaxed);
            }
        }
        self.refresh(guard);
        self.world.play_sound(
            &sound_events::UI_LOOM_TAKE_RESULT,
            SoundSource::Blocks,
            self.pos,
            1.0,
            1.0,
            None,
        );
        None
    }

    fn is_result_valid(&self, guard: &ContainerLockGuard, _player: &Player) -> bool {
        let Some(input_c) = guard.get(ContainerId::from_arc(&self.input)) else {
            return false;
        };
        !self.current_output(input_c).is_empty()
    }
}

/// Loom menu state (`LoomMenu`).
pub struct LoomKind {
    input: Shared<SimpleContainer>,
    result: Shared<ResultContainer>,
    selected: Arc<AtomicI32>,
    selected_slot: DataSlot,
    selectable: Vec<BannerPatternRef>,
    pos: BlockPos,
    world: Arc<World>,
}

// SAFETY: This Steel-owned key uniquely identifies the concrete menu kind
// within the process.
unsafe impl DowncastType for LoomKind {
    const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new("steel:menu/loom");
}

impl LoomKind {
    fn sync_selected(&self, behavior: &mut MenuBehavior, value: i32) {
        self.selected.store(value, Ordering::Relaxed);
        self.selected_slot
            .set(behavior, i16::try_from(value).unwrap_or(-1));
    }
}

impl MenuKind for LoomKind {
    fn still_valid(&self, _behavior: &MenuBehavior, player: &Player) -> bool {
        still_valid_block(&self.world, self.pos, &vanilla_blocks::LOOM, player)
    }

    fn removed(&mut self, _behavior: &mut MenuBehavior, _player: &Player) {
        self.result.lock().set_item(0, ItemStack::empty());
    }

    fn slots_changed(
        &mut self,
        behavior: &mut MenuBehavior,
        guard: &mut ContainerLockGuard,
        _player: &Player,
    ) {
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return;
        };
        let banner = input_c.get_item(0);
        let dye = input_c.get_item(1);
        let pattern_stack = input_c.get_item(2);

        if banner.is_empty() || dye.is_empty() {
            self.selectable.clear();
            self.sync_selected(behavior, -1);
            result_c.set_item(0, ItemStack::empty());
            return;
        }

        let previous_selected = self.selected.load(Ordering::Relaxed);
        let previous_pattern = usize::try_from(previous_selected)
            .ok()
            .and_then(|idx| self.selectable.get(idx).copied());
        self.selectable = selectable_banner_patterns(pattern_stack);

        let chosen = if self.selectable.len() == 1 {
            self.sync_selected(behavior, 0);
            Some(self.selectable[0])
        } else if let Some(prev) = previous_pattern
            && let Some(new_idx) = self
                .selectable
                .iter()
                .position(|entry| entry.key == prev.key)
        {
            self.sync_selected(behavior, new_idx as i32);
            Some(self.selectable[new_idx])
        } else {
            self.sync_selected(behavior, -1);
            None
        };

        if let Some(pattern) = chosen {
            let output = assemble_loom(banner, dye, pattern);
            if output.is_empty() {
                self.sync_selected(behavior, -1);
            }
            result_c.set_item(0, output);
        } else {
            result_c.set_item(0, ItemStack::empty());
        }
    }

    fn click_menu_button(
        &mut self,
        behavior: &mut MenuBehavior,
        button_id: i32,
        _player: &Player,
    ) -> bool {
        let Some(index) = usize::try_from(button_id)
            .ok()
            .filter(|&idx| idx < self.selectable.len())
        else {
            return false;
        };
        let pattern = self.selectable[index];
        let mut guard = ContainerLockGuard::lock_all(&[
            ContainerRef::from(&self.input),
            ContainerRef::from(&self.result),
        ]);
        let Some([input_c, result_c]) = guard.get_disjoint_mut([
            ContainerId::from_arc(&self.input),
            ContainerId::from_arc(&self.result),
        ]) else {
            return false;
        };
        let output = assemble_loom(input_c.get_item(0), input_c.get_item(1), pattern);
        if output.is_empty() {
            return false;
        }
        self.sync_selected(behavior, button_id);
        result_c.set_item(0, output);
        true
    }
}

/// Builds the vanilla loom menu (banner, dye, pattern, result).
#[must_use]
pub fn loom(
    inventory: Shared<PlayerInventory>,
    container_id: u8,
    pos: BlockPos,
    world: &Arc<World>,
) -> Menu {
    let input = SimpleContainer::new(3).into_shared();
    let result = ResultContainer::new().into_shared();
    let selected = Arc::new(AtomicI32::new(-1));

    let mut builder = MenuBuilder::new(&vanilla_menu_types::LOOM, container_id);
    let selected_slot = builder.data_slot(-1);
    let input_section = builder.section_all_with(
        &input,
        SectionKind::restricted(|slot, stack| match slot {
            0 => stack.item().has_tag(&ItemTag::BANNERS),
            1 => stack.get(DYE).is_some(),
            2 => stack.get(PROVIDES_BANNER_PATTERNS).is_some(),
            _ => false,
        }),
    );
    let result_section = builder.crafting_result_slot(LoomResultHandler {
        input: Arc::clone(&input),
        result: Arc::clone(&result),
        selected: Arc::clone(&selected),
        pos,
        world: Arc::clone(world),
    });
    let player = builder.player_inventory(&inventory);

    builder.route_with_remainder_policy(
        result_section,
        player.all(),
        FillDirection::Backward,
        FakeResultRemainderPolicy::Discard,
    );
    builder.route(input_section, player.all(), FillDirection::Forward);
    builder.route(player.all(), input_section, FillDirection::Forward);
    builder.drain(input_section);

    builder.build(LoomKind {
        input,
        result,
        selected,
        selected_slot,
        selectable: Vec::new(),
        pos,
        world: Arc::clone(world),
    })
}

#[cfg(test)]
mod tests {
    use steel_registry::data_components::vanilla_components::CUSTOM_NAME;
    use steel_registry::init_vanilla_registry;
    use text_components::TextComponent;

    use super::*;

    #[test]
    fn smithing_netherite_upgrade_preserves_custom_components() {
        init_vanilla_registry();
        let template = ItemStack::new(&vanilla_items::NETHERITE_UPGRADE_SMITHING_TEMPLATE);
        let mut sword = ItemStack::new(&vanilla_items::DIAMOND_SWORD);
        sword.set(CUSTOM_NAME, TextComponent::plain("Excalibur"));
        sword.set_damage_value(42);
        let ingot = ItemStack::new(&vanilla_items::NETHERITE_INGOT);

        let (upgraded, _) =
            assemble_smithing(&template, &sword, &ingot).expect("netherite upgrade should match");
        assert!(upgraded.is(&vanilla_items::NETHERITE_SWORD));
        assert_eq!(
            upgraded.get(CUSTOM_NAME),
            Some(&TextComponent::plain("Excalibur"))
        );
        assert_eq!(upgraded.get_damage_value(), 42);
    }

    #[test]
    fn smithing_armor_trim_applies_once_and_rejects_identical_retrim() {
        init_vanilla_registry();
        let template = ItemStack::new(&vanilla_items::COAST_ARMOR_TRIM_SMITHING_TEMPLATE);
        let chestplate = ItemStack::new(&vanilla_items::DIAMOND_CHESTPLATE);
        let material = ItemStack::new(&vanilla_items::EMERALD);

        let (trimmed, _) =
            assemble_smithing(&template, &chestplate, &material).expect("trim should match");
        assert!(trimmed.get(TRIM).is_some());
        assert!(
            assemble_smithing(&template, &trimmed, &material).is_none(),
            "vanilla rejects applying the exact same trim a second time"
        );
    }

    #[test]
    fn loom_applies_pattern_and_enforces_six_layer_limit() {
        init_vanilla_registry();
        let patterns = selectable_banner_patterns(&ItemStack::empty());
        assert!(!patterns.is_empty());
        let mut banner = ItemStack::new(&vanilla_items::WHITE_BANNER);
        let dye = ItemStack::new(&vanilla_items::BLUE_DYE);

        for expected in 1..=MAX_LOOM_BANNER_LAYERS {
            banner = assemble_loom(&banner, &dye, patterns[0]);
            assert_eq!(
                banner
                    .get(BANNER_PATTERNS)
                    .map(|layers| layers.layers().len()),
                Some(expected)
            );
        }
        assert!(assemble_loom(&banner, &dye, patterns[0]).is_empty());
    }

    #[test]
    fn stonecutter_produces_stone_recipes() {
        init_vanilla_registry();
        let stone = ItemStack::new(&vanilla_items::STONE);
        let (first, _) = StonecutterResultHandler::result_for(&stone, 0)
            .expect("stone has stonecutting recipes");
        assert!(!first.is_empty());
        assert!(StonecutterResultHandler::result_for(&stone, -1).is_none());
    }
}
