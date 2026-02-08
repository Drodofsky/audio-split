use std::{fmt, fs::File, path::PathBuf, sync::Arc, time::Duration};

use iced::{Element, Length, Subscription, Task, widget};
use rfd::AsyncFileDialog;
use rodio::{Decoder, Source};

use crate::audio_split::error::Error;
mod error;

#[derive(Debug, Default)]
pub struct AudioSplit {
    audio: Option<Audio>,
    is_playing: bool,
}

impl AudioSplit {
    pub fn init() -> Self {
        Self::default()
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenAudioFileDialog => {
                Task::perform(open_audio_file_dialog(), Message::AudioFilePathLoaded)
            }
            Message::AudioFilePathLoaded(path) => {
                if let Some(path) = path {
                    Task::perform(open_audio_file(path), Message::AudioLoaded)
                } else {
                    Task::none()
                }
            }
            Message::AudioLoaded(audio) => {
                self.audio = Some(audio.unwrap());
                self.is_playing = true;
                Task::none()
            }
            Message::AudioSpanPositionUpdate(id, pos) => {
                self.audio.as_mut().unwrap().set_pos(id, pos);
                Task::none()
            }
            Message::Tick => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.update_position_info();
                }
                Task::none()
            }
            Message::Pause => {
                self.is_playing = false;
                if let Some(audio) = self.audio.as_mut() {
                    audio.set_pause();
                }
                Task::none()
            }
            Message::Play => {
                self.is_playing = true;
                if let Some(audio) = self.audio.as_mut() {
                    audio.set_play();
                }
                Task::none()
            }
            Message::Split => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.split();
                }
                Task::none()
            }
            Message::DeleteAudioSpan(id) => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.delete_span(id);
                }
                Task::none()
            }
            Message::SpanTextUpdate(id, text) => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.update_span_text(id, text);
                }
                Task::none()
            }
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        widget::column![self.view_top(), self.view_center()].into()
    }
    fn view_top(&self) -> Element<'_, Message> {
        widget::row![
            widget::button("open audio file").on_press(Message::OpenAudioFileDialog),
            if self.is_playing {
                widget::button("pause").on_press(Message::Pause)
            } else {
                widget::button("play").on_press(Message::Play)
            },
            widget::button("split").on_press(Message::Split)
        ]
        .into()
    }
    fn view_center(&self) -> Element<'_, Message> {
        widget::container(if let Some(audio) = &self.audio {
            audio.view()
        } else {
            widget::text("Please open an audio file").into()
        })
        .center(Length::Fill)
        .into()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }
}

#[derive(Clone)]
pub struct Audio {
    sink: Arc<rodio::Sink>,
    _stream_handle: Arc<rodio::OutputStream>,
    spans: Vec<AudioSpan>,
    file_name: String,
    index_counter: u32,
}

#[derive(Debug, Clone)]
pub struct AudioSpan {
    id: u32,
    start: Duration,
    end: Duration,
    name: String,
    position: f32,
}

impl AudioSpan {
    pub fn new(id: u32, start: Duration, end: Duration, name: String) -> Self {
        Self {
            id,
            start,
            end,
            name,
            position: start.as_secs_f32(),
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column![
                widget::slider(
                    self.start.as_secs_f32()..=self.end.as_secs_f32(),
                    self.position,
                    |p| Message::AudioSpanPositionUpdate(self.id, p)
                ),
                widget::text_input("", &self.name)
                    .on_input(|t| Message::SpanTextUpdate(self.id, t)),
                widget::button("delete")
                    .style(widget::button::danger)
                    .on_press(Message::DeleteAudioSpan(self.id))
            ]
            .spacing(5),
        )
        .padding(5)
        .width(self.calc_slider_length())
        .into()
    }
    pub fn set_pos(&mut self, pos: f32) {
        if pos <= self.start.as_secs_f32() {
            self.position = self.start.as_secs_f32();
        } else if pos >= self.end.as_secs_f32() {
            self.position = self.end.as_secs_f32();
        } else {
            self.position = pos;
        }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn contains(&self, duration: Duration) -> bool {
        self.start <= duration && self.end >= duration
    }
    fn calc_slider_length(&self) -> Length {
        let size = f32::max(
            (self.end.as_secs_f32() - self.start.as_secs_f32()) * 10.,
            100.,
        );
        Length::Fixed(size)
    }
}

impl Audio {
    pub fn view(&self) -> Element<'_, Message> {
        let mut row = widget::Row::new();
        for span in &self.spans {
            row = row.push(span.view());
        }

        widget::scrollable(row.padding(5))
            .horizontal()
            .width(Length::Fill)
            .into()
    }
    pub fn set_pos(&mut self, span_id: u32, pos: f32) {
        self.spans
            .iter_mut()
            .find(|s| s.id() == span_id)
            .map(|s| s.set_pos(pos));
        self.sink.try_seek(Duration::from_secs_f32(pos)).unwrap();
    }
    pub fn update_position_info(&mut self) {
        let pos = self.sink.get_pos().as_secs_f32();
        self.spans.iter_mut().for_each(|s| s.set_pos(pos));
    }
    pub fn set_play(&mut self) {
        self.sink.play();
    }
    pub fn set_pause(&mut self) {
        self.sink.pause();
    }
    pub fn split(&mut self) {
        let pos = self.sink.get_pos();
        let (index, span) = self
            .spans
            .iter()
            .enumerate()
            .find(|(_, s)| s.contains(pos))
            .unwrap();
        let start = span.start;
        let end = span.end;
        let name = span.name.clone();
        let id = span.id;
        self.spans.remove(index);
        self.spans
            .insert(index, AudioSpan::new(id, start, pos, name));
        self.index_counter += 1;
        self.spans.insert(
            index + 1,
            AudioSpan::new(
                self.index_counter,
                pos,
                end,
                format!("{}_{}", self.file_name, self.index_counter),
            ),
        );
    }
    pub fn delete_span(&mut self, id: u32) {
        let res = self.spans.iter().enumerate().find(|(_, s)| id == s.id());
        if let Some((i, _)) = res {
            self.spans.remove(i);
        }
    }
    pub fn update_span_text(&mut self, id: u32, text: String) {
        if let Some(span) = self.get_span_mut(id) {
            span.name = text;
        }
    }
    fn get_span_mut(&mut self, id: u32) -> Option<&mut AudioSpan> {
        self.spans.iter_mut().find(|s| s.id() == id)
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenAudioFileDialog,
    Tick,
    AudioFilePathLoaded(Option<String>),
    AudioLoaded(Result<Audio, Error>),
    AudioSpanPositionUpdate(u32, f32),
    Pause,
    Play,
    Split,
    DeleteAudioSpan(u32),
    SpanTextUpdate(u32, String),
}

pub async fn open_audio_file_dialog() -> Option<String> {
    AsyncFileDialog::new()
        .set_title("Open Audio File")
        .pick_file()
        .await
        .and_then(|h| h.path().to_str().map(|s| s.to_string()))
}

pub async fn open_audio_file(path: String) -> Result<Audio, Error> {
    tokio::task::spawn_blocking(|| {
        let file_name: String = PathBuf::from(&path)
            .file_prefix()
            .unwrap()
            .display()
            .to_string();
        let file = File::open(path).unwrap();
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
        let sink = rodio::Sink::connect_new(stream_handle.mixer());
        let source = Decoder::try_from(file).unwrap();
        let length = source.total_duration().unwrap();
        sink.append(source);
        let spans = vec![AudioSpan::new(
            0,
            Duration::new(0, 0),
            length,
            format!("{file_name}_0"),
        )];

        Ok(Audio {
            sink: Arc::new(sink),
            _stream_handle: Arc::new(stream_handle),
            spans,
            file_name,
            index_counter: 0,
        })
    })
    .await
    .unwrap()
}

impl fmt::Debug for Audio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Audio")
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::audio_split::AudioSpan;

    #[test]
    fn contains_position() {
        let span = AudioSpan::new(
            0,
            Duration::from_secs_f32(12.5),
            Duration::from_secs_f32(34.6),
            String::new(),
        );
        assert!(span.contains(Duration::from_secs_f32(12.6)))
    }
}
