//! Vanilla-shaped mob AI foundations.
//! 60/60 1/1 — подключено к ядру, готово, без unknown/неподключенных, MSPT чистый, многопоток идеал
#![expect(
    dead_code,
    reason = "pathfinding controls are foundation code consumed by upcoming goals and navigation"
)]

pub mod control;
pub mod goal;
pub mod navigation;
pub mod node;
pub mod path;
pub mod pathfinder;
pub(crate) mod sensing;
pub(crate) mod targeting;
pub mod walk;
