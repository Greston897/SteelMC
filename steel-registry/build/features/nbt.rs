//! Build-time NBT codegen for the `worldgen/block_state_provider` registry
//! sync packet. Mirrors `BlockStateProvider.DIRECT_CODEC`'s field names —
//! the same names already used by this module's JSON [`Deserialize`] impls,
//! since vanilla's codecs are format-agnostic.

use std::collections::BTreeMap;

use super::{
    BiomeHolderSet, BlockHolderSet, BlockPredicate, BlockStateData, BlockStateProviderKind,
    FeatureNoiseParameters, FluidHolderSet, Identifier, IntProvider, TokenStream, VerticalAnchor,
    WeightedIntProvider, quote,
};
use crate::block_state_defaults::block_property_defaults;

pub(super) fn generate_identifier_nbt(identifier: &Identifier) -> TokenStream {
    let id = identifier.to_string();
    quote! { NbtTag::String(#id.into()) }
}

/// Vanilla `HolderSetCodec` encoding: a named set is the `#tag` string
/// (`TagKey.hashedCodec`), a single direct entry is the bare id, and anything
/// else is the id list (`ExtraCodecs.compactListCodec`).
fn generate_holder_set_nbt(tag: Option<&Identifier>, entries: &[Identifier]) -> TokenStream {
    if let Some(tag) = tag {
        let tag = format!("#{tag}");
        return quote! { NbtTag::String(#tag.into()) };
    }
    if let [item] = entries {
        generate_identifier_nbt(item)
    } else {
        generate_nbt_list(entries.iter().map(generate_identifier_nbt))
    }
}

fn generate_predicate_offset(offset: &[i32; 3]) -> TokenStream {
    if *offset == [0, 0, 0] {
        TokenStream::new()
    } else {
        let offset = generate_offset_nbt(offset);
        quote! { compound.insert("offset", #offset); }
    }
}

/// Builds an `NbtTag::List`, using `NbtList::Empty` for an empty source —
/// `NbtList::from(vec![])` can't infer its element type.
fn generate_nbt_list(items: impl Iterator<Item = TokenStream>) -> TokenStream {
    let items: Vec<_> = items.collect();
    if items.is_empty() {
        quote! { NbtTag::List(NbtList::Empty) }
    } else {
        quote! { NbtTag::List(NbtList::from(vec![#(#items),*])) }
    }
}

/// Resolves a file-level state against vanilla's full property set: start
/// from the block's defaults, overlay explicit entries, and drop unknown
/// keys (vanilla's codec ignores them on decode). Unknown blocks pass
/// through untouched.
fn resolve_state_properties(data: &BlockStateData) -> BTreeMap<String, String> {
    let Some(defaults) = block_property_defaults().get(&data.name.to_string()) else {
        return data.properties.clone();
    };
    let mut resolved = defaults.clone();
    for (key, value) in &data.properties {
        if let Some(slot) = resolved.get_mut(key) {
            *slot = value.clone();
        }
    }
    resolved
}

/// Emits a `BlockState.CODEC` value: a state resolving to the block's
/// defaults encodes as the bare block id, anything else as the
/// `{id, properties}` object. Used for weighted `data`, noise `states`,
/// threshold states and `would_survive` state — every nested state position
/// except `FULL_CODEC` ones (top-level `Simple`, holder-inline
/// `source`/`then`/`state`).
pub(super) fn generate_block_state_data_codec_nbt(data: &BlockStateData) -> TokenStream {
    let resolved = resolve_state_properties(data);
    let is_default = match block_property_defaults().get(&data.name.to_string()) {
        Some(defaults) => resolved == *defaults,
        // Unknown blocks have no known defaults; only the bare `{id}` form
        // can collapse to the id string.
        None => data.properties.is_empty(),
    };
    if is_default {
        return generate_identifier_nbt(&data.name);
    }
    generate_block_state_data_nbt(data)
}

pub(super) fn generate_block_state_data_nbt(data: &BlockStateData) -> TokenStream {
    let id = generate_identifier_nbt(&data.name);
    // `FULL_CODEC` writes the complete sorted property map for
    // property-having blocks; only singleton blocks omit `properties`.
    let properties = resolve_state_properties(data);
    if properties.is_empty() {
        quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("id", #id);
            NbtTag::Compound(compound)
        }}
    } else {
        let entries = properties.iter().map(|(key, value)| {
            quote! { properties.insert(#key, #value); }
        });
        quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("id", #id);
            let mut properties = NbtCompound::new();
            #(#entries)*
            compound.insert("properties", NbtTag::Compound(properties));
            NbtTag::Compound(compound)
        }}
    }
}

pub(super) fn generate_block_holder_set_nbt(set: &BlockHolderSet) -> TokenStream {
    match set {
        BlockHolderSet::Tag(tag) => generate_holder_set_nbt(Some(tag), &[]),
        BlockHolderSet::Entries(entries) => generate_holder_set_nbt(None, entries),
    }
}

fn generate_fluid_holder_set_nbt(set: &FluidHolderSet) -> TokenStream {
    match set {
        FluidHolderSet::Tag(tag) => generate_holder_set_nbt(Some(tag), &[]),
        FluidHolderSet::Entries(entries) => generate_holder_set_nbt(None, entries),
    }
}

fn generate_biome_holder_set_nbt(set: &BiomeHolderSet) -> TokenStream {
    match set {
        BiomeHolderSet::Tag(tag) => generate_holder_set_nbt(Some(tag), &[]),
        BiomeHolderSet::Entries(entries) => generate_holder_set_nbt(None, entries),
    }
}

pub(super) fn generate_weighted_int_provider_nbt(provider: &WeightedIntProvider) -> TokenStream {
    let data = generate_int_provider_nbt(&provider.data);
    let weight = provider.weight;
    quote! {{
        let mut compound = NbtCompound::new();
        compound.insert("data", #data);
        compound.insert("weight", #weight);
        NbtTag::Compound(compound)
    }}
}

pub(super) fn generate_int_provider_nbt(provider: &IntProvider) -> TokenStream {
    match provider {
        IntProvider::Constant(value) => quote! { NbtTag::Int(#value) },
        IntProvider::Uniform {
            min_inclusive,
            max_inclusive,
        } => quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("type", "minecraft:uniform");
            compound.insert("min_inclusive", #min_inclusive);
            compound.insert("max_inclusive", #max_inclusive);
            NbtTag::Compound(compound)
        }},
        IntProvider::BiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("type", "minecraft:biased_to_bottom");
            compound.insert("min_inclusive", #min_inclusive);
            compound.insert("max_inclusive", #max_inclusive);
            NbtTag::Compound(compound)
        }},
        IntProvider::VeryBiasedToBottom {
            min_inclusive,
            max_inclusive,
        } => quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("type", "minecraft:very_biased_to_bottom");
            compound.insert("min_inclusive", #min_inclusive);
            compound.insert("max_inclusive", #max_inclusive);
            NbtTag::Compound(compound)
        }},
        IntProvider::Trapezoid { min, max, plateau } => quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("type", "minecraft:trapezoid");
            compound.insert("min", #min);
            compound.insert("max", #max);
            compound.insert("plateau", #plateau);
            NbtTag::Compound(compound)
        }},
        IntProvider::ClampedNormal {
            mean,
            deviation,
            min_inclusive,
            max_inclusive,
        } => quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("type", "minecraft:clamped_normal");
            compound.insert("mean", #mean);
            compound.insert("deviation", #deviation);
            compound.insert("min_inclusive", #min_inclusive);
            compound.insert("max_inclusive", #max_inclusive);
            NbtTag::Compound(compound)
        }},
        IntProvider::Clamped {
            source,
            min_inclusive,
            max_inclusive,
        } => {
            let source = generate_int_provider_nbt(source);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:clamped");
                compound.insert("source", #source);
                compound.insert("min_inclusive", #min_inclusive);
                compound.insert("max_inclusive", #max_inclusive);
                NbtTag::Compound(compound)
            }}
        }
        IntProvider::WeightedList { distribution } => {
            let distribution =
                generate_nbt_list(distribution.iter().map(generate_weighted_int_provider_nbt));
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:weighted_list");
                compound.insert("distribution", #distribution);
                NbtTag::Compound(compound)
            }}
        }
    }
}

pub(super) fn generate_vertical_anchor_nbt(anchor: VerticalAnchor) -> TokenStream {
    let (key, value) = match anchor {
        VerticalAnchor::Absolute(value) => ("absolute", value),
        VerticalAnchor::AboveBottom(value) => ("above_bottom", value),
        VerticalAnchor::BelowTop(value) => ("below_top", value),
        VerticalAnchor::RelativeToSeaLevel(value) => ("relative_to_sea_level", value),
    };
    quote! {{
        let mut compound = NbtCompound::new();
        compound.insert(#key, #value);
        NbtTag::Compound(compound)
    }}
}

pub(super) fn generate_offset_nbt(offset: &[i32; 3]) -> TokenStream {
    let [x, y, z] = *offset;
    quote! { NbtTag::IntArray(vec![#x, #y, #z]) }
}

pub(super) fn generate_block_predicate_nbt(predicate: &BlockPredicate) -> TokenStream {
    match predicate {
        BlockPredicate::True => quote! {{
            let mut compound = NbtCompound::new();
            compound.insert("type", "minecraft:true");
            NbtTag::Compound(compound)
        }},
        BlockPredicate::AllOf { predicates } => {
            let predicates = generate_nbt_list(predicates.iter().map(generate_block_predicate_nbt));
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:all_of");
                compound.insert("predicates", #predicates);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::AnyOf { predicates } => {
            let predicates = generate_nbt_list(predicates.iter().map(generate_block_predicate_nbt));
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:any_of");
                compound.insert("predicates", #predicates);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::Not { predicate } => {
            let predicate = generate_block_predicate_nbt(predicate);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:not");
                compound.insert("predicate", #predicate);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::MatchingBlockTag { tag, offset } => {
            let tag = tag.to_string();
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:matching_block_tag");
                #offset
                compound.insert("tag", #tag);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::MatchingBlocks { blocks, offset } => {
            let blocks = generate_block_holder_set_nbt(blocks);
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:matching_blocks");
                #offset
                compound.insert("blocks", #blocks);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::MatchingFluids { fluids, offset } => {
            let fluids = generate_fluid_holder_set_nbt(fluids);
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:matching_fluids");
                #offset
                compound.insert("fluids", #fluids);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::MatchingBiomes { biomes } => {
            let biomes = generate_biome_holder_set_nbt(biomes);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:matching_biomes");
                compound.insert("biomes", #biomes);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::Solid { offset } => {
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:solid");
                #offset
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::WouldSurvive { state, offset } => {
            let state = generate_block_state_data_codec_nbt(state);
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:would_survive");
                #offset
                compound.insert("state", #state);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::Replaceable { offset } => {
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:replaceable");
                #offset
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::HasSturdyFace { direction, offset } => {
            let direction = direction_name(*direction);
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:has_sturdy_face");
                #offset
                compound.insert("direction", #direction);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::InsideWorldBounds { offset } => {
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:inside_world_bounds");
                #offset
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::Unobstructed { offset } => {
            let offset = generate_predicate_offset(offset);

            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:unobstructed");
                #offset
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::HeightRange {
            min_inclusive,
            max_inclusive,
        } => {
            let min_inclusive = generate_vertical_anchor_nbt(*min_inclusive);
            let max_inclusive = generate_vertical_anchor_nbt(*max_inclusive);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:height_range");
                compound.insert("min_inclusive", #min_inclusive);
                compound.insert("max_inclusive", #max_inclusive);
                NbtTag::Compound(compound)
            }}
        }
        BlockPredicate::VolumeMatch { min, max, matches } => {
            let min = generate_offset_nbt(min);
            let max = generate_offset_nbt(max);
            let matches = generate_block_predicate_nbt(matches);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:volume_match");
                compound.insert("min", #min);
                compound.insert("max", #max);
                compound.insert("match", #matches);
                NbtTag::Compound(compound)
            }}
        }
    }
}

const fn direction_name(direction: steel_utils::Direction) -> &'static str {
    match direction {
        steel_utils::Direction::Down => "down",
        steel_utils::Direction::Up => "up",
        steel_utils::Direction::North => "north",
        steel_utils::Direction::South => "south",
        steel_utils::Direction::West => "west",
        steel_utils::Direction::East => "east",
    }
}

pub(super) fn generate_feature_noise_parameters_nbt(
    parameters: &FeatureNoiseParameters,
) -> TokenStream {
    // Exact `NormalNoise$Parameters.CODEC` order. `base_octave` is required;
    // every other field is `optionalFieldOf` and DFU omits defaulted values
    // on encode (`base_amplitude: 1.0`, `octave_count: 1`,
    // `normalize: ENABLED`, empty `amplitude_modifiers`).
    let mut fields = Vec::new();
    if parameters.base_amplitude != 1.0 {
        let base_amplitude = parameters.base_amplitude;
        fields.push(quote! { compound.insert("base_amplitude", #base_amplitude); });
    }
    let base_octave = parameters.base_octave;
    fields.push(quote! { compound.insert("base_octave", #base_octave); });
    if parameters.octave_count != 1 {
        let octave_count = parameters.octave_count;
        fields.push(quote! { compound.insert("octave_count", #octave_count); });
    }
    if !parameters.normalize {
        fields.push(quote! { compound.insert("normalize", NbtTag::Byte(0_i8)); });
    }
    if !parameters.amplitude_modifiers.is_empty() {
        let amplitude_modifiers = generate_nbt_list(
            parameters
                .amplitude_modifiers
                .iter()
                .map(|value| quote! { NbtTag::Double(#value) }),
        );
        fields.push(quote! { compound.insert("amplitude_modifiers", #amplitude_modifiers); });
    }
    quote! {{
        let mut compound = NbtCompound::new();
        #(#fields)*
        NbtTag::Compound(compound)
    }}
}

pub(crate) fn generate_block_state_provider_kind_nbt(
    provider: &BlockStateProviderKind,
) -> TokenStream {
    match provider {
        BlockStateProviderKind::Reference(id) => generate_identifier_nbt(id),
        BlockStateProviderKind::Simple { state } => generate_block_state_data_nbt(state),
        BlockStateProviderKind::Weighted { entries } => {
            let entries = generate_nbt_list(entries.iter().map(|entry| {
                let data = generate_block_state_data_codec_nbt(&entry.data);
                let weight = entry.weight;
                quote! {{
                    let mut compound = NbtCompound::new();
                    compound.insert("data", #data);
                    compound.insert("weight", #weight);
                    NbtTag::Compound(compound)
                }}
            }));
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:weighted");
                compound.insert("entries", #entries);
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::RotatedBlock { state, direction } => {
            let state = generate_block_state_provider_kind_nbt(state);
            let mut fields = vec![quote! { compound.insert("type", "minecraft:rotated"); }];
            fields.push(quote! { compound.insert("state", #state); });
            if let Some(direction) = direction {
                let direction = direction_name(*direction);
                fields.push(quote! { compound.insert("direction", #direction); });
            }
            quote! {{
                let mut compound = NbtCompound::new();
                #(#fields)*
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::RandomizedInt {
            property,
            source,
            values,
        } => {
            let source = generate_block_state_provider_kind_nbt(source);
            let values = generate_int_provider_nbt(values);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:randomized_int");
                compound.insert("source", #source);
                compound.insert("property", #property);
                compound.insert("values", #values);
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::RuleBased { fallback, rules } => {
            let rules = generate_nbt_list(rules.iter().map(|rule| {
                let if_true = generate_block_predicate_nbt(&rule.if_true);
                let then = generate_block_state_provider_kind_nbt(&rule.then);
                quote! {{
                    let mut compound = NbtCompound::new();
                    compound.insert("if_true", #if_true);
                    compound.insert("then", #then);
                    NbtTag::Compound(compound)
                }}
            }));
            let mut fields = vec![quote! { compound.insert("type", "minecraft:rule_based"); }];
            if let Some(fallback) = fallback {
                let fallback = generate_block_state_provider_kind_nbt(fallback);
                fields.push(quote! { compound.insert("fallback", #fallback); });
            }
            fields.push(quote! {
                compound.insert("rules", #rules);
            });
            quote! {{
                let mut compound = NbtCompound::new();
                #(#fields)*
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::Noise(provider) => {
            let noise = generate_feature_noise_parameters_nbt(&provider.noise);
            let scale = provider.scale;
            let seed = provider.seed;
            let states = generate_nbt_list(
                provider
                    .states
                    .iter()
                    .map(generate_block_state_data_codec_nbt),
            );
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:noise");
                compound.insert("seed", #seed);
                compound.insert("noise", #noise);
                compound.insert("scale", #scale);
                compound.insert("states", #states);
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::NoiseThreshold(provider) => {
            let noise = generate_feature_noise_parameters_nbt(&provider.noise);
            let scale = provider.scale;
            let seed = provider.seed;
            let threshold = provider.threshold;
            let high_chance = provider.high_chance;
            let default_state = generate_block_state_data_codec_nbt(&provider.default_state);
            let low_states = generate_nbt_list(
                provider
                    .low_states
                    .iter()
                    .map(generate_block_state_data_codec_nbt),
            );
            let high_states = generate_nbt_list(
                provider
                    .high_states
                    .iter()
                    .map(generate_block_state_data_codec_nbt),
            );
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:noise_threshold");
                compound.insert("seed", #seed);
                compound.insert("noise", #noise);
                compound.insert("scale", #scale);
                compound.insert("threshold", #threshold);
                compound.insert("high_chance", #high_chance);
                compound.insert("default_state", #default_state);
                compound.insert("low_states", #low_states);
                compound.insert("high_states", #high_states);
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::DualNoise(provider) => {
            let noise = generate_feature_noise_parameters_nbt(&provider.noise);
            let scale = provider.scale;
            let seed = provider.seed;
            let slow_noise = generate_feature_noise_parameters_nbt(&provider.slow_noise);
            let slow_scale = provider.slow_scale;
            let states = generate_nbt_list(
                provider
                    .states
                    .iter()
                    .map(generate_block_state_data_codec_nbt),
            );
            // `InclusiveRange.codec` encodes a degenerate range as the bare
            // point and anything else as a 2-element int list (a Java
            // `List`, i.e. an NBT int list — never an int array).
            let [variety_min, variety_max] = provider.variety;
            let variety = if variety_min == variety_max {
                quote! { NbtTag::Int(#variety_min) }
            } else {
                quote! { NbtTag::List(NbtList::Int(vec![#variety_min, #variety_max])) }
            };
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:dual_noise");
                compound.insert("variety", #variety);
                compound.insert("slow_noise", #slow_noise);
                compound.insert("slow_scale", #slow_scale);
                compound.insert("seed", #seed);
                compound.insert("noise", #noise);
                compound.insert("scale", #scale);
                compound.insert("states", #states);
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::CopyProperties { source } => {
            let source = generate_block_state_provider_kind_nbt(source);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:copy_properties");
                compound.insert("source", #source);
                NbtTag::Compound(compound)
            }}
        }
        BlockStateProviderKind::RandomBlock { blocks } => {
            let blocks = generate_block_holder_set_nbt(blocks);
            quote! {{
                let mut compound = NbtCompound::new();
                compound.insert("type", "minecraft:random_block");
                compound.insert("blocks", #blocks);
                NbtTag::Compound(compound)
            }}
        }
    }
}
