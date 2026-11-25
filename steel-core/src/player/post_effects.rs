//! Vanilla `ServerPlayer` post-effect bookkeeping.
//!
//! `post_effects` is an ordered, duplicate-free list of `minecraft:post_effect`
//! registry ids the client applies to the whole screen, plus the dirty flag that
//! makes the play tick push them through `ClientboundPostEffectsPacket`.

use steel_protocol::packets::game::{CPostEffects, PostEffect};
use steel_utils::Identifier;

/// Vanilla `ServerPlayer.postEffects` + `postEffectsDirty`.
#[derive(Debug, Clone, Default)]
pub struct PlayerPostEffects {
    effects: Vec<Identifier>,
    dirty: bool,
}

impl PlayerPostEffects {
    /// An empty list that still needs one push, matching vanilla's
    /// `postEffectsDirty = true` initial value.
    ///
    /// Vanilla starts dirty so `PlayerList.placeNewPlayer` always sends the
    /// (possibly empty) list once, which is how the client clears a stale
    /// pipeline after a reconnect.
    #[must_use]
    pub const fn new_pending() -> Self {
        Self {
            effects: Vec::new(),
            dirty: true,
        }
    }

    /// The active effects, in application order.
    #[must_use]
    pub fn effects(&self) -> &[Identifier] {
        &self.effects
    }

    /// Vanilla `ServerPlayer.addPostEffect`: no-op for a duplicate id.
    pub fn add(&mut self, effect: Identifier) -> bool {
        if self.effects.contains(&effect) {
            return false;
        }
        self.effects.push(effect);
        self.dirty = true;
        true
    }

    /// Vanilla `ServerPlayer.removePostEffect`.
    pub fn remove(&mut self, effect: &Identifier) -> bool {
        let Some(index) = self.effects.iter().position(|current| current == effect) else {
            return false;
        };
        self.effects.remove(index);
        self.dirty = true;
        true
    }

    /// Vanilla `ServerPlayer.clearPostEffects`.
    pub fn clear(&mut self) -> bool {
        if self.effects.is_empty() {
            return false;
        }
        self.effects.clear();
        self.dirty = true;
        true
    }

    /// Replaces the whole list, as `PlayerList.placeNewPlayer` restores a save.
    pub fn replace(&mut self, effects: Vec<Identifier>) {
        let mut unique = Vec::with_capacity(effects.len());
        for effect in effects {
            if !unique.contains(&effect) {
                unique.push(effect);
            }
        }
        if unique != self.effects {
            self.effects = unique;
            self.dirty = true;
        }
    }

    /// Takes the packet to send if the list changed since the last push.
    ///
    /// Vanilla `ServerPlayer.tick` checks the flag after the advancement flush:
    /// `if (this.postEffectsDirty) { this.sendPostEffects(); }`.
    pub fn take_update(&mut self) -> Option<CPostEffects> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        Some(post_effects_packet(&self.effects))
    }
}

fn post_effects_packet(effects: &[Identifier]) -> CPostEffects {
    CPostEffects {
        effects: effects
            .iter()
            .map(|id| PostEffect { id: id.clone() })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(path: &'static str) -> Identifier {
        Identifier::vanilla_static(path)
    }

    #[test]
    fn a_fresh_list_is_pushed_once_even_when_empty() {
        let mut effects = PlayerPostEffects::new_pending();
        let packet = effects
            .take_update()
            .expect("the initial dirty flag pushes an empty list");
        assert!(packet.effects.is_empty());
        assert!(effects.take_update().is_none());
    }

    #[test]
    fn only_changes_make_the_list_dirty() {
        let mut effects = PlayerPostEffects::default();
        assert!(effects.take_update().is_none());

        assert!(effects.add(id("invert")));
        assert!(!effects.add(id("invert")), "duplicates are rejected");
        let packet = effects.take_update().expect("add dirties the list");
        assert_eq!(packet.effects.len(), 1);
        assert!(effects.take_update().is_none());

        assert!(!effects.remove(&id("blur")));
        assert!(effects.take_update().is_none());
        assert!(effects.remove(&id("invert")));
        assert!(effects.take_update().is_some_and(|p| p.effects.is_empty()));
    }

    #[test]
    fn clearing_an_empty_list_reports_no_change() {
        let mut effects = PlayerPostEffects::default();
        assert!(!effects.clear());
        assert!(effects.take_update().is_none());

        effects.add(id("creeper"));
        assert!(effects.clear());
        assert!(effects.take_update().is_some_and(|p| p.effects.is_empty()));
    }

    #[test]
    fn replace_keeps_order_and_drops_duplicates() {
        let mut effects = PlayerPostEffects::default();
        effects.replace(vec![id("blur"), id("invert"), id("blur")]);
        assert_eq!(effects.effects(), [id("blur"), id("invert")]);
        assert!(effects.take_update().is_some());

        // Replacing with the same list is not a change.
        effects.replace(vec![id("blur"), id("invert")]);
        assert!(effects.take_update().is_none());
    }
}
