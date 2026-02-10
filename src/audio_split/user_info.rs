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
    pub const AUDIO_LOADED: &str = "Audio file loaded.";
    pub const SPLIT_POINTS_DETECTED: &str = "Detected {} potential split points.";
}

pub mod warning {
    pub const NO_AUDIO_LOADED: &str = "No audio file loaded. Please open an audio file first.";
    pub const NO_SPLIT_POINT_SELECTED: &str = "No split point selected. Please analyze the audio first and then select split points to cut.";
    pub const NO_SPLIT_POINTS_FOUND: &str =
        "No Split Points found. Please try to increase the db threshold or decrease duration.";
}
