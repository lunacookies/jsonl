#![warn(rust_2018_idioms, missing_debug_implementations)]

mod connection;
mod errors;

pub use connection::Connection;
pub use errors::{ReadError, WriteError};

use std::io::{BufRead, Write};

// Reads a line from the reader and deserializes it into a given type.
pub fn read<R: BufRead, T: serde::de::DeserializeOwned>(mut reader: R) -> Result<T, ReadError> {
    let mut buf = String::new();
    reader.read_line(&mut buf).map_err(ReadError::Io)?;

    Ok(serde_json::from_str(&buf).map_err(ReadError::Deserialize)?)
}

// Writes a given value to the writer, serializing it into JSON.
pub fn write<W: Write, T: serde::Serialize>(mut writer: W, t: &T) -> Result<(), WriteError> {
    // We use to_string here instead of to_vec because it verifies that the JSON is valid UTF-8,
    // which is required by the JSON Lines specification (https://jsonlines.org).
    let json = serde_json::to_string(t).map_err(WriteError::Serialize)?;

    writer.write_all(json.as_bytes()).map_err(WriteError::Io)?;
    writer.write_all(b"\n").map_err(WriteError::Io)?;

    Ok(())
}
