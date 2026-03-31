use iced::widget::Id;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, Copy, IntoStaticStr)]
pub enum DebugId {
    InfoAudioLoaded,
    InfoSplitPointsDetected(usize),
    InfoSplits(usize),
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
    ButtonUndo,
    TextInputThreshold,
    TextInputDuration,
    Canvas,
}

impl From<DebugId> for Id {
    fn from(value: DebugId) -> Self {
        match value {
            DebugId::ButtonDelete(v) => Id::from(format!("ButtonDelete:{v}")),
            DebugId::InfoSplitPointsDetected(v) => Id::from(format!("InfoSplitPointsSelected:{v}")),
            DebugId::InfoSplits(v) => Id::from(format!("InfoSplits:{v}")),

            _ => Id::new(value.into()),
        }
    }
}
impl DebugId {
    pub fn id(self) -> Id {
        self.into()
    }
}
