#![warn(rust_2018_idioms, missing_debug_implementations)]

//! An implementation of JSON Lines for Rust.
//!
//! [JSON Lines](https://jsonlines.org) is a simple format consisting of [JSON](https://json.org)
//! values separated by newlines. Use [`read()`] and [`write()`] to interact wtih readers and
//! writers in the JSON Lines format. Serialization and deserialization is done automatically.
//!
//! See [`Connection`] for situations in which you have both a reader and a writer and would like to
//! bundle them up together.
//!
//! Enable the `tokio` feature to replace the usages of `std` IO primitives with those from Tokio.

mod connection;
mod errors;

pub use connection::Connection;
pub use errors::{ReadError, WriteError};

#[cfg(not(feature = "tokio"))]
mod imp {
    use super::*;
    use std::io::{BufRead, Write};

    /// Reads a line from the reader and deserializes it into a given type.
    pub fn read<R: BufRead, T: serde::de::DeserializeOwned>(mut reader: R) -> Result<T, ReadError> {
        let mut buf = String::new();
        reader.read_line(&mut buf).map_err(ReadError::Io)?;

        Ok(serde_json::from_str(&buf).map_err(ReadError::Deserialize)?)
    }

    /// Writes a given value to the writer, serializing it into JSON.
    pub fn write<W: Write, T: serde::Serialize>(mut writer: W, t: &T) -> Result<(), WriteError> {
        // We use to_string here instead of to_vec because it verifies that the JSON is valid UTF-8,
        // which is required by the JSON Lines specification (https://jsonlines.org).
        let json = serde_json::to_string(t).map_err(WriteError::Serialize)?;

        writer.write_all(json.as_bytes()).map_err(WriteError::Io)?;
        writer.write_all(b"\n").map_err(WriteError::Io)?;

        Ok(())
    }
}

#[cfg(feature = "tokio")]
mod imp {
    use super::*;
    use tokio::io::{AsyncBufRead as BufRead, AsyncBufReadExt, AsyncWrite as Write, AsyncWriteExt};

    /// Reads a line from the reader and deserializes it into a given type.
    pub async fn read<R: BufRead + Unpin, T: serde::de::DeserializeOwned>(
        mut reader: R,
    ) -> Result<T, ReadError> {
        let mut buf = String::new();
        reader.read_line(&mut buf).await.map_err(ReadError::Io)?;

        Ok(serde_json::from_str(&buf).map_err(ReadError::Deserialize)?)
    }

    /// Writes a given value to the writer, serializing it into JSON.
    pub async fn write<W: Write + Unpin, T: serde::Serialize>(
        mut writer: W,
        t: &T,
    ) -> Result<(), WriteError> {
        // We use to_string here instead of to_vec because it verifies that the JSON is valid UTF-8,
        // which is required by the JSON Lines specification (https://jsonlines.org).
        let json = serde_json::to_string(t).map_err(WriteError::Serialize)?;

        writer
            .write_all(json.as_bytes())
            .await
            .map_err(WriteError::Io)?;

        writer.write_all(b"\n").await.map_err(WriteError::Io)?;

        Ok(())
    }
}

pub use imp::*;
