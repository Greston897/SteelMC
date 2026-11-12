//! # Steel Protocol Packet Writer
//!
//! This module contains the implementation of the packet writer.
/*
Credit to https://github.com/Pumpkin-MC/Pumpkin/ for this implementation.
*/

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use aes::cipher::KeyIvInit;
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{
    packet_traits::EncodedPacket,
    utils::{Aes128Cfb8Enc, PacketError, StreamEncryptor},
};

// raw -> compress -> encrypt
/// A writer that can encrypt data.
pub enum EncryptionWriter<W: AsyncWrite + Unpin> {
    /// A writer that encrypts data.
    Encrypt(Box<StreamEncryptor<W>>),
    /// A writer that does not encrypt data.
    None(W),
}

impl<W: AsyncWrite + Unpin> EncryptionWriter<W> {
    /// Upgrades the writer to encrypt data.
    ///
    /// # Panics
    /// - If the writer is already encrypting data.
    #[must_use]
    pub fn upgrade(self, cipher: Aes128Cfb8Enc) -> Self {
        match self {
            Self::None(stream) => Self::Encrypt(Box::new(StreamEncryptor::new(cipher, stream))),
            Self::Encrypt(_) => panic!("Cannot upgrade a stream that already has a cipher!"),
        }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for EncryptionWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = Pin::new(writer);
                writer.poll_write(cx, buf)
            }
            Self::None(writer) => {
                let writer = Pin::new(writer);
                writer.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = Pin::new(writer);
                writer.poll_flush(cx)
            }
            Self::None(writer) => {
                let writer = Pin::new(writer);
                writer.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = Pin::new(writer);
                writer.poll_shutdown(cx)
            }
            Self::None(writer) => {
                let writer = Pin::new(writer);
                writer.poll_shutdown(cx)
            }
        }
    }
}

/// Encoder: Server -> Client
/// Supports `ZLib` endecoding/compression
/// Supports Aes128 Encryption
pub struct TCPNetworkEncoder<W: AsyncWrite + Unpin> {
    writer: EncryptionWriter<W>,
}

impl<W: AsyncWrite + Unpin> TCPNetworkEncoder<W> {
    /// Creates a new `TCPNetworkEncoder`.
    pub const fn new(writer: W) -> Self {
        Self {
            writer: EncryptionWriter::None(writer),
        }
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    ///
    /// # Panics
    /// - If the stream is already encrypted.
    /// - If the key is invalid.
    pub fn set_encryption(&mut self, key: &[u8; 16]) {
        if matches!(self.writer, EncryptionWriter::Encrypt(_)) {
            panic!("Cannot upgrade a stream that already has a cipher!");
        }
        let cipher = Aes128Cfb8Enc::new_from_slices(key, key).expect("invalid key");
        replace_with::replace_with_or_abort(&mut self.writer, |encoder| encoder.upgrade(cipher));
    }

    /// Writes a packet to the stream.
    ///
    /// # Errors
    /// - If the packet fails to write.
    /// - If the stream fails to flush.
    pub async fn write_packet(&mut self, packet: &EncodedPacket) -> Result<(), PacketError> {
        self.writer
            .write_all(&packet.encoded_data)
            .await
            .map_err(|e| PacketError::EncryptionFailed(e.to_string()))?;

        self.writer
            .flush()
            .await
            .map_err(|e| PacketError::EncryptionFailed(e.to_string()))
    }
}

/// An error that occurs when the compression level is invalid.
#[derive(Error, Debug)]
#[error("Invalid compression Level")]
pub struct CompressionLevelError;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

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

    /// Builds the encoded frame a packet with this payload should produce.
    fn expected_frame(packet_data: &[u8]) -> Vec<u8> {
        let mut framed = Vec::with_capacity(packet_data.len() + 5);
        write_varint(packet_data.len() as u32, &mut framed);
        framed.extend_from_slice(packet_data);
        framed
    }

    fn encoded(packet_data: &[u8]) -> EncodedPacket {
        EncodedPacket::from_raw_data(packet_data)
    }

    #[tokio::test]
    async fn writes_length_prefixed_frame() {
        let packet_data = b"\x01payload";
        let mut buf = Vec::new();
        {
            let mut encoder = TCPNetworkEncoder::new(&mut buf);
            encoder
                .write_packet(&encoded(packet_data))
                .await
                .expect("frame writes");
        }

        assert_eq!(buf, expected_frame(packet_data));
    }

    #[tokio::test]
    async fn writes_multi_byte_length_prefix() {
        let packet_data = vec![0x5a; 300];
        let mut buf = Vec::new();
        {
            let mut encoder = TCPNetworkEncoder::new(&mut buf);
            encoder
                .write_packet(&encoded(&packet_data))
                .await
                .expect("frame writes");
        }

        assert_eq!(buf, expected_frame(&packet_data));
        // A 300 byte frame needs a two byte VarInt length.
        assert_eq!(buf[1] & VARINT_CONTINUE_BIT, VARINT_CONTINUE_BIT);
    }

    #[tokio::test]
    async fn appends_packets_in_order() {
        let mut buf = Vec::new();
        {
            let mut encoder = TCPNetworkEncoder::new(&mut buf);
            for byte in [1u8, 2] {
                encoder
                    .write_packet(&encoded(&[byte; 4]))
                    .await
                    .expect("frame writes");
            }
        }

        let mut expected = expected_frame(&[1; 4]);
        expected.extend(expected_frame(&[2; 4]));
        assert_eq!(buf, expected);
    }

    /// Encryption must be transparent to the frame layout once decrypted; the
    /// same key on both sides restores the plaintext frame.
    #[tokio::test]
    async fn encryption_round_trips_the_frame() {
        let key = [7u8; 16];
        let packet_data = b"\x02secret payload";
        let mut buf = Vec::new();
        {
            let mut encoder = TCPNetworkEncoder::new(&mut buf);
            encoder.set_encryption(&key);
            encoder
                .write_packet(&encoded(packet_data))
                .await
                .expect("encrypted frame writes");
        }

        assert_ne!(buf, expected_frame(packet_data));

        let mut decoder = crate::packet_reader::TCPNetworkDecoder::new(buf.as_slice());
        decoder.set_encryption(&key);
        let packet = decoder
            .get_raw_packet()
            .await
            .expect("encrypted frame decodes");

        assert_eq!(packet.id, i32::from(packet_data[0]));
        assert_eq!(packet.payload(), &packet_data[1..]);
    }
}
