#![warn(rust_2018_idioms, missing_debug_implementations)]

use std::io::{BufRead, Write};

mod connection;
mod errors;

pub use connection::Connection;
pub use errors::{RecvError, SendError};

// Receives a message from the source and deserializes it into a given type.
pub fn recv_message<Source: BufRead, T: serde::de::DeserializeOwned>(
    source: &mut Source,
) -> Result<T, RecvError> {
    let mut buf = String::new();
    source.read_line(&mut buf).map_err(RecvError::Read)?;

    Ok(serde_json::from_str(&buf).map_err(RecvError::Deserialize)?)
}

// Sends a given value to the sink, serializing it into JSON.
pub fn send_message<Sink: Write, T: serde::Serialize>(
    sink: &mut Sink,
    t: &T,
) -> Result<(), SendError> {
    // We use to_string here instead of to_vec because it verifies that the JSON is valid UTF-8,
    // which is required by the JSON Lines specification (https://jsonlines.org).
    let json = serde_json::to_string(t).map_err(SendError::Serialize)?;

    sink.write_all(json.as_bytes()).map_err(SendError::Write)?;
    sink.write_all(b"\n").map_err(SendError::Write)?;

    Ok(())
}
