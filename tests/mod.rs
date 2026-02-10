mod analyze;
mod audio_file;
mod play_pause;
mod split;
use std::sync::Arc;

use audio_split::{audio_player::AudioPlayer, error::Error, *};
use iced::{Task, futures::StreamExt};
use iced_runtime::task::into_stream;
use rodio::{Sink, queue::SourcesQueueOutput};
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

pub async fn execute_tasks<P: AudioPlayer>(task: Task<Message>, audio_split: &mut AudioSplit<P>) {
    let mut messages = execute_task(task).await;
    while !messages.is_empty() {
        let next_task = audio_split.update(messages.pop().unwrap());
        let mut new_messages = execute_task(next_task).await;
        messages.append(&mut new_messages);
    }
}

pub struct TestPlayer {
    sink: Arc<rodio::Sink>,
    _source_queue_output: SourcesQueueOutput,
}

impl AudioPlayer for TestPlayer {
    fn init() -> Result<Self, Error> {
        let (sink, source_queue_output) = Sink::new();
        Ok(Self {
            sink: Arc::new(sink),
            _source_queue_output: source_queue_output,
        })
    }
    fn get_sink(&self) -> Arc<Sink> {
        self.sink.clone()
    }
}

pub fn init() -> AudioSplit<TestPlayer> {
    AudioSplit::init(TestPlayer::init().unwrap())
}
