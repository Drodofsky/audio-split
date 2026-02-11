use iced::widget::Id;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, Copy, IntoStaticStr)]
pub enum DebugId {
    InfoAudioLoaded,
    InfoSplitPointsSelected(usize),
    WarningNoAudioLoaded,
    WarningNoSplitPointFound,
    WarningNoSplitPointSelected,
    ErrorAudioDecoder,
    ErrorIO,
    ErrorParseFloat,
    ErrorNegativeDuration,
    ButtonOpen,
    ButtonPlay,
    ButtonPause,
    ButtonAnalyze,
    ButtonSplit,
    ButtonExport,
    ButtonDelete(u32),
    TextInputThreshold,
    TextInputDuration,
}

impl From<DebugId> for Id {
    fn from(value: DebugId) -> Self {
        match value {
            DebugId::ButtonDelete(v) => Id::from(format!("ButtonDelete:{v}")),
            DebugId::InfoSplitPointsSelected(v) => Id::from(format!("InfoSplitPointsSelected:{v}")),
            _ => Id::new(value.into()),
        }
    }
}
impl DebugId {
    pub fn id(self) -> Id {
        self.into()
    }
}
