use crate::audio_split::{debug_id::DebugId, error::Error};

#[derive(Debug, Clone, Default)]
pub enum UserInfo {
    #[default]
    None,
    Info(String, DebugId),
    Waring(String, DebugId),
    Error(Error),
}

pub mod info {
    pub const AUDIO_LOADED: &str = "audio file loaded";
}

pub mod warning {
    pub const NO_AUDIO_LOADED: &str = "no audio file loaded";
}
