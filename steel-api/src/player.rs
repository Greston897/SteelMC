//! Игрок-API — чат, инвентарь, телепорт, пакеты, права.

use glam::DVec3;
use uuid::Uuid;
use steel_utils::{BlockPos, Identifier};
use crate::world::WorldRef;

/// Игрок — центральный объект (как `Player` в Bukkit + `ServerPlayer` в ванили).
pub trait Player: Send + Sync + 'static {
    fn uuid(&self) -> Uuid;
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn world(&self) -> WorldRef;
    fn pos(&self) -> DVec3;
    fn block_pos(&self) -> BlockPos;
    fn health(&self) -> f32;
    fn food(&self) -> i32;
    fn game_mode(&self) -> String;
    fn set_game_mode(&self, mode: &str) -> bool;
    fn is_op(&self) -> bool;
    fn has_permission(&self, perm: &str) -> bool;

    // — сообщения —
    fn send_message(&self, msg: impl AsRef<str>);
    fn send_action_bar(&self, msg: impl AsRef<str>);
    fn send_title(&self, title: impl AsRef<str>, subtitle: impl AsRef<str>, fade_in: i32, stay: i32, fade_out: i32);
    fn kick(&self, reason: impl AsRef<str>);

    // — телепорт/движение —
    fn teleport(&self, pos: DVec3, yaw: f32, pitch: f32) -> bool;
    fn teleport_world(&self, world: &WorldRef, pos: DVec3) -> bool;

    // — инвентарь (упрощённо) —
    fn give_item(&self, item: &str, count: i32) -> bool;
    fn clear_inventory(&self);

    // — звук/частицы —
    fn play_sound(&self, sound: &str, volume: f32, pitch: f32);
    fn stop_sound(&self, sound: Option<&str>);

    // — пакеты (низкоуровнево, для ViaVersion/продвинутых) —
    fn send_plugin_message(&self, channel: &Identifier, data: &[u8]) -> bool;
    fn protocol_version(&self) -> i32;
    fn ping(&self) -> i32;
}
