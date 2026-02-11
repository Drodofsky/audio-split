use std::{fmt, num::ParseFloatError, sync::Arc};

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
    IO(Arc<std::io::Error>),
    Parsing(ParseFloatError),
    NegativeDuration,
}
impl From<rodio::decoder::DecoderError> for Error {
    fn from(value: rodio::decoder::DecoderError) -> Self {
        Error::new(ErrorKind::AudioDecoder(value), DebugId::ErrorAudioDecoder)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::new(ErrorKind::IO(Arc::new(value)), DebugId::ErrorIO)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Error::new(ErrorKind::Parsing(value), DebugId::ErrorParseFloat)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::AudioDecoder(_) => write!(f, "failed to decode audio file"),
            ErrorKind::IO(io) => write!(f, "{}", io),
            ErrorKind::Parsing(_) => write!(
                f,
                "failed to parse float; please check duration and threshold"
            ),
            ErrorKind::NegativeDuration => write!(f, "a negative duration value is not allowed"),
        }
    }
}

impl std::error::Error for Error {}
