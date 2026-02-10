mod audio_file;
mod play_pause;
use audio_split::*;
use iced::{Task, futures::StreamExt};
use iced_runtime::task::into_stream;
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
