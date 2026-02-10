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
