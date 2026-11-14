//! Scoreboard objective creation/update packet.

use steel_macros::ClientPacket;
use steel_registry::packets::play::C_SET_OBJECTIVE;
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;
use text_components::TextComponent;

/// Vanilla `ScoreboardCriteria.RenderType` ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderType {
    /// Plain integer values.
    Integer = 0,
    /// Heart icons.
    Hearts = 1,
}

impl RenderType {
    /// Wire ordinal of the render type.
    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

/// Vanilla `ClientboundSetObjectivePacket` actions.
#[derive(Debug, Clone)]
pub enum ObjectiveAction {
    /// Creates the objective (`mode = 0`).
    Add {
        /// Displayed name.
        display_name: TextComponent,
        /// Rendering type.
        render_type: RenderType,
    },
    /// Removes the objective (`mode = 1`).
    Remove,
    /// Changes the displayed name and render type (`mode = 2`).
    Update {
        /// Displayed name.
        display_name: TextComponent,
        /// Rendering type.
        render_type: RenderType,
    },
}

/// Clientbound packet that creates, updates or removes a scoreboard objective.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Play = C_SET_OBJECTIVE)]
pub struct CSetObjective {
    /// Objective name.
    pub name: String,
    /// Action to apply.
    pub action: ObjectiveAction,
}

impl WriteTo for CSetObjective {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.name.write(writer)?;
        match &self.action {
            ObjectiveAction::Add {
                display_name,
                render_type,
            } => {
                0_i8.write(writer)?;
                display_name.write(writer)?;
                VarInt(render_type.id()).write(writer)?;
                // Vanilla writes the optional number format; `None` is a bool.
                false.write(writer)?;
            }
            ObjectiveAction::Update {
                display_name,
                render_type,
            } => {
                2_i8.write(writer)?;
                display_name.write(writer)?;
                VarInt(render_type.id()).write(writer)?;
                false.write(writer)?;
            }
            ObjectiveAction::Remove => {
                1_i8.write(writer)?;
            }
        }
        Ok(())
    }
}
