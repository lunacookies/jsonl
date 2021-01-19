use super::Connection;
use mio::net::TcpStream;
use parking_lot::RwLock;
use std::io::{self, BufReader, Read, Write};
use std::sync::Arc;

impl Connection<BufReader<ArcRwLockTcpStream>, ArcRwLockTcpStream> {
    /// Creates a new `Connection` from a Mio TCP stream.
    pub fn new_from_mio_tcp_stream(tcp_stream: TcpStream) -> Self {
        let tcp_stream = ArcRwLockTcpStream(Arc::new(RwLock::new(tcp_stream)));

        Self {
            reader: BufReader::new(tcp_stream.clone()),
            writer: tcp_stream,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArcRwLockTcpStream(Arc<RwLock<TcpStream>>);

impl Read for ArcRwLockTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.write().read(buf)
    }
}

impl Write for ArcRwLockTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.write().flush()
    }
}
