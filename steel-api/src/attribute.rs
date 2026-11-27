//! Аттрибут-API — `Attribute`, `AttributeModifier`, `EntityAttribute` как в Paper.
//! Плагин меняет `generic.max_health`, `generic.movement_speed`, кастомные аттрибуты.

use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Attribute {
    MaxHealth, FollowRange, KnockbackResistance, MovementSpeed, AttackDamage, AttackKnockback, AttackSpeed, Armor, ArmorToughness, Luck, JumpStrength, SpawnReinforcements,
    Custom(u32),
}

impl Attribute {
    pub fn key(&self) -> &'static str {
        match self {
            Self::MaxHealth => "generic.max_health",
            Self::MovementSpeed => "generic.movement_speed",
            Self::AttackDamage => "generic.attack_damage",
            Self::Armor => "generic.armor",
            _ => "generic.custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    AddValue, AddMultipliedBase, AddMultipliedTotal,
}

#[derive(Debug, Clone)]
pub struct AttributeModifier {
    pub id: String,
    pub attribute: Attribute,
    pub operation: Operation,
    pub amount: f64,
    pub slot: Option<String>,
}

impl AttributeModifier {
    pub fn new(id: impl Into<String>, attr: Attribute, op: Operation, amount: f64) -> Self {
        Self { id: id.into(), attribute: attr, operation: op, amount, slot: None }
    }
    pub fn slot(mut self, s: impl Into<String>) -> Self { self.slot=Some(s.into()); self }
}

#[derive(Debug, Clone, Default)]
pub struct AttributeInstance {
    pub base: f64,
    pub modifiers: FxHashMap<String, AttributeModifier>,
}

impl AttributeInstance {
    pub fn new(base: f64) -> Self { Self { base, modifiers: Default::default() } }
    pub fn add_modifier(&mut self, m: AttributeModifier) { self.modifiers.insert(m.id.clone(), m); }
    pub fn remove_modifier(&mut self, id: &str) -> Option<AttributeModifier> { self.modifiers.remove(id) }
    pub fn value(&self) -> f64 {
        let mut v=self.base;
        for m in self.modifiers.values() { if m.operation==Operation::AddValue { v+=m.amount; } }
        let mut mul_base=0.0;
        for m in self.modifiers.values() { if m.operation==Operation::AddMultipliedBase { mul_base+=m.amount; } }
        v += self.base * mul_base;
        let mut mul_total=1.0;
        for m in self.modifiers.values() { if m.operation==Operation::AddMultipliedTotal { mul_total*=(1.0+m.amount); } }
        v * mul_total
    }
}

pub trait Attributable: Send + Sync {
    fn get_attribute(&self, attr: Attribute) -> Option<AttributeInstance>;
    fn set_attribute(&self, attr: Attribute, instance: AttributeInstance) -> bool;
    fn add_modifier(&self, modifier: AttributeModifier) -> bool;
    fn remove_modifier(&self, attr: Attribute, id: &str) -> bool;
}
