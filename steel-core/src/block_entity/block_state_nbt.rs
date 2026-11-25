//! Re-export of the shared block-state NBT codec.
//!
//! The codec lives in `steel-registry` because particle payloads need it too;
//! block entities keep using this module path so existing call sites are stable.

pub(crate) use steel_registry::blocks::block_state_nbt::{load, save};
