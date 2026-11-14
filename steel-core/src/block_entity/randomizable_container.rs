//! Vanilla `RandomizableContainer` state: the loot table a container was placed
//! with and the seed that fills it.
//!
//! Structure templates place chests, barrels and shulker boxes carrying
//! `LootTable` and `LootTableSeed` instead of items, and vanilla fills them the
//! first time a player looks inside. Steel previously loaded the NBT items and
//! dropped both fields, so every structure chest was empty and a relog could
//! never re-roll it.
//!
//! The state here is deliberately separate from [`crate::inventory::container::Container`]:
//! vanilla keeps it in `RandomizableContainerBlockEntity` and only rolls the
//! table in `unpackLootTable`, which is what this module mirrors.

use rand::Rng;
use steel_registry::REGISTRY;
use steel_registry::item_stack::ItemStack;
use steel_registry::loot_table::LootTableRegistry;
use steel_registry::loot_table::context::LootContext;
use steel_registry::loot_table::entries::LootRngSeed;
use steel_utils::Identifier;
use steel_utils::locks::SyncMutex;

/// Loot state of one container, mirroring vanilla `RandomizableContainer`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RandomizableContainer {
    /// The loot table key, `None` when the container was placed with items.
    loot_table: Option<Identifier>,
    /// Vanilla `LootTableSeed`: `0` means "derive the seed from the table".
    loot_table_seed: i64,
}

impl RandomizableContainer {
    /// Creates the empty (no loot table) state.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            loot_table: None,
            loot_table_seed: 0,
        }
    }

    /// Vanilla `RandomizableContainer.setLootTable`.
    pub fn set_loot_table(&mut self, table: Option<Identifier>, seed: i64) {
        self.loot_table = table;
        self.loot_table_seed = seed;
    }

    /// Vanilla `RandomizableContainer.tryLoadLootTable`.
    ///
    /// Every container block entity reads the same two fields, so the parse
    /// lives here instead of being repeated per container.
    pub fn load_from_nbt(nbt: simdnbt::borrow::NbtCompound<'_, '_>, loot: &SyncMutex<Self>) {
        use steel_utils::nbt::NbtValueInput as _;
        use std::str::FromStr as _;

        let table = nbt
            .string("LootTable")
            .and_then(|key| Identifier::from_str(&key.to_str()).ok());
        // Vanilla defaults the seed to 0, which means "randomize".
        let seed = nbt.long_value("LootTableSeed").unwrap_or(0);
        loot.lock().set_loot_table(table, seed);
    }

    /// Returns the loot table key this container was placed with.
    #[must_use]
    pub const fn loot_table(&self) -> Option<&Identifier> {
        self.loot_table.as_ref()
    }

    /// Returns the stored `LootTableSeed`.
    #[must_use]
    pub const fn loot_table_seed(&self) -> i64 {
        self.loot_table_seed
    }

    /// Whether a loot table still has to be rolled.
    #[must_use]
    pub const fn has_loot_table(&self) -> bool {
        self.loot_table.is_some()
    }

    /// Vanilla `RandomizableContainer.unpackLootTable`: rolls the table and
    /// fills `items`, then clears the table so it is rolled exactly once.
    ///
    /// The fill reuses the roll's RNG, exactly like vanilla's `LootTable.fill`,
    /// which draws the roll and the slot shuffle from the same source.
    ///
    /// Returns whether a pending table was rolled.
    pub fn unpack_loot_table(&mut self, items: &mut [ItemStack], world_seed: i64) -> bool {
        let Some(key) = self.loot_table.take() else {
            return false;
        };
        let seed = std::mem::take(&mut self.loot_table_seed);

        let registry: &LootTableRegistry = &REGISTRY.loot_tables;
        let Some(table) = registry.by_key(&key) else {
            // Vanilla logs the missing table and leaves the container empty.
            log::warn!("Container references unknown loot table {key}");
            return false;
        };

        match table.loot_rng_seed(seed, world_seed as u64) {
            Some(rng_seed) => roll_into(items, table, &mut LootRngSeed::rng(rng_seed)),
            None => {
                // Vanilla falls back to the level's random source. Steel has no
                // level-wide source reachable from a block entity, so the roll
                // uses the thread RNG, the same fallback `BrushableBlockEntity`
                // documents.
                roll_into(items, table, &mut rand::rng());
            }
        }
        true
    }

    /// Writes the fields vanilla's `trySaveLootTable` writes.
    ///
    /// An unpacked container writes nothing, and a zero seed is omitted, which
    /// is what vanilla does (a zero seed means "use the random sequence").
    pub fn save_to_nbt(&self, nbt: &mut simdnbt::owned::NbtCompound) {
        let Some(table) = self.loot_table.as_ref() else {
            return;
        };
        nbt.insert("LootTable", table.to_string());
        if self.loot_table_seed != 0 {
            nbt.insert("LootTableSeed", self.loot_table_seed);
        }
    }
}

/// Rolls `table` with `rng` and places the result into `items`.
fn roll_into<R: Rng>(
    items: &mut [ItemStack],
    table: steel_registry::loot_table::LootTableRef,
    rng: &mut R,
) {
    let mut ctx = LootContext::new(rng);
    let loot = table.get_random_items(&mut ctx);
    fill_from_loot(items, loot, rng);
}

/// Places generated loot into a container the way vanilla's `LootTable.fill`
/// does.
///
/// Vanilla collects the free slots, shuffles that list with the roll's random
/// source and then hands each rolled stack the last remaining slot, splitting a
/// stack across slots when it exceeds the item's max stack size. Occupied slots
/// are never offered, which is why a structure chest that also carries items
/// keeps them; once the free slots run out the remaining stacks are dropped.
///
/// The exact Java slot order is not reproducible outside Java, but the item
/// multiset, the splitting and the "only free slots" rule are vanilla's, and the
/// layout is deterministic for a given seed.
pub fn fill_from_loot<S: Rng>(slots: &mut [ItemStack], loot: Vec<ItemStack>, rng: &mut S) {
    let mut pending: Vec<ItemStack> = loot.into_iter().filter(|item| !item.is_empty()).collect();
    if pending.is_empty() {
        return;
    }

    // `LootTable.getAvailableSlots`: only empty slots, in slot order.
    let mut free: Vec<usize> = slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| slot.is_empty().then_some(index))
        .collect();
    if free.is_empty() {
        // Vanilla warns and returns without placing anything.
        return;
    }

    // `Collections.shuffle`: walk from the end, swapping with a random earlier
    // element, exactly like the JDK implementation.
    for index in (1..free.len()).rev() {
        free.swap(index, rng.random_range(0..=index));
    }

    let mut item_index = 0;
    while item_index < pending.len() {
        // Vanilla takes the last slot of the shuffled list each time.
        let Some(slot) = free.pop() else {
            // `Tried to over-fill a container`: the rest is dropped.
            return;
        };
        let mut stack = pending[item_index].clone();
        let max_stack = stack.max_stack_size();
        if stack.count > max_stack {
            // Vanilla splits the overflow into the next slot.
            stack.set_count(max_stack);
            pending[item_index].count -= max_stack;
            slots[slot] = stack;
        } else {
            slots[slot] = stack;
            item_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RandomizableContainer, fill_from_loot};
    use steel_registry::init_vanilla_registry;
    use rand::SeedableRng as _;
    use rand::rngs::StdRng;
    use steel_registry::item_stack::ItemStack;
    use steel_utils::Identifier;

    fn rng() -> StdRng {
        StdRng::seed_from_u64(7)
    }

    #[test]
    fn an_empty_container_has_no_table() {
        let state = RandomizableContainer::none();
        assert!(!state.has_loot_table());
        assert_eq!(state.loot_table(), None);
    }

    #[test]
    fn a_container_keeps_the_seed_from_the_template() {
        // Vanilla `loadAdditional`: `LootTableSeed` defaults to 0.
        let mut state = RandomizableContainer::none();
        state.set_loot_table(Some(Identifier::vanilla_static("chests/simple_dungeon")), 42);
        assert_eq!(
            state.loot_table().map(Identifier::to_string),
            Some("minecraft:chests/simple_dungeon".to_owned())
        );
        assert_eq!(state.loot_table_seed(), 42);
    }

    #[test]
    fn unpacking_clears_the_table_so_it_rolls_once() {
        init_vanilla_registry();
        let mut state = RandomizableContainer::none();
        state.set_loot_table(Some(Identifier::vanilla_static("chests/does_not_exist")), 7);
        let mut slots = vec![ItemStack::empty(); 27];
        // An unknown table yields no items but must still be consumed, matching
        // vanilla's `setLootTable(null, 0)` after the attempt.
        assert!(!state.unpack_loot_table(&mut slots, 0));
        assert!(!state.has_loot_table(), "the table is rolled only once");
        assert_eq!(state.loot_table_seed(), 0);
        assert!(slots.iter().all(ItemStack::is_empty));
    }

    #[test]
    fn filling_stops_when_the_container_is_full() {
        let mut slots = vec![ItemStack::empty(); 2];
        // `vanilla_items` entries are `LazyLock`s, hence the deref.
        let stone = ItemStack::new(&*steel_registry::vanilla_items::STONE);
        let dirt = ItemStack::new(&*steel_registry::vanilla_items::DIRT);
        let sand = ItemStack::new(&*steel_registry::vanilla_items::SAND);
        fill_from_loot(&mut slots, vec![stone, dirt, sand], &mut rng());
        // The third item has no free slot and is dropped, like vanilla's fill.
        assert!(!slots[0].is_empty());
        assert!(!slots[1].is_empty());
    }

    #[test]
    fn filling_never_overwrites_a_present_item() {
        // Vanilla `getAvailableSlots` only offers empty slots, so a structure
        // chest that also carries items keeps them.
        let mut slots = vec![ItemStack::empty(); 3];
        let dirt = ItemStack::new(&*steel_registry::vanilla_items::DIRT);
        slots[1] = dirt.clone();
        fill_from_loot(
            &mut slots,
            vec![ItemStack::new(&*steel_registry::vanilla_items::STONE)],
            &mut rng(),
        );
        assert_eq!(slots[1], dirt, "the pre-placed item survives the fill");
        assert!(!slots[0].is_empty() || !slots[2].is_empty());
    }

    #[test]
    fn a_real_table_fills_the_container_deterministically() {
        init_vanilla_registry();
        // Runs the actual registry table through the real roll path: the same
        // seed must produce the same layout on both calls, and the second call
        // on the same state must do nothing (the table is consumed).
        let mut first_slots = vec![ItemStack::empty(); 27];
        let mut first = RandomizableContainer::none();
        first.set_loot_table(Some(Identifier::vanilla_static("chests/simple_dungeon")), 1234);
        assert!(first.unpack_loot_table(&mut first_slots, 99));
        let filled = first_slots.iter().filter(|slot| !slot.is_empty()).count();
        assert!(filled > 0, "simple_dungeon must roll at least one item");
        assert!(!first.has_loot_table(), "the table is consumed by the roll");
        assert!(!first.unpack_loot_table(&mut first_slots, 99));

        let mut second_slots = vec![ItemStack::empty(); 27];
        let mut second = RandomizableContainer::none();
        second.set_loot_table(Some(Identifier::vanilla_static("chests/simple_dungeon")), 1234);
        assert!(second.unpack_loot_table(&mut second_slots, 99));
        assert_eq!(
            filled,
            second_slots
                .iter()
                .filter(|slot| !slot.is_empty())
                .count(),
            "the same seed rolls the same layout"
        );
        for (first_slot, second_slot) in first_slots.iter().zip(&second_slots) {
            assert_eq!(first_slot, second_slot, "slot placement is deterministic");
        }
    }

    #[test]
    fn empty_loot_never_touches_the_slots() {
        let mut slots = vec![ItemStack::empty(); 1];
        fill_from_loot(&mut slots, vec![ItemStack::empty()], &mut rng());
        assert!(slots[0].is_empty());
    }
}
