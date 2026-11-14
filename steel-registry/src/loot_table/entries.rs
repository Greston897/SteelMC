use super::{
    ConditionalLootFunction, Identifier, ItemStack, LootCondition, LootContext, LootType,
    NumberProvider, REGISTRY, RegistryExt, RngExt, TaggedRegistryExt,
};

/// A loot table entry that can generate items.
#[derive(Debug, Clone)]
pub enum LootEntry {
    /// Drop a specific item.
    Item {
        name: Identifier,
        weight: i32,
        quality: i32,
        conditions: &'static [LootCondition],
        functions: &'static [ConditionalLootFunction],
    },
    /// Reference another loot table by name.
    LootTableRef {
        name: Identifier,
        weight: i32,
        quality: i32,
        conditions: &'static [LootCondition],
        functions: &'static [ConditionalLootFunction],
    },
    /// Inline loot table (embedded pools directly in entry).
    InlineLootTable {
        pools: &'static [LootPool],
        weight: i32,
        quality: i32,
        conditions: &'static [LootCondition],
        functions: &'static [ConditionalLootFunction],
    },
    /// Drop items from a tag.
    Tag {
        name: Identifier,
        expand: bool,
        weight: i32,
        quality: i32,
        conditions: &'static [LootCondition],
        functions: &'static [ConditionalLootFunction],
    },
    /// Try children in order, use first that matches.
    Alternatives {
        children: &'static [LootEntry],
        conditions: &'static [LootCondition],
    },
    /// Use all children.
    Group {
        children: &'static [LootEntry],
        conditions: &'static [LootCondition],
    },
    /// Use children in sequence until one fails.
    Sequence {
        children: &'static [LootEntry],
        conditions: &'static [LootCondition],
    },
    /// Empty entry (no drop).
    Empty {
        weight: i32,
        conditions: &'static [LootCondition],
    },
    /// Dynamic content (e.g., block entity contents).
    Dynamic {
        name: Identifier,
        conditions: &'static [LootCondition],
    },
    /// Select items from specific block entity slots.
    Slots {
        /// Slots to select from (can be single slot or range).
        slots: SlotRange,
        conditions: &'static [LootCondition],
        functions: &'static [ConditionalLootFunction],
    },
}

/// A range of slots for the Slots entry type.
#[derive(Debug, Clone, Copy)]
pub enum SlotRange {
    /// A single specific slot index.
    Single(i32),
    /// A range of slots (inclusive).
    Range { min: i32, max: i32 },
    /// All contents slots.
    Contents,
    /// Specific named slots (for entities).
    Named(&'static [&'static str]),
}

impl LootEntry {
    /// Get the weight of this entry for random selection.
    #[must_use]
    pub const fn weight(&self) -> i32 {
        match self {
            Self::Item { weight, .. } => *weight,
            Self::LootTableRef { weight, .. } => *weight,
            Self::InlineLootTable { weight, .. } => *weight,
            Self::Tag { weight, .. } => *weight,
            Self::Empty { weight, .. } => *weight,
            // Composite entries don't have weight
            Self::Alternatives { .. }
            | Self::Group { .. }
            | Self::Sequence { .. }
            | Self::Dynamic { .. }
            | Self::Slots { .. } => 1,
        }
    }

    /// Get the quality modifier for luck-based weight adjustment.
    #[must_use]
    pub const fn quality(&self) -> i32 {
        match self {
            Self::Item { quality, .. } => *quality,
            Self::LootTableRef { quality, .. } => *quality,
            Self::InlineLootTable { quality, .. } => *quality,
            Self::Tag { quality, .. } => *quality,
            Self::Empty { .. }
            | Self::Alternatives { .. }
            | Self::Group { .. }
            | Self::Sequence { .. }
            | Self::Dynamic { .. }
            | Self::Slots { .. } => 0,
        }
    }

    /// Get the effective weight adjusted for luck.
    /// Formula: max(floor(weight + quality * luck), 0)
    #[must_use]
    pub fn effective_weight(&self, luck: f32) -> i32 {
        let base = self.weight() as f32;
        let quality = self.quality() as f32;
        (base + quality * luck).floor().max(0.0) as i32
    }

    /// Get the conditions for this entry.
    #[must_use]
    pub const fn conditions(&self) -> &'static [LootCondition] {
        match self {
            Self::Item { conditions, .. } => conditions,
            Self::LootTableRef { conditions, .. } => conditions,
            Self::InlineLootTable { conditions, .. } => conditions,
            Self::Tag { conditions, .. } => conditions,
            Self::Alternatives { conditions, .. } => conditions,
            Self::Group { conditions, .. } => conditions,
            Self::Sequence { conditions, .. } => conditions,
            Self::Empty { conditions, .. } => conditions,
            Self::Dynamic { conditions, .. } => conditions,
            Self::Slots { conditions, .. } => conditions,
        }
    }

    /// Get the functions for this entry.
    #[must_use]
    pub const fn functions(&self) -> &'static [ConditionalLootFunction] {
        match self {
            Self::Item { functions, .. } => functions,
            Self::LootTableRef { functions, .. } => functions,
            Self::InlineLootTable { functions, .. } => functions,
            Self::Tag { functions, .. } => functions,
            Self::Slots { functions, .. } => functions,
            Self::Empty { .. }
            | Self::Alternatives { .. }
            | Self::Group { .. }
            | Self::Sequence { .. }
            | Self::Dynamic { .. } => &[],
        }
    }
}

/// A pool of loot entries with roll counts.
#[derive(Debug, Clone)]
pub struct LootPool {
    pub rolls: NumberProvider,
    pub bonus_rolls: f32,
    pub entries: &'static [LootEntry],
    pub conditions: &'static [LootCondition],
    pub functions: &'static [ConditionalLootFunction],
}

/// A complete loot table definition.
#[derive(Debug)]
pub struct LootTable {
    pub key: Identifier,
    pub loot_type: LootType,
    pub pools: &'static [LootPool],
    pub functions: &'static [ConditionalLootFunction],
    pub random_sequence: Option<Identifier>,
}

impl LootTable {
    /// Selects the random source vanilla would use for this table.
    ///
    /// Vanilla `LootTable.getRandomItems` picks its RNG before evaluating
    /// anything:
    ///
    /// 1. a non-zero loot seed (a block entity's `LootTableSeed`) replays the roll
    ///    with `LegacyRandom`, so the same chest always yields the same loot;
    /// 2. otherwise a table with a `random_sequence` uses that sequence's stream,
    ///    derived from the world seed, so every table sharing a sequence replays
    ///    identically while independent tables stay independent;
    /// 3. otherwise the level's own random source is used.
    ///
    /// Returning the seed keeps the registry free of world state: the caller
    /// builds the `Rng` and evaluates the table.
    #[must_use]
    pub fn loot_rng_seed(&self, loot_seed: i64, world_seed: u64) -> Option<LootRngSeed> {
        if loot_seed != 0 {
            return Some(LootRngSeed::Legacy(loot_seed as u64));
        }
        self.random_sequence
            .as_ref()
            .map(|sequence| LootRngSeed::Sequence(sequence_seed(world_seed, sequence)))
    }

    /// Generate random items from this loot table.
    ///
    /// # Arguments
    /// * `ctx` - The loot context containing RNG, luck, block state, tool, etc.
    ///
    /// This follows vanilla's approach:
    /// 1. For each pool, check conditions
    /// 2. Roll `rolls + floor(bonus_rolls * luck)` times
    /// 3. Each roll does weighted random selection among valid entries
    /// 4. Apply entry-level functions to each item
    /// 5. Apply pool-level functions to all items from that pool
    /// 6. Apply table-level functions to all items from the table
    pub fn get_random_items<R: rand::Rng>(&self, ctx: &mut LootContext<'_, R>) -> Vec<ItemStack> {
        let mut result = Vec::new();
        for pool in self.pools {
            pool.add_random_items(ctx, &mut result);
        }

        // Apply table-level functions to all items
        if !self.functions.is_empty() {
            for item in &mut result {
                for cond_func in self.functions {
                    if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                        cond_func.function.apply(item, ctx);
                    }
                }
            }
            // Remove items with zero count after applying functions
            result.retain(|item| item.count > 0);
        }

        result
    }
}

impl LootPool {
    /// Add random items from this pool to the result.
    fn add_random_items<R: rand::Rng>(
        &self,
        ctx: &mut LootContext<'_, R>,
        result: &mut Vec<ItemStack>,
    ) {
        // Check pool conditions
        for condition in self.conditions {
            if !condition.test(ctx) {
                return;
            }
        }

        // Track where items from this pool start
        let start_index = result.len();

        // Calculate number of rolls
        let roll_count = self.rolls.get_int(ctx.rng) + (self.bonus_rolls * ctx.luck).floor() as i32;

        for _ in 0..roll_count {
            self.add_random_item(ctx, result);
        }

        // Apply pool-level functions to all items generated by this pool
        if !self.functions.is_empty() {
            for item in result.iter_mut().skip(start_index) {
                for cond_func in self.functions {
                    if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                        cond_func.function.apply(item, ctx);
                    }
                }
            }
            // Remove items with zero count after applying functions
            result.retain(|item| item.count > 0);
        }
    }

    /// Select and add a single random item from this pool.
    fn add_random_item<R: rand::Rng>(
        &self,
        ctx: &mut LootContext<'_, R>,
        result: &mut Vec<ItemStack>,
    ) {
        // Collect valid entries with their effective weights
        let mut valid_entries: Vec<(&LootEntry, i32)> = Vec::new();
        let mut total_weight = 0;

        for entry in self.entries {
            // Check entry conditions
            let passes_conditions = entry.conditions().iter().all(|c| c.test(ctx));

            if !passes_conditions {
                continue;
            }

            let weight = entry.effective_weight(ctx.luck);
            if weight > 0 {
                valid_entries.push((entry, weight));
                total_weight += weight;
            }
        }

        if total_weight == 0 || valid_entries.is_empty() {
            return;
        }

        // Weighted random selection
        let selected = if valid_entries.len() == 1 {
            valid_entries[0].0
        } else {
            let mut index = ctx.rng.random_range(0..total_weight);
            let mut selected_entry = valid_entries[0].0;
            for (entry, weight) in &valid_entries {
                index -= weight;
                if index < 0 {
                    selected_entry = entry;
                    break;
                }
            }
            selected_entry
        };

        // Generate item(s) from the selected entry
        selected.create_items(ctx, result);
    }
}

impl LootEntry {
    /// Create items from this entry and add them to the result.
    pub(crate) fn create_items<R: rand::Rng>(
        &self,
        ctx: &mut LootContext<'_, R>,
        result: &mut Vec<ItemStack>,
    ) {
        match self {
            LootEntry::Item {
                name, functions, ..
            } => {
                if let Some(item_ref) = REGISTRY.items.by_key(name) {
                    let mut item = ItemStack::new(item_ref);

                    // Apply functions
                    for cond_func in *functions {
                        if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                            cond_func.function.apply(&mut item, ctx);
                        }
                    }

                    if item.count > 0 {
                        result.push(item);
                    }
                }
            }
            LootEntry::LootTableRef {
                name, functions, ..
            } => {
                // Recursively get items from referenced loot table
                if let Some(table) = REGISTRY.loot_tables.by_key(name) {
                    let mut items = table.get_random_items(ctx);
                    // Apply functions to all items from the referenced table
                    for item in &mut items {
                        for cond_func in *functions {
                            if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                                cond_func.function.apply(item, ctx);
                            }
                        }
                    }
                    result.extend(items.into_iter().filter(|i| i.count > 0));
                }
            }
            LootEntry::InlineLootTable {
                pools, functions, ..
            } => {
                // Process inline loot table pools directly
                let mut items = Vec::new();
                for pool in *pools {
                    pool.add_random_items(ctx, &mut items);
                }
                // Apply functions to all items from the inline table
                for item in &mut items {
                    for cond_func in *functions {
                        if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                            cond_func.function.apply(item, ctx);
                        }
                    }
                }
                result.extend(items.into_iter().filter(|i| i.count > 0));
            }
            LootEntry::Tag {
                name,
                expand,
                functions,
                ..
            } => {
                // Get all items in the tag
                if let Some(items) = REGISTRY.items.get_tag(name) {
                    if *expand {
                        // Pick one random item from the tag (weighted equally)
                        if !items.is_empty() {
                            let index = ctx.rng.random_range(0..items.len());
                            let mut item = ItemStack::new(items[index]);
                            for cond_func in *functions {
                                if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                                    cond_func.function.apply(&mut item, ctx);
                                }
                            }
                            if item.count > 0 {
                                result.push(item);
                            }
                        }
                    } else {
                        // Drop all items from the tag
                        for item_ref in items {
                            let mut item = ItemStack::new(item_ref);
                            for cond_func in *functions {
                                if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                                    cond_func.function.apply(&mut item, ctx);
                                }
                            }
                            if item.count > 0 {
                                result.push(item);
                            }
                        }
                    }
                }
            }
            LootEntry::Alternatives { children, .. } => {
                // Try children in order, use first that passes conditions and produces items
                for child in *children {
                    // Check child's conditions first
                    let passes_conditions = child.conditions().iter().all(|c| c.test(ctx));
                    if !passes_conditions {
                        continue; // Try next alternative
                    }

                    let before_len = result.len();
                    child.create_items(ctx, result);
                    if result.len() > before_len {
                        break; // First successful child that produced items, stop
                    }
                }
            }
            LootEntry::Group { children, .. } => {
                // Use all children that pass their conditions
                for child in *children {
                    let passes_conditions = child.conditions().iter().all(|c| c.test(ctx));
                    if passes_conditions {
                        child.create_items(ctx, result);
                    }
                }
            }
            LootEntry::Sequence { children, .. } => {
                // Use children in sequence until one fails its conditions
                // Note: Unlike Alternatives, Sequence stops when conditions FAIL,
                // not when items are produced. A child can produce nothing but still "succeed".
                for child in *children {
                    let passes_conditions = child.conditions().iter().all(|c| c.test(ctx));
                    if !passes_conditions {
                        break; // Condition failed, stop sequence
                    }
                    child.create_items(ctx, result);
                }
            }
            LootEntry::Empty { .. } => {
                // Empty entry produces nothing
            }
            LootEntry::Dynamic { name, .. } => {
                // Vanilla `DynamicLootEntry`: only `minecraft:contents` is
                // defined, and it yields the block entity's whole inventory.
                if name.path.as_ref() != "contents" {
                    return;
                }
                let Some(inventory) = ctx
                    .block_entity
                    .as_ref()
                    .and_then(|block_entity| block_entity.inventory)
                else {
                    return;
                };
                result.extend(inventory.iter().filter(|item| !item.is_empty()).cloned());
            }
            LootEntry::Slots {
                slots, functions, ..
            } => {
                // Vanilla `SlotsLootEntry`: read the named slots, then run the
                // entry functions on each selected stack.
                let Some(inventory) = ctx
                    .block_entity
                    .as_ref()
                    .and_then(|block_entity| block_entity.inventory)
                else {
                    return;
                };
                // Vanilla `SlotRange` resolves to a slot interval; `Named` slots
                // only exist for entity inventories, which Steel has no loot-context
                // access to yet.
                let (first, last) = match *slots {
                    SlotRange::Single(slot) => (slot, slot),
                    SlotRange::Range { min, max } => (min, max),
                    SlotRange::Contents => (0, inventory.len() as i32 - 1),
                    SlotRange::Named(_) => return,
                };
                for slot in first..=last {
                    let Ok(index) = usize::try_from(slot) else {
                        continue;
                    };
                    let Some(stack) = inventory.get(index) else {
                        continue;
                    };
                    if stack.is_empty() {
                        continue;
                    }
                    let mut stack = stack.clone();
                    for cond_func in *functions {
                        if cond_func.conditions.iter().all(|c| c.test(ctx)) {
                            cond_func.function.apply(&mut stack, ctx);
                        }
                    }
                    if !stack.is_empty() {
                        result.push(stack);
                    }
                }
            }
        }
    }
}

/// Derives the per-sequence seed vanilla's `RandomSequences` uses.
///
/// The exact hash is an implementation detail of vanilla's salt table; Steel
/// mixes the world seed with the sequence's `Random` salt using the same
/// multiply-xor scheme as `PositionalRandomFactory`, which keeps the streams
/// stable and independent.
fn sequence_seed(world_seed: u64, sequence: &Identifier) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64; // FNV-1a offset basis
    for byte in sequence.to_string().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    let salt = steel_utils::random::get_seed(
        hash as i32,
        (hash >> 32) as i32,
        (hash >> 16) as i32,
    );
    world_seed ^ (salt as u64)
}

/// The random source a loot table must be evaluated with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootRngSeed {
    /// Replay the roll from a stored loot seed using vanilla's legacy LCG.
    Legacy(u64),
    /// Use the derived stream of the table's `random_sequence`.
    Sequence(u64),
}

impl LootRngSeed {
    /// Creates the generator for this selection.
    #[must_use]
    pub fn rng(self) -> rand::rngs::StdRng {
        use rand::SeedableRng as _;

        let seed = match self {
            Self::Legacy(seed) | Self::Sequence(seed) => seed,
        };
        rand::rngs::StdRng::seed_from_u64(seed)
    }
}

#[cfg(test)]
mod seed_tests {
    use steel_utils::Identifier;

    use super::{LootRngSeed, LootTable, LootType, sequence_seed};

    fn table(random_sequence: Option<&'static str>) -> LootTable {
        LootTable {
            key: Identifier::vanilla_static("test/table"),
            loot_type: LootType::Chest,
            pools: &[],
            functions: &[],
            random_sequence: random_sequence.map(Identifier::vanilla_static),
        }
    }

    #[test]
    fn a_stored_loot_seed_wins_over_every_other_source() {
        let table = table(Some("test/sequence"));
        assert_eq!(
            table.loot_rng_seed(12345, 999),
            Some(LootRngSeed::Legacy(12345))
        );
    }

    #[test]
    fn a_random_sequence_derives_a_stable_world_seeded_stream() {
        let table = table(Some("test/sequence"));
        let first = table.loot_rng_seed(0, 42);
        let second = table.loot_rng_seed(0, 42);
        assert_eq!(first, second);
        assert_ne!(first, table.loot_rng_seed(0, 43));
        assert!(matches!(first, Some(LootRngSeed::Sequence(_))));
    }

    #[test]
    fn a_plain_table_defers_to_the_level_rng() {
        assert_eq!(table(None).loot_rng_seed(0, 42), None);
    }

    #[test]
    fn distinct_sequences_from_one_world_seed_differ() {
        assert_ne!(
            sequence_seed(7, &Identifier::vanilla_static("test/a")),
            sequence_seed(7, &Identifier::vanilla_static("test/b"))
        );
    }
}
