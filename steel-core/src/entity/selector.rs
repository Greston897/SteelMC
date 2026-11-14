//! Entity selection predicates shared by queries, after vanilla `EntitySelector`.

use super::Entity;

pub(crate) struct EntitySelector;

impl EntitySelector {
    /// Vanilla `EntitySelector.pushableBy`: an entity is pushable when it is not
    /// the pusher, not a spectator, is pushable at all, and no team rule forbids
    /// the pair from colliding.
    pub(crate) fn pushable_by(pusher: &dyn Entity) -> impl Fn(&dyn Entity) -> bool {
        let pusher_id = pusher.id();
        let pusher_team = team_name(pusher);

        move |entity| {
            if entity.id() == pusher_id || entity.is_spectator() || !entity.is_pushable() {
                return false;
            }
            match (&pusher_team, team_name(entity)) {
                // Vanilla `Team.canCollideWith`: two members of the same team
                // collide only when the team allows friendly fire.
                (Some(pusher_team), Some(entity_team)) if *pusher_team == entity_team => {
                    pusher.team_allows_friendly_fire()
                }
                _ => true,
            }
        }
    }
}

/// Returns the scoreboard team the entity belongs to, if the world knows one.
fn team_name(entity: &dyn Entity) -> Option<String> {
    entity.team_name()
}
