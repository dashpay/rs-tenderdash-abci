//! Encoding/decoding mechanisms for ABCI requests and responses.
//!
//! Implements the [Tendermint Socket Protocol][tsp].
//!
//! [tsp]: https://github.com/tendermint/tendermint/blob/v0.34.x/spec/abci/client-server.md#tsp

use std::{fmt::Debug, sync::Arc};

use bytes::{Buf, BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use prost::Message;
use proto::abci::{Request, Response};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex,
};
use tokio_util::{
    codec::{Decoder, Encoder},
    net::Listener,
};

use super::ServerRuntime;
use crate::{proto, CancellationToken, Error};

/// The maximum number of bytes we expect in a varint. We use this to check if
/// we're encountering a decoding error for a varint.
pub const MAX_VARINT_LENGTH: usize = 16;

pub struct Codec {
    request_rx: Receiver<Request>,
    response_tx: Sender<Response>,
}

impl<'a> Codec {
    pub(crate) fn new<L>(
        listener: Arc<Mutex<L>>,
        cancel: CancellationToken,
        runtime: &ServerRuntime,
    ) -> Self
    where
        L: Listener + Send + Sync + 'static,
        L::Addr: Send + Debug,
        L::Io: Send,
    {
        let (request_tx, request_rx) = mpsc::channel::<proto::abci::Request>(1);
        let (response_tx, response_rx) = mpsc::channel::<proto::abci::Response>(1);

        runtime
            .handle
            .spawn(Self::worker(listener, request_tx, response_rx, cancel));

        Self {
            request_rx,
            response_tx,
        }
    }

    /// Worker that bridges data between async streams and sync processing code.
    ///
    /// ## Error handling
    ///
    /// Any error will cause disconnect
    async fn worker<L>(
        listener: Arc<Mutex<L>>,
        request_tx: Sender<proto::abci::Request>,
        mut response_rx: Receiver<proto::abci::Response>,
        cancel: CancellationToken,
    ) where
        L: Listener + Send + Sync,
        L::Addr: Debug,
    {
        let mut listener = listener.lock().await;
        tracing::trace!("listening for new connection");

        let (stream, address) = tokio::select! {
            conn = listener.accept() => match conn {
                Ok(r) => r,
                Err(error) => {
                    tracing::error!(?error, "cannot accept connection");
                    cancel.cancel();
                    return;
                },
            },
            _ = cancel.cancelled() => return,
        };

        tracing::info!(?address, "accepted connection");

        let stream = Box::pin(stream);
        let mut codec = tokio_util::codec::Framed::new(stream, Coder {});

        loop {
            tokio::select! {
                request = codec.next() => match request {
                    Some(Ok(i)) => {
                        if let Err(error) = request_tx.send(i).await {
                            tracing::error!(?error, "unable to forward request for processing");
                            cancel.cancel();
                        }
                    },
                    Some(Err(error)) => {
                        tracing::error!(?error, "unable to parse request");
                        cancel.cancel();
                    },
                    None => {
                        tracing::warn!("client connection terminated");
                        cancel.cancel();
                    },
                },
                response = response_rx.recv() => match response{
                    Some(msg) => {
                        if let Err(error) =   codec.send(msg).await {
                            tracing::error!(?error, "unable to send response to tenderdash");
                            cancel.cancel();
                        }
                    },
                    None => {
                        tracing::warn!("client connection terminated");
                        cancel.cancel();
                    }
                },
                _ = cancel.cancelled() => {
                    tracing::debug!("codec worker shutting down");
                    return; // stop processing
                }
            }
        }
    }

    pub fn next(&mut self) -> Option<Request> {
        self.request_rx.blocking_recv()
    }

    pub fn send(&self, value: Response) -> Result<(), Error> {
        self.response_tx
            .blocking_send(value)
            .map_err(|e| Error::Async(e.to_string()))
    }
}

pub struct Coder;

impl Decoder for Coder {
    type Error = Error;
    type Item = proto::abci::Request;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let src_len = src.len();

        let mut tmp = src.clone().freeze();
        let encoded_len = match prost::encoding::decode_varint(&mut tmp) {
            Ok(len) => len,
            // We've potentially only received a partial length delimiter
            Err(_) if src_len <= MAX_VARINT_LENGTH => return Ok(None),
            Err(e) => return Err(e.into()),
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
}

impl Encoder<proto::abci::Response> for Coder {
    type Error = Error;

    fn encode(
        &mut self,
        message: proto::abci::Response,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let mut buf = BytesMut::new();
        message.encode(&mut buf)?;

        let buf = buf.freeze();
        prost::encoding::encode_varint(buf.len() as u64, dst);
        dst.put(buf);
        Ok(())
    }
}
