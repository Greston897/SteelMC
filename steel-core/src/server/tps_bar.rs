//! `/tpsbar`: server performance shown in a boss bar.
//!
//! The bar reuses the tick statistics the server already collects for the tab
//! list ([`TabListTickStats`]): TPS, the smoothed MSPT, the five second average
//! and the p95 MSPT. A single bar identity is shared by every player, so the
//! client only ever updates one bar.

use steel_protocol::packets::game::{BossBarColor, BossBarOverlay, CBossEvent};
use steel_utils::TICKS_PER_SECOND;
use text_components::format::Color;
use text_components::{Modifier, TextComponent};
use uuid::Uuid;

use super::{Server, TabListTickStats};

/// Identity of the shared `/tpsbar` boss bar.
pub const TPS_BAR_UUID: Uuid = Uuid::from_u128(0x5450_5342_4152_0000_0000_0000_0000_0001);

/// TPS at and above which the bar is green (matches the tab list threshold).
pub const TPS_BAR_GREEN_THRESHOLD: f32 = 19.5;
/// TPS at and above which the bar is yellow.
pub const TPS_BAR_YELLOW_THRESHOLD: f32 = 15.0;
/// MSPT at and above which the numbers turn red (`50` ms is one full tick).
pub const MSPT_BAR_RED_THRESHOLD: f32 = 50.0;

/// Boss bar fill for a TPS value, clamped to `0.0..=1.0`.
#[must_use]
pub fn progress_for(tps: f32) -> f32 {
    if !tps.is_finite() {
        return 0.0;
    }
    (tps / TICKS_PER_SECOND as f32).clamp(0.0, 1.0)
}

/// Boss bar colour for a TPS value, using the tab list thresholds.
#[must_use]
pub fn color_for(tps: f32) -> BossBarColor {
    if tps >= TPS_BAR_GREEN_THRESHOLD {
        BossBarColor::Green
    } else if tps >= TPS_BAR_YELLOW_THRESHOLD {
        BossBarColor::Yellow
    } else {
        BossBarColor::Red
    }
}

fn mspt_color(mspt: f32) -> Color {
    if mspt <= MSPT_BAR_RED_THRESHOLD {
        Color::Aqua
    } else {
        Color::Red
    }
}

/// Title of the boss bar: TPS plus recent / average / p95 MSPT.
#[must_use]
pub fn tps_bar_text(stats: TabListTickStats) -> TextComponent {
    TextComponent::plain("").add_children(vec![
        TextComponent::plain("TPS ").color(Color::Gray),
        TextComponent::plain(format!("{:.1}", stats.tps)).color(color_for(stats.tps)),
        TextComponent::plain("  MSPT ").color(Color::Gray),
        TextComponent::plain(format!("{:.2}", stats.recent_mspt)).color(mspt_color(stats.recent_mspt)),
        TextComponent::plain(format!(
            " (5s {:.2}, p95 {:.2})",
            stats.average_mspt, stats.p95_mspt
        ))
        .color(Color::DarkGray),
    ])
}

impl Server {
    /// Whether the `/tpsbar` boss bar is currently enabled.
    #[must_use]
    pub fn is_tps_bar_enabled(&self) -> bool {
        self.tps_bar_enabled.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Flips the `/tpsbar` toggle and returns the new state.
    ///
    /// The bars are not touched here: the next broadcast sends one `remove`
    /// action per remembered player, so disabling never leaves a stale bar.
    pub fn toggle_tps_bar(&self) -> bool {
        let enabled = !self.is_tps_bar_enabled();
        self.tps_bar_enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
        enabled
    }

    /// Drops remembered bars for players that are no longer online.
    fn prune_tps_bar_players(&self) {
        let mut shown = self.tps_bar_shown.lock();
        let stale: Vec<Uuid> = shown
            .keys()
            .copied()
            .filter(|uuid| self.online_players.get_by_uuid(uuid).is_none())
            .collect();
        for uuid in stale {
            shown.remove(&uuid);
        }
    }

    /// Sends the current `/tpsbar` state to every online player.
    ///
    /// Runs on the same cadence as the tab list (once per second).
    pub(super) fn broadcast_tps_bar(&self, stats: TabListTickStats) {
        self.prune_tps_bar_players();

        if !self.is_tps_bar_enabled() {
            let shown: Vec<Uuid> = {
                let mut shown = self.tps_bar_shown.lock();
                let uuids = shown.keys().copied().collect();
                shown.clear();
                uuids
            };
            for uuid in shown {
                if let Some(player) = self.online_players.get_by_uuid(&uuid) {
                    player.send_packet(CBossEvent::remove(TPS_BAR_UUID));
                }
            }
            return;
        }

        let progress = progress_for(stats.tps);
        let color = color_for(stats.tps);
        let text = tps_bar_text(stats);

        self.online_players.iter_players(|uuid, player| {
            let already_shown = self.tps_bar_shown.lock().contains_key(uuid);
            if already_shown {
                player.send_packet(CBossEvent::update_progress(TPS_BAR_UUID, progress));
                player.send_packet(CBossEvent::update_name(TPS_BAR_UUID, text.clone()));
            } else {
                player.send_packet(CBossEvent::add(
                    TPS_BAR_UUID,
                    text.clone(),
                    progress,
                    color,
                    BossBarOverlay::Progress,
                    0,
                ));
                self.tps_bar_shown.lock().insert(*uuid, ());
            }
            true
        });
    }
}

#[cfg(test)]
mod tests {
    use steel_utils::text::DisplayResolutor;

    use super::{
        TPS_BAR_GREEN_THRESHOLD, TPS_BAR_YELLOW_THRESHOLD, color_for, progress_for, tps_bar_text,
    };
    use crate::server::TabListTickStats;
    use steel_protocol::packets::game::BossBarColor;

    fn stats(tps: f32, recent_mspt: f32, average_mspt: f32, p95_mspt: f32) -> TabListTickStats {
        TabListTickStats {
            tps,
            recent_mspt,
            average_mspt,
            p95_mspt,
        }
    }

    #[test]
    fn progress_maps_tps_onto_the_bar() {
        assert!((progress_for(20.0) - 1.0).abs() < f32::EPSILON);
        assert!((progress_for(10.0) - 0.5).abs() < f32::EPSILON);
        assert!((progress_for(0.0)).abs() < f32::EPSILON);
        assert!((progress_for(40.0) - 1.0).abs() < f32::EPSILON, "clamped");
        assert!((progress_for(-5.0)).abs() < f32::EPSILON, "clamped");
        assert!((progress_for(f32::NAN)).abs() < f32::EPSILON, "nan safe");
    }

    #[test]
    fn colours_follow_the_tab_list_thresholds() {
        assert_eq!(color_for(TPS_BAR_GREEN_THRESHOLD), BossBarColor::Green);
        assert_eq!(color_for(20.0), BossBarColor::Green);
        assert_eq!(color_for(TPS_BAR_YELLOW_THRESHOLD), BossBarColor::Yellow);
        assert_eq!(color_for(18.0), BossBarColor::Yellow);
        assert_eq!(color_for(14.9), BossBarColor::Red);
        assert_eq!(color_for(0.0), BossBarColor::Red);
    }

    #[test]
    fn bar_text_reports_tps_and_all_mspt_views() {
        let text = tps_bar_text(stats(19.98, 1.25, 1.5, 3.0))
            .to_plain(&DisplayResolutor);
        assert!(text.contains("TPS 20.0"), "rounded tps: {text}");
        assert!(text.contains("MSPT 1.25"), "recent mspt: {text}");
        assert!(text.contains("5s 1.50"), "average mspt: {text}");
        assert!(text.contains("p95 3.00"), "p95 mspt: {text}");
    }
}
