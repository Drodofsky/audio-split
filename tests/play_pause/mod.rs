use crate::{execute_tasks, init};
use audio_split::*;
use iced_test::simulator;

#[tokio::test]
async fn playing_on_clean_load() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ButtonPause.id()).unwrap();
}
#[tokio::test]
async fn press_pause_while_playing() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonPause.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());

    ui.find(DebugId::ButtonPlay.id()).unwrap();
}

#[tokio::test]
async fn press_play_while_paused() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonPause.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonPlay.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }

    let mut ui = simulator(audio_split.view());

    ui.find(DebugId::ButtonPause.id()).unwrap();
}

#[tokio::test]
async fn auto_playing_when_new_media_loaded() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;
    let mut ui = simulator(audio_split.view());

    ui.click(DebugId::ButtonPause.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    {
        let mut ui = simulator(audio_split.view());
        ui.find(DebugId::ButtonPlay.id()).unwrap();
    }
    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ButtonPause.id()).unwrap();
}
