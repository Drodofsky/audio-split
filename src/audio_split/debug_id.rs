use iced::widget::Id;
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, Copy, IntoStaticStr)]
pub enum DebugId {
    InfoAudioLoaded,
    WarningNoAudioLoaded,
    ErrorAudioDecoder,
}

impl From<DebugId> for Id {
    fn from(value: DebugId) -> Self {
        Id::new(value.into())
    }
}
impl DebugId {
    pub fn id(self) -> Id {
        self.into()
    }
}
