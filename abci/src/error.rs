//! tenderdash-abci error definitions.

use flex_error::{define_error, DisplayError};
use tenderdash_proto::abci::response::Value;

define_error! {
    Error {
        Io
            [ DisplayError<std::io::Error> ]
            | _ | { "I/O error" },

        Encode
            [ DisplayError<prost::EncodeError> ]
            | _ | { "error encoding protocol buffer" },

        Decode
            [ DisplayError<prost::DecodeError> ]
            | _ | { "error encoding protocol buffer" },

        ServerConnectionTerminated
            | _ | { "server connection terminated" },

        MalformedServerResponse
            | _ | { "malformed server response" },

        UnexpectedServerResponseType
            {
                expected: String,
                got: Value,
            }
            | e | {
                format_args!("unexpected server response type: expected {0}, but got {1:?}",
                    e.expected, e.got)
            },

        ChannelSend
            | _ | { "channel send error" },

        ChannelRecv
            [ DisplayError<std::sync::mpsc::RecvError> ]
            | _ | { "channel recv error" },
    }
}

impl Error {
    pub fn send<T>(_e: std::sync::mpsc::SendError<T>) -> Error {
        Error::channel_send()
    }
}

// FIXME: I think this should be generated somehow by the define_error! macro above, but it is not
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::io(value)
    }
}
