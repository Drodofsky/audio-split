use iced::widget::Id;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, Copy, IntoStaticStr)]
pub enum DebugId {
    InfoAudioLoaded,
    WarningNoAudioLoaded,
    ErrorAudioDecoder,
    ButtonOpen,
    ButtonPlay,
    ButtonPause,
    ButtonAnalyze,
    ButtonSplit,
    ButtonExport,
    ButtonDelete(u32),
}

impl From<DebugId> for Id {
    fn from(value: DebugId) -> Self {
        match value {
            DebugId::ButtonDelete(v) => Id::from(format!("ButtonDelete:{v}")),
            _ => Id::new(value.into()),
        }
    }
}
impl DebugId {
    pub fn id(self) -> Id {
        self.into()
    }
}
