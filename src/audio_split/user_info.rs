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
    pub const NO_AUDIO_LOADED: &str = "No audio file loaded. Please open an audio file first.";
    pub const NO_SPLICE_SELECTED: &str =
        "No splice selected. Please analyze the audio first and then select splices to cut.";
}
