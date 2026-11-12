//! Custom report details packet.

use steel_macros::ClientPacket;
use steel_registry::packets::{config, play};
use steel_utils::codec::VarInt;
use steel_utils::serial::WriteTo;

/// One `title`/`description` pair shown in the chat report screen.
#[derive(Debug, Clone)]
pub struct ReportDetail {
    /// Field title.
    pub title: String,
    /// Field description.
    pub description: String,
}

/// Clientbound packet that adds server-specific fields to chat reports.
#[derive(ClientPacket, Debug, Clone)]
#[packet_id(Config = config::C_CUSTOM_REPORT_DETAILS, Play = play::C_CUSTOM_REPORT_DETAILS)]
pub struct CCustomReportDetails {
    /// Extra report fields.
    pub details: Vec<ReportDetail>,
}

impl WriteTo for CCustomReportDetails {
    fn write(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        VarInt(self.details.len() as i32).write(writer)?;
        for detail in &self.details {
            detail.title.write(writer)?;
            detail.description.write(writer)?;
        }
        Ok(())
    }
}
