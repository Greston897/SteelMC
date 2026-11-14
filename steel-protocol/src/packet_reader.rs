//! # Steel Protocol Packet Reader
//!
//! This module contains the implementation of the packet reader.
/*
Credit to https://github.com/Pumpkin-MC/Pumpkin/ for this implementation.
*/

use std::{
    io::{self, Read},
    num::NonZeroU32,
    pin::Pin,
    task::{Context, Poll},
};

use aes::cipher::KeyIvInit;
use flate2::read::ZlibDecoder;
use steel_utils::codec::VarInt;
use steel_utils::serial::ReadFrom;
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

use crate::utils::{
    Aes128Cfb8Dec, MAX_PACKET_DATA_SIZE, MAX_PACKET_SIZE, PacketError, RawPacket, StreamDecryptor,
};

/// A reader that can decrypt data.
pub enum DecryptionReader<R: AsyncRead + Unpin> {
    /// A reader that decrypts data.
    Decrypt(Box<StreamDecryptor<R>>),
    /// A reader that does not decrypt data.
    None(R),
}

impl<R: AsyncRead + Unpin> DecryptionReader<R> {
    /// Upgrades the reader to decrypt data.
    ///
    /// # Panics
    /// - If the reader is already decrypting data.
    #[must_use]
    pub fn upgrade(self, cipher: Aes128Cfb8Dec) -> Self {
        match self {
            Self::None(stream) => Self::Decrypt(Box::new(StreamDecryptor::new(cipher, stream))),
            Self::Decrypt(_) => panic!("Cannot upgrade a stream that already has a cipher!"),
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for DecryptionReader<R> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Decrypt(reader) => {
                let reader = Pin::new(reader);
                reader.poll_read(cx, buf)
            }
            Self::None(reader) => {
                let reader = Pin::new(reader);
                reader.poll_read(cx, buf)
            }
        }
    }
}

/// Decoder: Client -> Server
/// Supports `ZLib` decoding/decompression
/// Supports Aes128 Encryption
pub struct TCPNetworkDecoder<R: AsyncRead + Unpin> {
    reader: DecryptionReader<R>,
    compression: Option<NonZeroU32>,
}

impl<R: AsyncRead + Unpin> TCPNetworkDecoder<R> {
    /// Creates a new `TCPNetworkDecoder`.
    pub const fn new(reader: R) -> Self {
        Self {
            reader: DecryptionReader::None(reader),
            compression: None,
        }
    }

    /// Sets the compression threshold for the decoder.
    pub const fn set_compression(&mut self, threshold: NonZeroU32) {
        self.compression = Some(threshold);
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    ///
    /// # Panics
    /// - If the reader is already decrypting data.
    /// - If the key is invalid.
    pub fn set_encryption(&mut self, key: &[u8; 16]) {
        if matches!(self.reader, DecryptionReader::Decrypt(_)) {
            panic!("Cannot upgrade a stream that already has a cipher!");
        }
        let cipher = Aes128Cfb8Dec::new_from_slices(key, key).expect("invalid key");
        replace_with::replace_with_or_abort(&mut self.reader, |decoder| decoder.upgrade(cipher));
    }

    /// Gets a raw packet from the stream.
    ///
    /// # Errors
    /// - If the packet length is invalid.
    /// - If the packet is too long.
    /// - If the packet is not compressed when it should be.
    /// - If the packet fails to decompress.
    #[expect(clippy::cast_sign_loss)]
    pub async fn get_raw_packet(&mut self) -> Result<RawPacket, PacketError> {
        let packet_len = VarInt::read_async(&mut self.reader).await? as usize;

        if packet_len > MAX_PACKET_SIZE {
            return Err(PacketError::OutOfBounds);
        }

        // Read the entire packet data into a buffer
        let mut packet_data = vec![0u8; packet_len];
        self.reader
            .read_exact(&mut packet_data)
            .await
            .map_err(|e| PacketError::Other(e.to_string()))?;

        let mut cursor = io::Cursor::new(packet_data.as_slice());

        let (packet_buffer, packet_data_start) = if let Some(threshold) = self.compression {
            let decompressed_len = VarInt::read(&mut cursor)?.0 as usize;
            let raw_packet_len = packet_len - VarInt::written_size(decompressed_len as i32);

            if decompressed_len > MAX_PACKET_DATA_SIZE {
                return Err(PacketError::TooLong(decompressed_len));
            }

            if decompressed_len > 0 {
                let mut decompressed = vec![0; decompressed_len];
                let mut decoder = ZlibDecoder::new(&mut cursor);
                decoder
                    .read_exact(&mut decompressed)
                    .map_err(|e| PacketError::DecompressionFailed(e.to_string()))?;

                let mut overflow = [0];
                if decoder
                    .read(&mut overflow)
                    .map_err(|e| PacketError::DecompressionFailed(e.to_string()))?
                    != 0
                {
                    return Err(PacketError::DecompressionFailed(format!(
                        "decompressed packet exceeds declared length of {decompressed_len}"
                    )));
                }
                (decompressed, 0)
            } else {
                // Validate that we are not less than the compression threshold
                if raw_packet_len > threshold.get() as _ {
                    return Err(PacketError::NotCompressed);
                }

                // The rest of the packet data is uncompressed.
                let packet_data_start = cursor.position() as usize;
                (packet_data, packet_data_start)
            }
        } else {
            (packet_data, 0)
        };

        // Parse the packet ID while retaining the buffer that already owns the payload.
        let mut cursor = io::Cursor::new(&packet_buffer[packet_data_start..]);
        let packet_id = VarInt::read(&mut cursor)?.0;
        let payload_start = packet_data_start + cursor.position() as usize;

        Ok(RawPacket::from_buffer(
            packet_id,
            packet_buffer,
            payload_start,
        ))
    }
}

#[cfg(test)]
mod payload_tests {
    use std::{io::Write as _, num::NonZeroU32};

    use flate2::{Compression, write::ZlibEncoder};

    use super::TCPNetworkDecoder;

    const VARINT_DATA_MASK: u8 = 0x7f;
    const VARINT_CONTINUE_BIT: u8 = 0x80;

    fn write_varint(mut value: u32, output: &mut Vec<u8>) {
        loop {
            if value <= u32::from(VARINT_DATA_MASK) {
                output.push(value as u8);
                return;
            }
            output.push((value as u8 & VARINT_DATA_MASK) | VARINT_CONTINUE_BIT);
            value >>= 7;
        }
    }

    fn packet_data(packet_id: u32, payload: &[u8]) -> Vec<u8> {
        let mut data = Vec::with_capacity(payload.len() + 5);
        write_varint(packet_id, &mut data);
        data.extend_from_slice(payload);
        data
    }

    fn frame(data: &[u8]) -> Vec<u8> {
        let mut framed = Vec::with_capacity(data.len() + 5);
        write_varint(data.len() as u32, &mut framed);
        framed.extend_from_slice(data);
        framed
    }

    #[tokio::test]
    async fn payload_slice_starts_after_framing_and_packet_id() {
        let packet_id = 300;
        let payload = b"payload bytes";
        let frame = frame(&packet_data(packet_id, payload));
        let mut decoder = TCPNetworkDecoder::new(frame.as_slice());

        let packet = decoder.get_raw_packet().await.expect("decode plain packet");

        assert_eq!(packet.id, packet_id as i32);
        assert_eq!(packet.payload(), payload);
    }

    #[tokio::test]
    async fn uncompressed_payload_slice_skips_compression_length() {
        let packet_id = 300;
        let payload = b"payload bytes";
        let packet_data = packet_data(packet_id, payload);
        let mut compression_data = Vec::with_capacity(packet_data.len() + 1);
        write_varint(0, &mut compression_data);
        compression_data.extend_from_slice(&packet_data);
        let frame = frame(&compression_data);
        let mut decoder = TCPNetworkDecoder::new(frame.as_slice());
        decoder.set_compression(NonZeroU32::new(256).expect("nonzero threshold"));

        let packet = decoder
            .get_raw_packet()
            .await
            .expect("decode uncompressed packet");

        assert_eq!(packet.id, packet_id as i32);
        assert_eq!(packet.payload(), payload);
    }

    #[tokio::test]
    async fn decompressed_payload_slice_starts_after_packet_id() {
        let packet_id = 300;
        let payload = vec![0x5a; 1_024];
        let packet_data = packet_data(packet_id, &payload);
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder
            .write_all(&packet_data)
            .expect("compress packet data");
        let compressed = encoder.finish().expect("finish packet compression");
        let mut compression_data = Vec::with_capacity(compressed.len() + 5);
        write_varint(packet_data.len() as u32, &mut compression_data);
        compression_data.extend_from_slice(&compressed);
        let frame = frame(&compression_data);
        let mut decoder = TCPNetworkDecoder::new(frame.as_slice());
        decoder.set_compression(NonZeroU32::new(256).expect("nonzero threshold"));

        let packet = decoder
            .get_raw_packet()
            .await
            .expect("decode compressed packet");

        assert_eq!(packet.id, packet_id as i32);
        assert_eq!(packet.payload(), payload);
    }
}



#[cfg(test)]
mod compression_security_tests {
    use std::io::Write;

    use flate2::{Compression, write::ZlibEncoder};
    use steel_utils::serial::WriteTo;

    use super::*;

    fn compressed_packet(claimed_len: usize, decompressed: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(decompressed)
            .expect("test payload compression should succeed");
        let compressed = encoder
            .finish()
            .expect("test payload compression should finish");

        let mut packet_data = Vec::new();
        VarInt::from(claimed_len)
            .write(&mut packet_data)
            .expect("test claimed length should encode");
        packet_data.extend_from_slice(&compressed);

        let mut packet = Vec::new();
        VarInt::from(packet_data.len())
            .write(&mut packet)
            .expect("test packet length should encode");
        packet.extend_from_slice(&packet_data);
        packet
    }

    #[tokio::test]
    async fn rejects_zlib_stream_expanding_past_declared_length() {
        // This models a tiny wire payload claiming one byte while expanding to twice the protocol
        // maximum.
        let expanded = vec![0; MAX_PACKET_DATA_SIZE * 2];
        let packet = compressed_packet(1, &expanded);
        assert!(packet.len() < 32 * 1024);

        let mut decoder = TCPNetworkDecoder::new(packet.as_slice());
        decoder.set_compression(NonZeroU32::MIN);

        assert!(matches!(
            decoder.get_raw_packet().await,
            Err(PacketError::DecompressionFailed(_))
        ));
    }

    #[tokio::test]
    async fn rejects_zlib_stream_shorter_than_declared_length() {
        let packet = compressed_packet(2, &[0]);
        let mut decoder = TCPNetworkDecoder::new(packet.as_slice());
        decoder.set_compression(NonZeroU32::MIN);

        assert!(matches!(
            decoder.get_raw_packet().await,
            Err(PacketError::DecompressionFailed(_))
        ));
    }

    #[tokio::test]
    async fn accepts_zlib_stream_matching_declared_length() {
        let decompressed = [2, b'h', b'e', b'l', b'l', b'o'];
        let packet = compressed_packet(decompressed.len(), &decompressed);
        let mut decoder = TCPNetworkDecoder::new(packet.as_slice());
        decoder.set_compression(NonZeroU32::MIN);

        let raw_packet = decoder
            .get_raw_packet()
            .await
            .expect("valid compressed packet should decode");

        assert_eq!(raw_packet.id, 2);
        assert_eq!(raw_packet.payload(), b"hello");
    }
}
