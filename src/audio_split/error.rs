use std::fmt;

use super::debug_id::DebugId;
#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    id: DebugId,
}

impl Error {
    pub fn new(kind: ErrorKind, id: DebugId) -> Self {
        Error { kind, id }
    }
    pub fn id(&self) -> DebugId {
        self.id
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    AudioDecoder(rodio::decoder::DecoderError),
}
impl From<rodio::decoder::DecoderError> for Error {
    fn from(value: rodio::decoder::DecoderError) -> Self {
        Error::new(ErrorKind::AudioDecoder(value), DebugId::ErrorAudioDecoder)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::AudioDecoder(_) => write!(f, "failed to decode audio file"),
        }
    }
}

impl std::error::Error for Error {}
