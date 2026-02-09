use audio_split::*;
use iced::{Task, futures::StreamExt, window};
use iced_runtime::task::into_stream;
use iced_test::simulator;
pub async fn execute_task(task: Task<Message>) -> Vec<Message> {
    let mut messages = Vec::new();

    if let Some(mut stream) = into_stream(task) {
        while let Some(action) = stream.next().await {
            if let iced_runtime::Action::Output(message) = action {
                messages.push(message);
            }
        }
    }
    messages
}

pub async fn execute_tasks(task: Task<Message>, audio_split: &mut AudioSplit) {
    let mut messages = execute_task(task).await;
    while !messages.is_empty() {
        let next_task = audio_split.update(messages.pop().unwrap());
        let mut new_messages = execute_task(next_task).await;
        messages.append(&mut new_messages);
    }
}

#[tokio::test]
async fn test_drop_file() {
    let mut audio_split = AudioSplit::init();

    let task = audio_split.update(Message::WindowEvent(window::Event::FileDropped(
        "media/LibriVox_00.mp3".into(),
    )));
    execute_tasks(task, &mut audio_split).await;

    let mut ui = simulator(audio_split.view());
    ui.find("LibriVox_00_0").unwrap();
}
