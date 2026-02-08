use crate::audio_split::AudioSplit;

pub mod audio_split;
fn main() -> iced::Result {
    iced::application(AudioSplit::init, AudioSplit::update, AudioSplit::view)
        .subscription(AudioSplit::subscription)
        .run()
}
