use crate::{execute_tasks, init};
use audio_split::*;
use iced::keyboard::Key;
use iced_test::simulator;
#[tokio::test]
async fn duration_not_a_number() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::TextInputDuration.id()).unwrap();
    ui.typewrite("b");
    ui.click(DebugId::ButtonAnalyze.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ErrorParseFloat.id()).unwrap();
}

#[tokio::test]
async fn threshold_not_a_number() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::TextInputThreshold.id()).unwrap();
    ui.typewrite("b");
    ui.click(DebugId::ButtonAnalyze.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ErrorParseFloat.id()).unwrap();
}

#[tokio::test]
async fn duration_not_negative() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::TextInputDuration.id()).unwrap();
    for _ in 0..5 {
        ui.tap_key(Key::Named(iced::keyboard::key::Named::Backspace));
    }
    ui.typewrite("-3");
    ui.click(DebugId::ButtonAnalyze.id()).unwrap();
    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::ErrorNegativeDuration.id()).unwrap();
}
