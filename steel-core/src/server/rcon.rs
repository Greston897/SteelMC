//! Source RCON server.
//!
//! Implements the protocol Valve documented for Source engine servers, which is
//! the one vanilla Minecraft speaks: a little-endian length prefix, a request
//! id, a type and a NUL-terminated payload pair. `SERVERDATA_AUTH` carries the
//! password, `SERVERDATA_AUTH_RESPONSE` answers it (with `-1` on failure), and
//! `SERVERDATA_EXECCOMMAND` runs one command whose console output comes back as
//! a single `SERVERDATA_RESPONSE_VALUE`.
//!
//! Vanilla's `RconConsoleSource` collects output into a buffer and sends it once
//! the command finishes; that is exactly what the reply channel here models.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::command::sender::{CommandSender, RconOutput};
use crate::config::RconConfig;
use crate::server::Server;

/// Payload bytes per packet, matching vanilla's 1443-byte packet limit.
const MAX_PAYLOAD_SIZE: usize = 1443;
/// Guard against a malformed length prefix.
const MAX_PACKET_SIZE: i32 = 1460;

const SERVERDATA_RESPONSE_VALUE: i32 = 0;
const SERVERDATA_EXECCOMMAND: i32 = 2;
const SERVERDATA_AUTH_RESPONSE: i32 = 2;
const SERVERDATA_AUTH: i32 = 3;

/// One decoded RCON frame.
#[derive(Debug, PartialEq, Eq)]
struct RconPacket {
    id: i32,
    kind: i32,
    body: String,
}

impl RconPacket {
    fn encode(&self) -> Vec<u8> {
        // The two trailing NULs are the payload terminator and the empty
        // "payload" the protocol mandates.
        let payload_len = 4 + 4 + self.body.len() + 2;
        let mut buffer = Vec::with_capacity(payload_len + 4);
        buffer.extend_from_slice(&(payload_len as i32).to_le_bytes());
        buffer.extend_from_slice(&self.id.to_le_bytes());
        buffer.extend_from_slice(&self.kind.to_le_bytes());
        buffer.extend_from_slice(self.body.as_bytes());
        buffer.extend_from_slice(&[0, 0]);
        buffer
    }

    /// Decodes the body of one frame, or `None` when it is malformed.
    fn decode(id: i32, kind: i32, payload: &[u8]) -> Option<Self> {
        // Drop the payload terminator and the extra NUL.
        let body = payload.strip_suffix(&[0, 0]).unwrap_or(payload);
        let body = std::str::from_utf8(body).ok()?;
        Some(Self {
            id,
            kind,
            body: body.to_owned(),
        })
    }
}

/// Errors that stop the RCON listener.
#[derive(Debug)]
pub enum RconError {
    /// The listener could not bind its port.
    Bind(std::io::Error),
}

impl std::fmt::Display for RconError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bind(error) => write!(formatter, "failed to bind the RCON port: {error}"),
        }
    }
}

impl std::error::Error for RconError {}

/// Runs the RCON listener until the server shuts down.
///
/// Returns immediately when the password is empty, which is vanilla's way of
/// disabling RCON.
pub async fn run_rcon(
    server: Arc<Server>,
    config: RconConfig,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(), RconError> {
    if config.password.is_empty() {
        return Ok(());
    }
    let listener = TcpListener::bind(("0.0.0.0", config.port))
        .await
        .map_err(RconError::Bind)?;
    log::info!("RCON listening on port {}", config.port);

    loop {
        let accepted = tokio::select! {
            accepted = listener.accept() => accepted,
            () = cancel_token.cancelled() => return Ok(()),
        };
        let (stream, address) = match accepted {
            Ok(connection) => connection,
            Err(error) => {
                log::warn!("RCON accept failed: {error}");
                continue;
            }
        };
        let password = config.password.clone();
        let server = Arc::clone(&server);
        let cancel = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(error) = serve_connection(server, password, stream, cancel).await {
                log::debug!("RCON connection from {address} ended: {error}");
            }
        });
    }
}

async fn serve_connection(
    server: Arc<Server>,
    password: String,
    mut stream: TcpStream,
    cancel_token: tokio_util::sync::CancellationToken,
) -> std::io::Result<()> {
    let mut authenticated = false;

    loop {
        let Some(packet) = read_packet(&mut stream, &cancel_token).await? else {
            return Ok(());
        };
        match packet.kind {
            SERVERDATA_AUTH => {
                authenticated = packet.body == password;
                // Vanilla answers a failed auth with id -1 and then closes the
                // connection; keep it open for a retry so a mistyped password
                // is recoverable, but report the failure faithfully.
                let response = RconPacket {
                    id: if authenticated { packet.id } else { -1 },
                    kind: SERVERDATA_AUTH_RESPONSE,
                    body: String::new(),
                };
                write_packet(&mut stream, &response).await?;
                if !authenticated {
                    log::debug!("RCON client failed to authenticate");
                }
            }
            SERVERDATA_EXECCOMMAND => {
                if !authenticated {
                    // Vanilla ignores commands before authentication.
                    write_packet(
                        &mut stream,
                        &RconPacket {
                            id: packet.id,
                            kind: SERVERDATA_RESPONSE_VALUE,
                            body: String::new(),
                        },
                    )
                    .await?;
                    continue;
                }
                let output = execute_command(&server, packet.body).await;
                write_packet(
                    &mut stream,
                    &RconPacket {
                        id: packet.id,
                        kind: SERVERDATA_RESPONSE_VALUE,
                        body: output,
                    },
                )
                .await?;
            }
            _ => {
                // Unknown types are ignored, as in vanilla.
            }
        }
    }
}

/// Runs one command and collects everything it sends to the RCON source.
async fn execute_command(server: &Arc<Server>, command: String) -> String {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let output = RconOutput::new(sender);
    if server
        .submit_command(CommandSender::Rcon(output), command)
        .is_err()
    {
        return "Command queue is full".to_owned();
    }

    // The command runs on the server tick; wait for its first reply and then
    // drain whatever the tick produced. A short settle window is enough
    // because the whole command is executed inside one tick.
    let mut lines = Vec::new();
    let first = tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv()).await;
    match first {
        Ok(Some(line)) => lines.push(line),
        Ok(None) => return String::new(),
        Err(_) => {
            // No message at all: the command either produced no output or is
            // still queued. Report an empty response instead of blocking the
            // connection.
            return String::new();
        }
    }
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    while let Ok(line) = receiver.try_recv() {
        lines.push(line);
    }
    lines.join("\n")
}

async fn read_packet(
    stream: &mut TcpStream,
    cancel_token: &tokio_util::sync::CancellationToken,
) -> std::io::Result<Option<RconPacket>> {
    let mut length_bytes = [0u8; 4];
    let read = tokio::select! {
        read = stream.read_exact(&mut length_bytes) => read,
        () = cancel_token.cancelled() => return Ok(None),
    };
    match read {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }

    let length = i32::from_le_bytes(length_bytes);
    if !(10..=MAX_PACKET_SIZE).contains(&length) {
        // A garbage length prefix cannot be recovered from; drop the client.
        return Ok(None);
    }

    let mut body = vec![0u8; length as usize];
    stream.read_exact(&mut body).await?;
    if body.len() < 10 {
        return Ok(None);
    }
    let id = i32::from_le_bytes([body[0], body[1], body[2], body[3]]);
    let kind = i32::from_le_bytes([body[4], body[5], body[6], body[7]]);
    let payload = &body[8..];
    if payload.len() > MAX_PAYLOAD_SIZE + 2 {
        return Ok(None);
    }
    Ok(RconPacket::decode(id, kind, payload))
}

async fn write_packet(stream: &mut TcpStream, packet: &RconPacket) -> std::io::Result<()> {
    stream.write_all(&packet.encode()).await?;
    stream.flush().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_round_trips_through_the_wire_format() {
        let packet = RconPacket {
            id: 7,
            kind: SERVERDATA_EXECCOMMAND,
            body: "list".to_owned(),
        };
        let encoded = packet.encode();
        assert_eq!(
            encoded.len(),
            4 + 4 + 4 + 4 + 2,
            "length prefix + id + type + payload + two NULs"
        );

        let length = i32::from_le_bytes(encoded[0..4].try_into().expect("length prefix"));
        assert_eq!(length as usize, encoded.len() - 4);
        let id = i32::from_le_bytes(encoded[4..8].try_into().expect("id"));
        let kind = i32::from_le_bytes(encoded[8..12].try_into().expect("kind"));
        let decoded = RconPacket::decode(id, kind, &encoded[12..]).expect("payload decodes");
        assert_eq!(decoded, packet);
    }

    #[test]
    fn empty_body_keeps_the_protocol_nuls() {
        let packet = RconPacket {
            id: -1,
            kind: SERVERDATA_AUTH_RESPONSE,
            body: String::new(),
        };
        let encoded = packet.encode();
        assert_eq!(encoded.len(), 14);
        assert_eq!(&encoded[10..], &[0, 0]);
        let decoded =
            RconPacket::decode(-1, SERVERDATA_AUTH_RESPONSE, &encoded[8..]).expect("decodes");
        assert_eq!(decoded.body, "");
    }
}
