#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Buffer is full")]
    BufferFull,
}
