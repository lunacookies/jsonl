use std::io;

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
