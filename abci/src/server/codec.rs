//! Encoding/decoding mechanisms for ABCI requests and responses.
//!
//! Implements the [Tendermint Socket Protocol][tsp].
//!
//! [tsp]: https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/client-server.md#tsp

use std::io;

use bytes::{Buf, BufMut, BytesMut};
use prost::{DecodeError, EncodeError, Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{proto, Error, ServerCancel};

/// The maximum number of bytes we expect in a varint. We use this to check if
/// we're encountering a decoding error for a varint.
pub const MAX_VARINT_LENGTH: usize = 16;

/// Allows for iteration over `S` to produce instances of `I`, as well as
/// sending instances of `O`.
pub struct Codec<S> {
    stream: S,
    // Long-running read buffer
    read_buf: BytesMut,
    // Fixed-length read window
    read_window: Vec<u8>,
    write_buf: BytesMut,
    cancel: ServerCancel,
}

impl<S> Codec<S> {
    pub fn new(stream: S, read_buf_size: usize, cancel: ServerCancel) -> Self {
        Self {
            stream,
            read_buf: BytesMut::new(),
            read_window: vec![0_u8; read_buf_size],
            write_buf: BytesMut::new(),
            cancel,
        }
    }
}

impl<S> Codec<S>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    pub(crate) async fn receive(&mut self) -> Result<Option<proto::abci::Request>, Error> {
        loop {
            // Try to decode an incoming message from our buffer first
            if let Some(incoming) = decode_length_delimited(&mut self.read_buf)? {
                return Ok(Some(incoming));
            }
            let cancel = &self.cancel;
            // If we don't have enough data to decode a message, try to read more
            let bytes_read = tokio::select! {
                b = self.stream.read(self.read_window.as_mut()) => b,
                _ = cancel.cancelled() => return Err(Error::Cancelled()),
            }?;
            if bytes_read == 0 {
                // The underlying stream terminated
                return Ok(None);
            }
            self.read_buf
                .extend_from_slice(&self.read_window[..bytes_read]);
        }
    }

    /// Send a message using this codec.
    pub(crate) async fn send(&mut self, message: proto::abci::Response) -> Result<(), Error> {
        encode_length_delimited(message, &mut self.write_buf)?;
        while !self.write_buf.is_empty() {
            let bytes_written = self.stream.write(self.write_buf.as_ref()).await?;

            if bytes_written == 0 {
                return Err(Error::Connection(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write to underlying stream",
                )));
            }
            self.write_buf.advance(bytes_written);
        }

        self.stream.flush().await?;

        Ok(())
    }
}

/// Encode the given message with a length prefix.
pub fn encode_length_delimited<B>(
    message: proto::abci::Response,
    mut dst: &mut B,
) -> Result<(), EncodeError>
where
    B: BufMut,
{
    let mut buf = BytesMut::new();
    message.encode(&mut buf)?;

    let buf = buf.freeze();
    prost::encoding::encode_varint(buf.len() as u64, &mut dst);
    dst.put(buf);
    Ok(())
}

/// Attempt to decode a message of type `M` from the given source buffer.
pub fn decode_length_delimited(
    src: &mut BytesMut,
) -> Result<Option<proto::abci::Request>, DecodeError> {
    let src_len = src.len();
    let mut tmp = src.clone().freeze();
    let encoded_len = match prost::encoding::decode_varint(&mut tmp) {
        Ok(len) => len,
        // We've potentially only received a partial length delimiter
        Err(_) if src_len <= MAX_VARINT_LENGTH => return Ok(None),
        Err(e) => return Err(e),
    };
    let remaining = tmp.remaining() as u64;
    if remaining < encoded_len {
        // We don't have enough data yet to decode the entire message
        Ok(None)
    } else {
        let delim_len = src_len - tmp.remaining();
        // We only advance the source buffer once we're sure we have enough
        // data to try to decode the result.
        src.advance(delim_len + (encoded_len as usize));

        let mut result_bytes = BytesMut::from(tmp.split_to(encoded_len as usize).as_ref());
        let res = proto::abci::Request::decode(&mut result_bytes)?;

        Ok(Some(res))
    }
}
