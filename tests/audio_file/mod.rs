use crate::{execute_tasks, init};
use audio_split::*;
use iced_test::simulator;

#[tokio::test]
async fn open_file_success() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoAudioLoaded.id()).unwrap();
}

#[tokio::test]
async fn open_file_failed_wrong_media_type() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/audio_split.gif".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ErrorAudioDecoder.id()).unwrap();
}
#[tokio::test]
async fn open_file_failed_wrong_media_path() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/no_a_file.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ErrorIO.id()).unwrap();
}

#[tokio::test]
async fn test_drop_file() {
    let mut audio_split = init();

    let task = audio_split.update(Message::WindowEvent(iced::window::Event::FileDropped(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoAudioLoaded.id()).unwrap();
}

#[tokio::test]
async fn no_audio_file_loaded_play_btn() {
    let mut audio_split = init();
    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonPlay.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::WarningNoAudioLoaded.id()).unwrap();
}
#[tokio::test]
async fn no_audio_file_loaded_analyze_btn() {
    let mut audio_split = init();
    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonAnalyze.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::WarningNoAudioLoaded.id()).unwrap();
}

#[tokio::test]
async fn no_audio_file_loaded_split_btn() {
    let mut audio_split = init();
    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonSplit.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::WarningNoAudioLoaded.id()).unwrap();
}

#[tokio::test]
async fn no_audio_file_loaded_export_btn() {
    let mut audio_split = init();
    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonExport.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::WarningNoAudioLoaded.id()).unwrap();
}
