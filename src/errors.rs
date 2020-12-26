use std::io;

/// An error that occurred during reading.
#[derive(Debug, thiserror::Error)]
pub enum ReadError {
    #[error("failed reading data from reader")]
    Io(#[from] io::Error),
    #[error("failed deserializing JSON")]
    Deserialize(#[from] serde_json::Error),
}

/// An error that occurred during writing.
#[derive(Debug, thiserror::Error)]
pub enum WriteError {
    #[error("failed writing data to writer")]
    Io(#[from] io::Error),
    #[error("failed serializing JSON")]
    Serialize(#[from] serde_json::Error),
}
