//! Vanilla MC 26.3 `minecraft:sulfur_cube_archetype` synchronized registry.
//!
//! Mirrors `net.minecraft.world.entity.animal.sulfurcube.SulfurCubeArchetype`
//! (`RegistryDataLoader.SYNCHRONIZED_REGISTRIES`). Defines the 12 built-in
//! sulfur cube archetypes (`bouncy`, `explosive`, `fast_flat`, `fast_sliding`,
//! `high_resistance`, `hot`, `light`, `regular`, `slow_bouncy`, `slow_flat`,
//! `slow_sliding`, `sticky`) with bidirectional NBT codecs for network sync
//! and custom datapacks.

use std::str::FromStr;

use rustc_hash::FxHashMap;
use simdnbt::owned::{NbtCompound, NbtList, NbtTag};
use simdnbt::{FromNbtTag, ToNbtTag};
use steel_utils::Identifier;
use steel_utils::nbt::NbtNumeric as _;
use steel_utils::value_providers::FloatProvider;

use crate::attribute::AttributeModifierOperation;
use crate::items::ItemRef;
use crate::{REGISTRY, RegistryTags, TaggedRegistryExt};

/// Items accepted by a sulfur cube archetype (`HolderSet<Item>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SulfurCubeItemSet {
    /// Tag reference (`#<namespace>:<path>`).
    Tag(Identifier),
    /// Explicit list of item identifiers.
    Items(Vec<Identifier>),
}

impl SulfurCubeItemSet {
    /// Returns whether `item` activates this archetype.
    #[must_use]
    pub fn contains(&self, item: ItemRef) -> bool {
        match self {
            Self::Tag(tag) => REGISTRY.items.is_in_tag(item, tag),
            Self::Items(keys) => keys.iter().any(|k| k == &item.key),
        }
    }

    #[must_use]
    pub fn to_nbt_tag_ref(&self) -> NbtTag {
        match self {
            Self::Tag(tag) => NbtTag::String(format!("#{tag}").into()),
            Self::Items(items) => {
                if items.len() == 1 {
                    NbtTag::String(items[0].to_string().into())
                } else {
                    NbtTag::List(NbtList::String(
                        items.iter().map(|id| id.to_string().into()).collect(),
                    ))
                }
            }
        }
    }

    #[must_use]
    pub fn from_borrowed_nbt(tag: simdnbt::borrow::NbtTag<'_, '_>) -> Option<Self> {
        if let Some(s) = tag.string() {
            let text = s.to_str();
            if let Some(raw_tag) = text.strip_prefix('#') {
                return Some(Self::Tag(Identifier::from_str(raw_tag).ok()?));
            }
            return Some(Self::Items(vec![Identifier::from_str(&text).ok()?]));
        }
        let list = tag.list()?.strings()?;
        let mut items = Vec::with_capacity(list.len());
        for raw in list {
            items.push(Identifier::from_str(&raw.to_str()).ok()?);
        }
        Some(Self::Items(items))
    }
}

/// Attribute modifier entry inside `SulfurCubeArchetype.attribute_modifiers`.
#[derive(Debug, Clone, PartialEq)]
pub struct SulfurCubeAttributeModifier {
    pub attribute: Identifier,
    pub id: Identifier,
    pub amount: f64,
    pub operation: AttributeModifierOperation,
}

impl SulfurCubeAttributeModifier {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert("attribute", self.attribute.to_string().as_str());
        compound.insert("id", self.id.to_string().as_str());
        compound.insert("amount", self.amount);
        compound.insert("operation", self.operation.name());
        compound
    }

    #[must_use]
    pub fn from_borrowed_compound(compound: simdnbt::borrow::NbtCompound<'_, '_>) -> Option<Self> {
        let attribute =
            Identifier::from_str(&compound.get("attribute")?.string()?.to_str()).ok()?;
        let id = Identifier::from_str(&compound.get("id")?.string()?.to_str()).ok()?;
        let amount = compound.get("amount")?.codec_f64()?;
        let op_str = compound.get("operation")?.string()?.to_str();
        let operation = AttributeModifierOperation::by_name(&op_str)?;
        Some(Self {
            attribute,
            id,
            amount,
            operation,
        })
    }
}

/// Optional explosion payload for explosive sulfur cube archetypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SulfurCubeExplosion {
    pub fuse: i32,
    pub power: i32,
    pub causes_fire: bool,
}

impl SulfurCubeExplosion {
    #[must_use]
    pub fn to_nbt_compound(self) -> NbtCompound {
        // Exact `ExplosionData.CODEC` order: power, causes_fire, fuse.
        let mut compound = NbtCompound::new();
        compound.insert("power", self.power);
        compound.insert("causes_fire", self.causes_fire);
        compound.insert("fuse", self.fuse);
        compound
    }

    #[must_use]
    pub fn from_borrowed_compound(compound: simdnbt::borrow::NbtCompound<'_, '_>) -> Option<Self> {
        let fuse = compound.get("fuse")?.codec_i32()?;
        let power = compound.get("power")?.codec_i32()?;
        let causes_fire = compound.get("causes_fire")?.codec_bool()?;
        (fuse >= 1 && power >= 0).then_some(Self {
            fuse,
            power,
            causes_fire,
        })
    }
}

/// Optional contact damage payload (e.g. `minecraft:hot` -> `sulfur_cube_hot`).
#[derive(Debug, Clone)]
pub struct SulfurCubeContactDamage {
    pub damage_type: Identifier,
    pub amount: FloatProvider,
    pub attribute_to_source: bool,
}

impl SulfurCubeContactDamage {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert("damage_type", self.damage_type.to_string().as_str());
        compound.insert("amount", float_provider_to_nbt(self.amount));
        compound.insert("attribute_to_source", self.attribute_to_source);
        compound
    }

    #[must_use]
    pub fn from_borrowed_compound(compound: simdnbt::borrow::NbtCompound<'_, '_>) -> Option<Self> {
        let damage_type =
            Identifier::from_str(&compound.get("damage_type")?.string()?.to_str()).ok()?;
        let amount = float_provider_from_nbt(compound.get("amount")?)?;
        let attribute_to_source = compound.get("attribute_to_source")?.codec_bool()?;
        Some(Self {
            damage_type,
            amount,
            attribute_to_source,
        })
    }
}

fn float_provider_to_nbt(provider: FloatProvider) -> NbtTag {
    match provider {
        FloatProvider::Constant(v) => NbtTag::Float(v),
        FloatProvider::Uniform {
            min_inclusive,
            max_exclusive,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:uniform");
            c.insert("min_inclusive", min_inclusive);
            c.insert("max_exclusive", max_exclusive);
            NbtTag::Compound(c)
        }
        FloatProvider::Trapezoid { min, max, plateau } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:trapezoid");
            c.insert("min", min);
            c.insert("max", max);
            c.insert("plateau", plateau);
            NbtTag::Compound(c)
        }
        FloatProvider::ClampedNormal {
            mean,
            deviation,
            min,
            max,
        } => {
            let mut c = NbtCompound::new();
            c.insert("type", "minecraft:clamped_normal");
            c.insert("mean", mean);
            c.insert("deviation", deviation);
            c.insert("min", min);
            c.insert("max", max);
            NbtTag::Compound(c)
        }
    }
}

fn float_provider_from_nbt(tag: simdnbt::borrow::NbtTag<'_, '_>) -> Option<FloatProvider> {
    if let Some(value) = tag.codec_f32() {
        return value.is_finite().then_some(FloatProvider::Constant(value));
    }
    let compound = tag.compound()?;
    let kind = compound.get("type")?.string()?.to_str();
    let kind = kind.strip_prefix("minecraft:").unwrap_or(&kind);
    match kind {
        "constant" => Some(FloatProvider::Constant(
            compound.get("value")?.codec_f32()?,
        )),
        "uniform" => Some(FloatProvider::Uniform {
            min_inclusive: compound.get("min_inclusive")?.codec_f32()?,
            max_exclusive: compound.get("max_exclusive")?.codec_f32()?,
        }),
        "trapezoid" => Some(FloatProvider::Trapezoid {
            min: compound.get("min")?.codec_f32()?,
            max: compound.get("max")?.codec_f32()?,
            plateau: compound.get("plateau")?.codec_f32()?,
        }),
        "clamped_normal" => Some(FloatProvider::ClampedNormal {
            mean: compound.get("mean")?.codec_f32()?,
            deviation: compound.get("deviation")?.codec_f32()?,
            min: compound.get("min")?.codec_f32()?,
            max: compound.get("max")?.codec_f32()?,
        }),
        _ => None,
    }
}

/// Knockback scale applied when striking or pushing a sulfur cube.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SulfurCubeKnockbackModifiers {
    pub horizontal_power: f32,
    pub vertical_power: f32,
}

impl SulfurCubeKnockbackModifiers {
    #[must_use]
    pub fn to_nbt_compound(self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert("horizontal_power", self.horizontal_power);
        compound.insert("vertical_power", self.vertical_power);
        compound
    }

    #[must_use]
    pub fn from_borrowed_compound(compound: simdnbt::borrow::NbtCompound<'_, '_>) -> Option<Self> {
        Some(Self {
            horizontal_power: compound.get("horizontal_power")?.codec_f32()?,
            vertical_power: compound.get("vertical_power")?.codec_f32()?,
        })
    }
}

/// Hit and push sound parameters of a sulfur cube archetype.
#[derive(Debug, Clone, PartialEq)]
pub struct SulfurCubeSoundSettings {
    pub hit_sound: Identifier,
    pub push_sound: Identifier,
    pub push_sound_impulse_threshold: f32,
    pub push_sound_cooldown: f32,
}

impl SulfurCubeSoundSettings {
    #[must_use]
    pub fn to_nbt_compound(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.insert("hit_sound", self.hit_sound.to_string().as_str());
        compound.insert("push_sound", self.push_sound.to_string().as_str());
        compound.insert(
            "push_sound_impulse_threshold",
            self.push_sound_impulse_threshold,
        );
        compound.insert("push_sound_cooldown", self.push_sound_cooldown);
        compound
    }

    #[must_use]
    pub fn from_borrowed_compound(compound: simdnbt::borrow::NbtCompound<'_, '_>) -> Option<Self> {
        Some(Self {
            hit_sound: Identifier::from_str(&compound.get("hit_sound")?.string()?.to_str()).ok()?,
            push_sound: Identifier::from_str(&compound.get("push_sound")?.string()?.to_str())
                .ok()?,
            push_sound_impulse_threshold: compound
                .get("push_sound_impulse_threshold")?
                .codec_f32()?,
            push_sound_cooldown: compound.get("push_sound_cooldown")?.codec_f32()?,
        })
    }
}

/// Complete registry-independent value for a sulfur cube archetype.
#[derive(Debug, Clone)]
pub struct SulfurCubeArchetypeValue {
    pub items: SulfurCubeItemSet,
    pub buoyant: bool,
    pub explosion: Option<SulfurCubeExplosion>,
    pub contact_damage: Option<SulfurCubeContactDamage>,
    pub knockback_modifiers: SulfurCubeKnockbackModifiers,
    pub attribute_modifiers: Vec<SulfurCubeAttributeModifier>,
    pub sound_settings: SulfurCubeSoundSettings,
}

impl SulfurCubeArchetypeValue {
    #[must_use]
    pub fn to_nbt_tag_ref(&self) -> NbtTag {
        // Exact `SulfurCubeArchetype.DIRECT_CODEC` field order. DFU
        // `optionalFieldOf` skips values equal to the default on encode, so
        // `buoyant: false` is omitted; true optionals (`explosion`,
        // `contact_damage`) are omitted when absent.
        let mut compound = NbtCompound::new();
        compound.insert("items", self.items.to_nbt_tag_ref());
        let modifiers: Vec<NbtCompound> = self
            .attribute_modifiers
            .iter()
            .map(SulfurCubeAttributeModifier::to_nbt_compound)
            .collect();
        compound.insert(
            "attribute_modifiers",
            NbtTag::List(NbtList::Compound(modifiers)),
        );
        if self.buoyant {
            compound.insert("buoyant", true);
        }
        if let Some(explosion) = self.explosion {
            compound.insert("explosion", NbtTag::Compound(explosion.to_nbt_compound()));
        }
        if let Some(contact_damage) = &self.contact_damage {
            compound.insert(
                "contact_damage",
                NbtTag::Compound(contact_damage.to_nbt_compound()),
            );
        }
        compound.insert(
            "knockback_modifiers",
            NbtTag::Compound(self.knockback_modifiers.to_nbt_compound()),
        );
        compound.insert(
            "sound_settings",
            NbtTag::Compound(self.sound_settings.to_nbt_compound()),
        );
        NbtTag::Compound(compound)
    }
}

impl ToNbtTag for SulfurCubeArchetypeValue {
    fn to_nbt_tag(self) -> NbtTag {
        self.to_nbt_tag_ref()
    }
}

impl FromNbtTag for SulfurCubeArchetypeValue {
    fn from_nbt_tag(tag: simdnbt::borrow::NbtTag<'_, '_>) -> Option<Self> {
        let compound = tag.compound()?;
        let items = SulfurCubeItemSet::from_borrowed_nbt(compound.get("items")?)?;
        let buoyant = match compound.get("buoyant") {
            Some(tag) => tag.codec_bool()?,
            None => false,
        };
        let explosion = match compound.get("explosion") {
            Some(tag) => Some(SulfurCubeExplosion::from_borrowed_compound(tag.compound()?)?),
            None => None,
        };
        let contact_damage = match compound.get("contact_damage") {
            Some(tag) => Some(SulfurCubeContactDamage::from_borrowed_compound(
                tag.compound()?,
            )?),
            None => None,
        };
        let knockback_modifiers = SulfurCubeKnockbackModifiers::from_borrowed_compound(
            compound.get("knockback_modifiers")?.compound()?,
        )?;
        let attr_list = compound.get("attribute_modifiers")?.list()?;
        let mut attribute_modifiers = Vec::new();
        if !attr_list.empty() {
            for entry in attr_list.compounds()? {
                attribute_modifiers.push(SulfurCubeAttributeModifier::from_borrowed_compound(
                    entry,
                )?);
            }
        }
        let sound_settings = SulfurCubeSoundSettings::from_borrowed_compound(
            compound.get("sound_settings")?.compound()?,
        )?;
        Some(Self {
            items,
            buoyant,
            explosion,
            contact_damage,
            knockback_modifiers,
            attribute_modifiers,
            sound_settings,
        })
    }
}

/// Registered `minecraft:sulfur_cube_archetype` definition.
#[derive(Debug, Clone)]
pub struct SulfurCubeArchetype {
    pub key: Identifier,
    pub value: SulfurCubeArchetypeValue,
}

impl ToNbtTag for &SulfurCubeArchetype {
    fn to_nbt_tag(self) -> NbtTag {
        self.value.to_nbt_tag_ref()
    }
}

pub type SulfurCubeArchetypeRef = &'static SulfurCubeArchetype;

pub struct SulfurCubeArchetypeRegistry {
    archetypes_by_id: Vec<SulfurCubeArchetypeRef>,
    archetypes_by_key: FxHashMap<Identifier, usize>,
    tags: RegistryTags,
    allows_registering: bool,
}

impl SulfurCubeArchetypeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            archetypes_by_id: Vec::new(),
            archetypes_by_key: FxHashMap::default(),
            tags: RegistryTags::default(),
            allows_registering: true,
        }
    }

    /// Resolves the first registered archetype whose `items` holder-set contains `item`.
    #[must_use]
    pub fn find_for_item(&self, item: ItemRef) -> Option<SulfurCubeArchetypeRef> {
        self.archetypes_by_id
            .iter()
            .copied()
            .find(|arch| arch.value.items.contains(item))
    }
}

crate::impl_standard_methods!(
    SulfurCubeArchetypeRegistry,
    SulfurCubeArchetypeRef,
    archetypes_by_id,
    archetypes_by_key,
    allows_registering
);

crate::impl_registry!(
    SulfurCubeArchetypeRegistry,
    SulfurCubeArchetype,
    archetypes_by_id,
    archetypes_by_key,
    sulfur_cube_archetypes
);

crate::impl_tagged_registry!(
    SulfurCubeArchetypeRegistry,
    archetypes_by_key,
    "sulfur_cube_archetype"
);
