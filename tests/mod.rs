mod analyze;
mod audio_file;
mod play_pause;
mod split;
mod text_input;
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::{self, JoinHandle},
    time::Duration,
};

use audio_split::{audio_player::AudioPlayer, error::Error, *};
use iced::{Task, futures::StreamExt};
use iced_runtime::task::into_stream;
use rodio::Sink;
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
    stop: Arc<AtomicBool>,
    drain_thread: Option<JoinHandle<()>>,
}

impl AudioPlayer for TestPlayer {
    fn init() -> Result<Self, Error> {
        let (sink, mut source_queue_output) = Sink::new();
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let drain_thread = thread::spawn(move || {
            // Tune these:
            let block_samples = 1024usize;
            let nap = Duration::from_millis(5);

            while !stop2.load(std::sync::atomic::Ordering::Relaxed) {
                for _ in 0..block_samples {
                    let _ = source_queue_output.next();
                }
                thread::sleep(nap);
            }
        });
        Ok(Self {
            sink: Arc::new(sink),
            stop,
            drain_thread: Some(drain_thread),
        })
    }
    fn get_sink(&self) -> Arc<Sink> {
        self.sink.clone()
    }
}

impl Drop for TestPlayer {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.drain_thread.take() {
            let _ = h.join();
        }
    }
}

pub fn init() -> AudioSplit<TestPlayer> {
    AudioSplit::init(TestPlayer::init().unwrap())
}
