//! Vanilla `ItemDisplay` and `TextDisplay`.
//!
//! Both concrete display entities share the `Display` synced-data layer
//! (transformation, billboard constraints, shadow, width/height) and differ only
//! in the payload they draw: an item stack or a text component. `BlockDisplay`
//! lives in `display_ui` next to the item frame.

use std::sync::{Arc, Weak};

use glam::DVec3;
use simdnbt::borrow::NbtCompound as BorrowedNbtCompoundView;
use simdnbt::owned::NbtCompound;
use steel_macros::entity_behavior;
use steel_registry::entity_data::{Quaternionf, Vector3f};
use steel_registry::entity_type::EntityTypeRef;
use steel_registry::item_stack::ItemStack;
use steel_registry::vanilla_entity_data::{
    DisplayItemDisplayEntityData, DisplayTextDisplayEntityData,
};
use steel_utils::locks::SyncMutex;
use steel_utils::types::InteractionHand;
use steel_utils::{DowncastType, DowncastTypeKey, WorldAabb};
use text_components::TextComponent;

use crate::behavior::InteractionResult;
use crate::entity::{
    Entity, EntityBase, EntityBaseLoad, EntityMovementEmission, EntitySyncedData, SharedEntity,
};
use crate::player::Player;
use crate::world::World;

/// Vanilla `Display.BillboardConstraints`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BillboardConstraints {
    /// Rotates with the entity.
    #[default]
    Fixed,
    /// Always faces the viewer vertically.
    Vertical,
    /// Always faces the viewer horizontally.
    Horizontal,
    /// Always faces the viewer on both axes.
    Center,
}

impl BillboardConstraints {
    /// Decodes the vanilla ordinal.
    #[must_use]
    pub const fn from_ordinal(ordinal: i8) -> Self {
        match ordinal {
            1 => Self::Vertical,
            2 => Self::Horizontal,
            3 => Self::Center,
            _ => Self::Fixed,
        }
    }

    /// Encodes the vanilla ordinal.
    #[must_use]
    pub const fn ordinal(self) -> i8 {
        match self {
            Self::Fixed => 0,
            Self::Vertical => 1,
            Self::Horizontal => 2,
            Self::Center => 3,
        }
    }
}

/// Shared state for every display entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayTransformation {
    pub translation: Vector3f,
    pub scale: Vector3f,
    pub left_rotation: Quaternionf,
    pub right_rotation: Quaternionf,
}

impl Default for DisplayTransformation {
    fn default() -> Self {
        Self {
            translation: Vector3f::ZERO,
            scale: Vector3f::new(1.0, 1.0, 1.0),
            left_rotation: Quaternionf::IDENTITY,
            right_rotation: Quaternionf::IDENTITY,
        }
    }
}

macro_rules! display_boilerplate {
    ($entity:ident, $data:ty, $key:literal) => {
        // SAFETY: This key is owned by Steel and uniquely identifies the entity.
        unsafe impl DowncastType for $entity {
            const TYPE_KEY: DowncastTypeKey = DowncastTypeKey::new($key);
        }

        impl $entity {
            /// The current display transformation.
            #[must_use]
            pub fn transformation(&self) -> DisplayTransformation {
                let data = self.entity_data.lock();
                DisplayTransformation {
                    translation: *data.display.translation.get(),
                    scale: *data.display.scale.get(),
                    left_rotation: *data.display.left_rotation.get(),
                    right_rotation: *data.display.right_rotation.get(),
                }
            }

            /// Replaces the display transformation.
            pub fn set_transformation(&self, transformation: DisplayTransformation) {
                let mut data = self.entity_data.lock();
                data.display.translation.set(transformation.translation);
                data.display.scale.set(transformation.scale);
                data.display.left_rotation.set(transformation.left_rotation);
                data.display.right_rotation.set(transformation.right_rotation);
            }

            /// The billboard mode, decoded from the synced byte.
            #[must_use]
            pub fn billboard(&self) -> BillboardConstraints {
                let data = self.entity_data.lock();
                BillboardConstraints::from_ordinal(*data.display.billboard_render_constraints.get())
            }

            /// Sets the billboard mode.
            pub fn set_billboard(&self, billboard: BillboardConstraints) {
                self.entity_data
                    .lock()
                    .display
                    .billboard_render_constraints
                    .set(billboard.ordinal());
            }

            /// Set whether the entity is glowing.
            pub fn set_glow_color_override(&self, color: i32) {
                self.entity_data
                    .lock()
                    .display
                    .glow_color_override
                    .set(color);
            }

            /// The display view range multiplier.
            #[must_use]
            pub fn view_range(&self) -> f32 {
                *self.entity_data.lock().display.view_range.get()
            }

            /// Sets the display view range multiplier.
            pub fn set_view_range(&self, range: f32) {
                self.entity_data.lock().display.view_range.set(range.clamp(0.0, 64.0));
            }

            /// The display width, `0.0` when unset.
            #[must_use]
            pub fn display_width(&self) -> f32 {
                *self.entity_data.lock().display.width.get()
            }

            /// The display height, `0.0` when unset.
            #[must_use]
            pub fn display_height(&self) -> f32 {
                *self.entity_data.lock().display.height.get()
            }

            /// Vanilla `Display.setWidth`/`setHeight`.
            pub fn set_display_size(&self, width: f32, height: f32) {
                let mut data = self.entity_data.lock();
                data.display.width.set(width.max(0.0));
                data.display.height.set(height.max(0.0));
            }
        }

        impl Entity for $entity {
            fn base(&self) -> &EntityBase {
                &self.base
            }

            fn entity_type(&self) -> EntityTypeRef {
                self.entity_type
            }

            fn movement_emission(&self) -> EntityMovementEmission {
                EntityMovementEmission::None
            }

            fn synced_data(&self) -> Option<&dyn EntitySyncedData> {
                Some(&self.entity_data)
            }

            fn interact(
                &self,
                _player: &Player,
                _hand: InteractionHand,
                _location: DVec3,
            ) -> InteractionResult {
                InteractionResult::Pass
            }

            fn save_additional(&self, nbt: &mut NbtCompound) {
                let data = self.entity_data.lock();
                nbt.insert(
                    "billboard",
                    data.display.billboard_render_constraints.get().to_string(),
                );
                nbt.insert("view_range", f64::from(*data.display.view_range.get()));
                nbt.insert("shadow_radius", f64::from(*data.display.shadow_radius.get()));
                nbt.insert("shadow_strength", f64::from(*data.display.shadow_strength.get()));
                nbt.insert("width", f64::from(*data.display.width.get()));
                nbt.insert("height", f64::from(*data.display.height.get()));
                nbt.insert("glow_color_override", *data.display.glow_color_override.get());
            }

            fn load_additional(&self, nbt: BorrowedNbtCompoundView<'_, '_>) {
                let mut data = self.entity_data.lock();
                if let Some(range) = nbt.float_value("view_range") {
                    data.display.view_range.set(range);
                }
                if let Some(width) = nbt.float_value("width") {
                    data.display.width.set(width);
                }
                if let Some(height) = nbt.float_value("height") {
                    data.display.height.set(height);
                }
                if let Some(color) = nbt.int("glow_color_override") {
                    data.display.glow_color_override.set(color);
                }
                if let Some(billboard) = nbt.byte_value("billboard") {
                    data.display
                        .billboard_render_constraints
                        .set(billboard);
                }
            }
        }
    };
}

macro_rules! display_entity {
    ($entity:ident, $data:ty, $class:literal, $key:literal) => {
        #[entity_behavior(class = $class)]
        /// A vanilla display entity.
        pub struct $entity {
            base: EntityBase,
            entity_type: EntityTypeRef,
            entity_data: SyncMutex<$data>,
        }

        display_boilerplate!($entity, $data, $key);

        impl $entity {
            /// Creates a new display entity.
            #[must_use]
            pub fn new(
                entity_type: EntityTypeRef,
                id: i32,
                position: DVec3,
                world: Weak<World>,
            ) -> Self {
                Self {
                    base: EntityBase::new(id, position, entity_type.dimensions, world),
                    entity_type,
                    entity_data: SyncMutex::new(<$data>::new()),
                }
            }

            /// Creates a display entity from saved data.
            #[must_use]
            pub fn from_saved(entity_type: EntityTypeRef, load: EntityBaseLoad) -> Self {
                Self {
                    base: EntityBase::from_load(load, entity_type.dimensions),
                    entity_type,
                    entity_data: SyncMutex::new(<$data>::new()),
                }
            }
        }
    };
}

// The plain `Display` entity has no payload of its own; vanilla never spawns it
// directly (it is abstract), so only the three concrete subclasses are exposed.
display_entity!(
    ItemDisplayEntity,
    DisplayItemDisplayEntityData,
    "ItemDisplay",
    "steel:entity/item_display"
);
display_entity!(
    TextDisplayEntity,
    DisplayTextDisplayEntityData,
    "TextDisplay",
    "steel:entity/text_display"
);

impl ItemDisplayEntity {
    /// The displayed item stack.
    #[must_use]
    pub fn item_stack(&self) -> ItemStack {
        self.entity_data.lock().item_display.item_stack.get().clone()
    }

    /// Sets the displayed item stack.
    pub fn set_item_stack(&self, stack: ItemStack) {
        self.entity_data.lock().item_display.item_stack.set(stack);
    }

    /// Vanilla `ItemDisplay.ItemDisplayContext` ordinal.
    #[must_use]
    pub fn display_context(&self) -> i8 {
        *self.entity_data.lock().item_display.item_display.get()
    }

    /// Sets the vanilla `ItemDisplay.ItemDisplayContext` ordinal.
    pub fn set_display_context(&self, context: i8) {
        self.entity_data
            .lock()
            .item_display
            .item_display
            .set(context);
    }
}

impl TextDisplayEntity {
    /// The displayed text component.
    #[must_use]
    pub fn text(&self) -> TextComponent {
        (**self.entity_data.lock().text_display.text.get()).clone()
    }

    /// Sets the displayed text component.
    pub fn set_text(&self, text: TextComponent) {
        self.entity_data
            .lock()
            .text_display
            .text
            .set(Box::new(text));
    }

    /// Vanilla `TextDisplay` line width, wrapped at `line_width` pixels.
    #[must_use]
    pub fn line_width(&self) -> i32 {
        *self.entity_data.lock().text_display.line_width.get()
    }

    /// Sets the vanilla `TextDisplay` line width.
    pub fn set_line_width(&self, width: i32) {
        self.entity_data.lock().text_display.line_width.set(width);
    }

    /// The background colour as ARGB.
    #[must_use]
    pub fn background_color(&self) -> i32 {
        *self.entity_data.lock().text_display.background_color.get()
    }

    /// Sets the background colour as ARGB.
    pub fn set_background_color(&self, color: i32) {
        self.entity_data
            .lock()
            .text_display
            .background_color
            .set(color);
    }

    /// Vanilla `TextDisplay` text opacity, `-1` means opaque.
    #[must_use]
    pub fn text_opacity(&self) -> i8 {
        *self.entity_data.lock().text_display.text_opacity.get()
    }

    /// Sets the vanilla `TextDisplay` text opacity.
    pub fn set_text_opacity(&self, opacity: i8) {
        self.entity_data.lock().text_display.text_opacity.set(opacity);
    }

    /// Whether the text is rendered see-through.
    #[must_use]
    pub fn is_see_through(&self) -> bool {
        *self.entity_data.lock().text_display.style_flags.get() & 0x02 != 0
    }

    /// Vanilla `TextDisplay.setFlags` for the see-through flag.
    pub fn set_see_through(&self, see_through: bool) {
        let mut data = self.entity_data.lock();
        let mut flags = *data.text_display.style_flags.get();
        if see_through {
            flags |= 0x02;
        } else {
            flags &= !0x02;
        }
        data.text_display.style_flags.set(flags);
    }

    /// Whether the text is drawn with a default shadow.
    #[must_use]
    pub fn has_shadow(&self) -> bool {
        *self.entity_data.lock().text_display.style_flags.get() & 0x01 != 0
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::init_vanilla_registry;
    use steel_registry::vanilla_entities;

    use super::*;

    #[test]
    fn billboard_ordinals_round_trip() {
        for constraint in [
            BillboardConstraints::Fixed,
            BillboardConstraints::Vertical,
            BillboardConstraints::Horizontal,
            BillboardConstraints::Center,
        ] {
            assert_eq!(
                BillboardConstraints::from_ordinal(constraint.ordinal()),
                constraint
            );
        }
        assert_eq!(
            BillboardConstraints::from_ordinal(99),
            BillboardConstraints::Fixed
        );
    }

    #[test]
    fn item_displays_store_their_stack() {
        init_vanilla_registry();
        let display = ItemDisplayEntity::new(
            &vanilla_entities::ITEM_DISPLAY,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert!(display.item_stack().is_empty());
        display.set_item_stack(ItemStack::new(&steel_registry::vanilla_items::STICK));
        assert_eq!(display.item_stack().item().key.to_string(), "minecraft:stick");
        assert_eq!(display.display_context(), 0);
    }

    #[test]
    fn text_display_flags_toggle_independently() {
        init_vanilla_registry();
        let display = TextDisplayEntity::new(
            &vanilla_entities::TEXT_DISPLAY,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert!(!display.is_see_through());
        display.set_see_through(true);
        assert!(display.is_see_through());
        assert!(!display.has_shadow());
        display.set_see_through(false);
        assert!(!display.is_see_through());
    }

    #[test]
    fn text_display_defaults_match_vanilla() {
        init_vanilla_registry();
        let display = TextDisplayEntity::new(
            &vanilla_entities::TEXT_DISPLAY,
            1,
            DVec3::ZERO,
            Weak::new(),
        );
        assert_eq!(display.line_width(), 200);
        assert_eq!(display.background_color(), 0x4000_0000);
        assert_eq!(display.text_opacity(), -1);
    }
}
