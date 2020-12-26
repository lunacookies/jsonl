#![warn(missing_debug_implementations)]

use std::io::{self, BufRead, Write};

mod connection;
pub use connection::Connection;

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

/// An error that occurred during the receiving of a message.
#[derive(Debug, thiserror::Error)]
pub enum RecvError {
    #[error("failed reading message data from source")]
    Read(#[from] io::Error),
    #[error("failed deserializing JSON")]
    Deserialize(#[from] serde_json::Error),
}

/// An error that occurred during the sending of a message.
#[derive(Debug, thiserror::Error)]
pub enum SendError {
    #[error("failed writing message data to sink")]
    Write(#[from] io::Error),
    #[error("failed serializing JSON")]
    Serialize(#[from] serde_json::Error),
}
