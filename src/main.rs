use crate::audio_split::{
    AudioSplit,
    audio_player::{AudioPlayer, RodioPlayer},
};

pub mod audio_split;
fn main() -> iced::Result {
    iced::application(
        || AudioSplit::init(RodioPlayer::init().unwrap()),
        AudioSplit::update,
        AudioSplit::view,
    )
    .subscription(AudioSplit::subscription)
    .run()
}
