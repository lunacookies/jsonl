use std::io::{self, BufRead, BufReader, Stdin, Stdout, Write};
use std::process::{Child, ChildStdin, ChildStdout};

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
    pub fn new_from_stdio() -> Self {
        Self {
            source: BufReader::new(io::stdin()),
            sink: io::stdout(),
        }
    }
}

impl<Source: BufRead, Sink: Write> Connection<Source, Sink> {
    pub fn recv_message<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, MessageIoError> {
        let mut buf = String::new();

        self.source
            .read_line(&mut buf)
            .map_err(MessageIoError::Recv)?;

        Ok(serde_json::from_str(&buf)?)
    }

    pub fn send_message<T: serde::Serialize>(&mut self, t: &T) -> Result<(), MessageIoError> {
        let json = serde_json::to_vec(t)?;
        self.sink.write_all(&json).map_err(MessageIoError::Send)?;
        self.sink.write_all(b"\n").map_err(MessageIoError::Send)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageIoError {
    #[error("failed receiving message from source")]
    Recv(io::Error),
    #[error("failed sending message to sink")]
    Send(io::Error),
    #[error("failed deserializing JSON")]
    Json(#[from] serde_json::Error),
}
