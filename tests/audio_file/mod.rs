use crate::execute_tasks;
use audio_split::*;
use iced_test::simulator;

#[tokio::test]
async fn open_file_success() {
    let mut audio_split = AudioSplit::init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoAudioLoaded.id()).unwrap();
}

#[tokio::test]
async fn open_file_failed_wrong_media_type() {
    let mut audio_split = AudioSplit::init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/audio_split.gif".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ErrorAudioDecoder.id()).unwrap();
}

#[tokio::test]
async fn test_drop_file() {
    let mut audio_split = AudioSplit::init();

    let task = audio_split.update(Message::WindowEvent(iced::window::Event::FileDropped(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoAudioLoaded.id()).unwrap();
}
