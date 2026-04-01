use crate::audio_split::{
    AudioSplit,
    audio_player::{AudioPlayer, RodioPlayer},
};
use iced::window;

pub mod audio_split;
fn main() -> iced::Result {
    iced::application(
        || AudioSplit::init(RodioPlayer::init().unwrap()),
        AudioSplit::update,
        AudioSplit::view,
    )
    .subscription(AudioSplit::subscription)
    .title(AudioSplit::title)
    .window(window::Settings {
        icon: Some(
            window::icon::from_rgba(include_bytes!("../media/icon.rgba").to_vec(), 256, 256)
                .unwrap(),
        ),
        platform_specific: window::settings::PlatformSpecific {
            application_id: "audio-split".to_string(),
            ..Default::default()
        },

        ..Default::default()
    })
    .run()
}
