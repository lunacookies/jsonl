#![warn(missing_debug_implementations)]

use std::io::{self, BufRead, BufReader, Stdin, Stdout, Write};
use std::process::{Child, ChildStdin, ChildStdout};

/// A connection that allows reading from a source and writing to a sink, both using JSON Lines.
#[derive(Debug)]
pub struct Connection<Source: BufRead, Sink: Write> {
    source: Source,
    sink: Sink,
}

impl<Source: BufRead, Sink: Write> Connection<Source, Sink> {
    pub fn new(source: Source, sink: Sink) -> Self {
        Self { source, sink }
    }
}

impl<'a> Connection<BufReader<&'a mut ChildStdout>, &'a mut ChildStdin> {
    // Creates a new `Connection` that uses the `stdin` of a child process as the sink and the child
    // process’ `stdout` as the source. This facilitates communication with this child process by
    // passing data into its `stdin` and reading from its `stdout`.
    pub fn new_from_child(child: &'a mut Child) -> Option<Self> {
        let stdin = child.stdin.as_mut()?;
        let stdout = child.stdout.as_mut()?;

        Some(Self {
            source: BufReader::new(stdout),
            sink: stdin,
        })
    }
}

impl Connection<BufReader<Stdin>, Stdout> {
    // Creates a `Connection` from the stdio of the current process – `stdin` is used as the source
    // and `stdout` is used as the sink.
    pub fn new_from_stdio() -> Self {
        Self {
            source: BufReader::new(io::stdin()),
            sink: io::stdout(),
        }
    }
}

impl<Source: BufRead, Sink: Write> Connection<Source, Sink> {
    // Receives a message from the source and deserializes it into a given type.
    pub fn recv_message<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, RecvError> {
        let mut buf = String::new();
        self.source.read_line(&mut buf).map_err(RecvError::Read)?;

        Ok(serde_json::from_str(&buf).map_err(RecvError::Deserialize)?)
    }

    // Sends a given value to the sink, serializing it into JSON.
    pub fn send_message<T: serde::Serialize>(&mut self, t: &T) -> Result<(), SendError> {
        // We use to_string here instead of to_vec because it verifies that the JSON is valid UTF-8,
        // which is required by the JSON Lines specification (https://jsonlines.org).
        let json = serde_json::to_string(t).map_err(SendError::Serialize)?;

        self.sink
            .write_all(json.as_bytes())
            .map_err(SendError::Write)?;

        self.sink.write_all(b"\n").map_err(SendError::Write)?;

        Ok(())
    }
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
