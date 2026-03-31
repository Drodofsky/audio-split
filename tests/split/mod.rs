use std::time::Duration;

use crate::{execute_tasks, init};
use audio_split::*;
use iced_test::simulator;

#[tokio::test]
async fn warn_when_no_splice_is_selected() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.click(DebugId::ButtonSplit.id()).unwrap();

    for message in ui.into_messages() {
        let task = audio_split.update(message);
        execute_tasks(task, &mut audio_split).await;
    }
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::WarningNoSplitPointSelected.id()).unwrap();
}

#[tokio::test]
async fn split_one_point() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let task = audio_split.update(Message::ClickSplitPoint(Duration::from_secs_f32(22.10245)));
    execute_tasks(task, &mut audio_split).await;

    let task = audio_split.update(Message::Split);

    execute_tasks(task, &mut audio_split).await;
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoSplits(1).id()).unwrap();
}

#[tokio::test]
async fn split_two_points() {
    let mut audio_split = init();

    let task = audio_split.update(Message::AudioFilePathLoaded(Some(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let task = audio_split.update(Message::ClickSplitPoint(Duration::from_secs_f32(22.10245)));
    execute_tasks(task, &mut audio_split).await;
    let task = audio_split.update(Message::ClickSplitPoint(Duration::from_secs_f32(33.10245)));
    execute_tasks(task, &mut audio_split).await;

    let task = audio_split.update(Message::Split);

    execute_tasks(task, &mut audio_split).await;
    let mut ui = simulator(audio_split.view());
    ui.find(DebugId::InfoSplits(2).id()).unwrap();
}
