use crate::advancement::{Advancement, positioner};
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::io::Write;
use steel_utils::Identifier;
use steel_utils::serial::WriteTo;

pub type AdvancementRef = &'static Advancement;

#[derive(Debug)]
pub struct AdvancementNode {
    children: Vec<usize>,
    parent: Option<usize>,
    value: AdvancementRef,
    x: f32,
    y: f32,
}

pub type AdvancementNodeRef = &'static AdvancementNode;

impl WriteTo for AdvancementNodeRef {
    fn write(&self, writer: &mut impl Write) -> std::io::Result<()> {
        self.value.write(writer)?;
        self.x.write(writer)?;
        self.y.write(writer)
    }
}

impl AdvancementNode {
    pub fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    #[must_use]
    pub const fn new(value: AdvancementRef, parent: Option<usize>) -> Self {
        Self {
            value,
            parent,
            children: Vec::new(),
            x: 0f32,
            y: 0f32,
        }
    }

    #[inline]
    #[must_use]
    pub const fn has_display(&self) -> bool {
        self.value.display.is_some()
    }

    #[inline]
    pub const fn set_location(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    #[inline]
    #[must_use]
    pub const fn get_location(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[must_use]
    #[inline]
    pub const fn value(&self) -> AdvancementRef {
        self.value
    }

    #[must_use]
    #[inline]
    pub fn children(&self) -> Vec<usize> {
        self.children.clone()
    }

    #[must_use]
    #[inline]
    pub const fn parent(&self) -> Option<usize> {
        self.parent
    }
}

impl PartialEq<Self> for AdvancementNode {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for AdvancementNode {}

impl Display for AdvancementNode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.value.key)
    }
}
impl Hash for AdvancementNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.key.hash(state);
    }
}

/// Equivalent of the `AdvancementTree` of the Minecraft source code.
///
/// Holds `'static` handles, so runtime-registered advancements are leaked once by
/// [`AdvancementRegistry::register_owned`]. Vanilla never unloads an advancement
/// tree either: `/reload` builds a fresh `ServerAdvancementManager` and swaps it in,
/// which this registry models with [`AdvancementRegistry::reload`].
#[derive(Default, Debug)]
pub struct AdvancementRegistry {
    pub(crate) advancements: Vec<AdvancementRef>,
    pub(crate) adv_nodes: Vec<AdvancementNode>,
    pub(crate) unloaded_advancement: FxHashSet<AdvancementRef>,
    pub(crate) by_key: FxHashMap<Identifier, usize>,
    pub(crate) roots: Vec<usize>,
    pub(crate) tasks: Vec<usize>,
}

impl AdvancementRegistry {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn advancements(&self) -> &[AdvancementRef] {
        &self.advancements
    }

    #[must_use]
    pub fn nodes(&self) -> &[AdvancementNode] {
        &self.adv_nodes
    }

    fn register_with_parent(&mut self, advancement: AdvancementRef, parent_idx: Option<usize>) {
        let id = advancement.key.clone();
        let node_idx = self.adv_nodes.len();
        self.adv_nodes
            .push(AdvancementNode::new(advancement, parent_idx));
        self.advancements.push(advancement);
        self.by_key.insert(id, node_idx);
        if let Some(parent) = parent_idx {
            let parent_node = self
                .adv_nodes
                .get_mut(parent)
                .expect("unable to get the parent node");
            parent_node.add_child(node_idx);
            self.tasks.push(node_idx);
        } else {
            self.roots.push(node_idx);
        }
    }

    fn try_register(&mut self, advancement: AdvancementRef) -> Option<AdvancementRef> {
        let parent_id = &advancement.parent;
        let parent_idx: Option<usize> = match parent_id {
            Some(id) => match self.by_key.get(id) {
                Some(node) => Some(*node),
                None => return Some(advancement),
            },
            None => None,
        };
        self.register_with_parent(advancement, parent_idx);
        None
    }

    fn register_all(&mut self, mut advancements_to_add: Vec<AdvancementRef>) {
        while !advancements_to_add.is_empty() {
            let len_before = advancements_to_add.len();

            advancements_to_add = advancements_to_add
                .into_iter()
                .filter_map(|advancement| self.try_register(advancement))
                .collect();

            if advancements_to_add.len() == len_before && !advancements_to_add.is_empty() {
                eprintln!(
                    "Couldn't load advancements: {:?}",
                    advancements_to_add
                        .iter()
                        .map(|a| &a.key)
                        .collect::<Vec<_>>()
                );
                break;
            }
        }
    }

    /// Registers a statically known advancement in the registry. Returns true if the advancement
    /// was added to the unloaded advancements set, false if it was already present.
    pub fn register(&mut self, advancement: AdvancementRef) -> bool {
        self.unloaded_advancement.insert(advancement)
    }

    /// Registers a runtime-built advancement (a data pack loaded at run time).
    ///
    /// `AdvancementRef` is `&'static`, so the value is leaked exactly once, which is
    /// what vanilla does implicitly by keeping the manager alive for the server's
    /// lifetime. Returns `true` when the key was previously unknown.
    pub fn register_owned(&mut self, advancement: Advancement) -> bool {
        let key = advancement.key.clone();
        if self.by_key.contains_key(&key)
            || self
                .unloaded_advancement
                .iter()
                .any(|existing| existing.key == key)
        {
            return false;
        }
        let leaked: AdvancementRef = Box::leak(Box::new(advancement));
        self.unloaded_advancement.insert(leaked)
    }

    /// Drops every tree and re-registers the supplied set, mirroring the
    /// `ServerAdvancementManager` rebuild that `/reload` performs.
    ///
    /// Earlier `'static` handles stay valid (they are leaked by construction), so
    /// player progress that references them keeps working.
    pub fn reload(&mut self, advancements: Vec<AdvancementRef>) {
        self.advancements.clear();
        self.adv_nodes.clear();
        self.by_key.clear();
        self.roots.clear();
        self.tasks.clear();
        self.unloaded_advancement.clear();
        for advancement in advancements {
            self.unloaded_advancement.insert(advancement);
        }
        self.update_tree();
    }

    #[cfg(test)]
    pub fn register_without_load(&mut self, advancements: &[AdvancementRef]) {
        self.register_all(advancements.to_vec());
    }

    #[must_use]
    #[inline]
    pub fn value_by_key(&self, key: &Identifier) -> Option<AdvancementRef> {
        self.by_key.get(key).map(|idx| self.advancements[*idx])
    }

    #[must_use]
    #[inline]
    pub fn by_key(&self, key: &Identifier) -> Option<&AdvancementNode> {
        self.by_key
            .get(key)
            .and_then(|idx| self.adv_nodes.get(*idx))
    }

    #[must_use]
    #[inline]
    pub fn get_by_idx(&self, idx: usize) -> Option<&AdvancementNode> {
        self.adv_nodes.get(idx)
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &AdvancementNode)> + '_ {
        self.adv_nodes.iter().enumerate()
    }

    /// Updates the advancement tree by loading and connecting all unloaded advancements.
    pub(crate) fn update_tree(&mut self) {
        let advancements = self.unloaded_advancement.drain().collect();
        self.register_all(advancements);
        for advancement_idx in self.roots.clone() {
            let node = self.adv_nodes.get(advancement_idx);
            let Some(node) = node else {
                eprintln!("unable to get the root node with index {advancement_idx}");
                return;
            };
            if node.has_display() {
                let res = positioner::run(self, advancement_idx);
                if let Err(e) = res {
                    eprintln!("{}", e.get_message());
                }
            }
        }
    }

    #[must_use]
    pub fn root(&'static self, advancement_node: AdvancementNodeRef) -> AdvancementNodeRef {
        let mut advancement_node = advancement_node;
        while let Some(parent) = &advancement_node.parent {
            advancement_node = &self.adv_nodes[*parent];
        }
        advancement_node
    }

    pub fn remove_unloaded_advancement(&mut self, advancement: &Identifier) {
        self.unloaded_advancement
            .retain(|adv| adv.key() != advancement);
    }
}
