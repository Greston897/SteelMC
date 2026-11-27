//! Рецепт-API — как `org.bukkit.inventory.Recipe` + Paper `RecipeManager`.
//! Плагин регистрирует кастомные крафты, печку, кузню, зельеварку.

use rustc_hash::FxHashMap;
use steel_utils::Identifier;

#[derive(Debug, Clone)]
pub struct Ingredient {
    pub items: Vec<Identifier>,
    pub count: i32,
}

impl Ingredient {
    pub fn of(item: Identifier) -> Self { Self { items: vec![item], count: 1 } }
    pub fn any(items: Vec<Identifier>) -> Self { Self { items, count: 1 } }
    pub fn with_count(mut self, c: i32) -> Self { self.count = c; self }
    pub fn matches(&self, id: &Identifier) -> bool { self.items.contains(id) }
}

#[derive(Debug, Clone)]
pub struct ShapedRecipe {
    pub key: Identifier,
    pub result: Identifier,
    pub result_count: i32,
    pub pattern: Vec<String>,
    pub ingredients: rustc_hash::FxHashMap<char, Ingredient>,
    pub group: Option<String>,
}

impl ShapedRecipe {
    pub fn new(key: Identifier, result: Identifier) -> Self {
        Self { key, result, result_count: 1, pattern: Vec::new(), ingredients: Default::default(), group: None }
    }
    pub fn pattern(mut self, rows: &[&str]) -> Self { self.pattern = rows.iter().map(|s| s.to_string()).collect(); self }
    pub fn ingredient(mut self, c: char, ing: Ingredient) -> Self { self.ingredients.insert(c, ing); self }
    pub fn group(mut self, g: impl Into<String>) -> Self { self.group = Some(g.into()); self }
}

#[derive(Debug, Clone)]
pub struct ShapelessRecipe {
    pub key: Identifier,
    pub result: Identifier,
    pub result_count: i32,
    pub ingredients: Vec<Ingredient>,
    pub group: Option<String>,
}

impl ShapelessRecipe {
    pub fn new(key: Identifier, result: Identifier) -> Self { Self { key, result, result_count: 1, ingredients: Vec::new(), group: None } }
    pub fn add_ingredient(mut self, ing: Ingredient) -> Self { self.ingredients.push(ing); self }
}

#[derive(Debug, Clone)]
pub struct FurnaceRecipe {
    pub key: Identifier,
    pub input: Ingredient,
    pub result: Identifier,
    pub experience: f32,
    pub cook_time: i32,
}

#[derive(Debug, Clone)]
pub enum Recipe {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
    Furnace(FurnaceRecipe),
    Blasting(FurnaceRecipe),
    Smoking(FurnaceRecipe),
    Campfire(FurnaceRecipe),
    Smithing { key: Identifier, base: Ingredient, addition: Ingredient, result: Identifier },
    Stonecutting { key: Identifier, input: Ingredient, result: Identifier, count: i32 },
}

impl Recipe {
    pub fn key(&self) -> &Identifier {
        match self {
            Self::Shaped(r) => &r.key,
            Self::Shapeless(r) => &r.key,
            Self::Furnace(r) => &r.key,
            Self::Blasting(r) => &r.key,
            Self::Smoking(r) => &r.key,
            Self::Campfire(r) => &r.key,
            Self::Smithing { key, .. } => key,
            Self::Stonecutting { key, .. } => key,
        }
    }
}

pub struct RecipeManager {
    recipes: steel_utils::locks::SyncRwLock<rustc_hash::FxHashMap<Identifier, Recipe>>,
}

impl Default for RecipeManager { fn default() -> Self { Self::new() } }

impl RecipeManager {
    pub fn new() -> Self { Self { recipes: steel_utils::locks::SyncRwLock::new(Default::default()) } }
    pub fn register(&self, recipe: Recipe) -> bool {
        let k=recipe.key().clone();
        let mut w=self.recipes.write();
        if w.contains_key(&k) { return false; }
        w.insert(k, recipe); true
    }
    pub fn unregister(&self, key: &Identifier) -> bool { self.recipes.write().remove(key).is_some() }
    pub fn get(&self, key: &Identifier) -> Option<Recipe> { self.recipes.read().get(key).cloned() }
    pub fn all(&self) -> Vec<Recipe> { self.recipes.read().values().cloned().collect() }
    pub fn clear(&self) { self.recipes.write().clear(); }
    pub fn count(&self) -> usize { self.recipes.read().len() }
}
