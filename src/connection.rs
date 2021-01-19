#[cfg(not(feature = "tokio"))]
mod imports {
    pub(super) use std::io::{self, BufRead, BufReader, Stdin, Stdout, Write};
    pub(super) use std::net::TcpStream;
    pub(super) use std::process::{Child, ChildStdin, ChildStdout};
}
#[cfg(feature = "tokio")]
mod imports {
    pub(super) use tokio::io::{
        self, AsyncBufRead as BufRead, AsyncWrite as Write, AsyncWriteExt, BufReader, Stdin, Stdout,
    };
    pub(super) use tokio::net::tcp::{ReadHalf, WriteHalf};
    pub(super) use tokio::net::TcpStream;
    pub(super) use tokio::process::{Child, ChildStdin, ChildStdout};
}

use imports::*;

/// Use this type when you have both a reader and writer, and want them to be grouped together.
///
/// There are situations in which you have both a reader and a writer being passed around code,
/// always kept together. This forms what is known as a ‘[data clump]’, and harms code readability.
/// By grouping the two together it makes clear that they are both needed, and prevents mistakes
/// when one is forgotten.
///
/// `Connection` is internally a pair of a reader and a writer, and delegates to [`crate::read`] and
/// [`crate::write`] for [`Connection::read`] and [`Connection::write`] respectively.
///
/// [data clump]: https://youtu.be/DC-pQPq0acs?t=521
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Connection<R: BufRead, W: Write> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> Connection<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }
}

impl<'a> Connection<BufReader<&'a mut ChildStdout>, &'a mut ChildStdin> {
    /// Creates a new `Connection` that uses the `stdin` of a child process as the writer and the
    /// child process’ `stdout` as the reader. This facilitates communication with this child process
    /// by passing data into its `stdin` and reading from its `stdout`.
    pub fn new_from_child(child: &'a mut Child) -> Option<Self> {
        let stdin = child.stdin.as_mut()?;
        let stdout = child.stdout.as_mut()?;

        Some(Self {
            reader: BufReader::new(stdout),
            writer: stdin,
        })
    }
}

impl Connection<BufReader<Stdin>, Stdout> {
    /// Creates a new `Connection` from the stdio of the current process – `stdin` is used as the reader
    /// and `stdout` is used as the writer.
    pub fn new_from_stdio() -> Self {
        Self {
            reader: BufReader::new(io::stdin()),
            writer: io::stdout(),
        }
    }
}

#[cfg(not(feature = "tokio"))]
impl Connection<BufReader<TcpStream>, TcpStream> {
    /// Creates a new `Connection` from a TCP stream.
    pub fn new_from_tcp_stream(tcp_stream: TcpStream) -> io::Result<Self> {
        Ok(Self {
            reader: BufReader::new(tcp_stream.try_clone()?),
            writer: tcp_stream,
        })
    }
}

#[cfg(feature = "tokio")]
impl<'a> Connection<BufReader<ReadHalf<'a>>, WriteHalf<'a>> {
    /// Creates a new `Connection` from a mutable reference to a TCP stream.
    pub fn new_from_tcp_stream(tcp_stream: &'a mut TcpStream) -> io::Result<Self> {
        let (read_half, write_half) = tcp_stream.split();

        Ok(Self {
            reader: BufReader::new(read_half),
            writer: write_half,
        })
    }
}

#[cfg(not(feature = "tokio"))]
impl<R: BufRead, W: Write> Connection<R, W> {
    /// Reads a line from the reader and deserializes it into a given type.
    pub fn read<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, crate::ReadError> {
        crate::read(&mut self.reader)
    }

    /// Writes a given value to the writer, serializing it into JSON.
    pub fn write<T: serde::Serialize>(&mut self, t: &T) -> Result<(), crate::WriteError> {
        crate::write(&mut self.writer, t)
    }

    /// Flushes the contained writer’s buffer.
    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.writer.flush()
    }
}

#[cfg(feature = "tokio")]
impl<R: BufRead + Unpin, W: Write + Unpin> Connection<R, W> {
    /// Reads a line from the reader and deserializes it into a given type.
    pub async fn read<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, crate::ReadError> {
        crate::read(&mut self.reader).await
    }

    /// Writes a given value to the writer, serializing it into JSON.
    pub async fn write<T: serde::Serialize>(&mut self, t: &T) -> Result<(), crate::WriteError> {
        crate::write(&mut self.writer, t).await
    }

    /// Flushes the contained writer’s buffer.
    pub async fn flush(&mut self) -> Result<(), io::Error> {
        self.writer.flush().await
    }
}
