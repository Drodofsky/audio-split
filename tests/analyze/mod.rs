use crate::{execute_tasks, init};
use audio_split::*;
use iced_test::simulator;

#[tokio::test]
async fn splits_detected() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonAnalyze.id()).unwrap();

    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoSplitPointsSelected(39).id()).unwrap();
}

#[tokio::test]
async fn no_splits_detected() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;
    let task = audio_split.update(Message::UpdateDuration("10.".into()));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonAnalyze.id()).unwrap();

    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::WarningNoSplitPointFound.id()).unwrap();
}
