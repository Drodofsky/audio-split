use std::{fmt, fs::File, path::PathBuf, sync::Arc, time::Duration};

use iced::{Element, Length, Subscription, Task, alignment::Vertical, widget, window::Event};
use rfd::AsyncFileDialog;
use rodio::{Decoder, Source};
use tokio::process::Command;

use crate::audio_split::{analyze::detect_silence, error::Error};
mod analyze;
mod canvas;
mod error;

#[derive(Debug, Default)]
pub struct AudioSplit {
    audio: Option<Audio>,
    is_playing: bool,
    import_path: Option<PathBuf>,
    export_path: Option<PathBuf>,
    threshold: String,
    duration: String,
    extension: Option<PathBuf>,
}

impl AudioSplit {
    pub fn init() -> Self {
        let mut s = Self::default();
        s.duration = "0.3".to_string();
        s.threshold = "-50.0".to_string();
        s
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenAudioFileDialog => Task::perform(
                open_audio_file_dialog(
                    self.import_path
                        .clone()
                        .map(|p| p.parent().map(|p| p.to_path_buf()))
                        .flatten(),
                ),
                Message::AudioFilePathLoaded,
            ),
            Message::OpenExportDialog => Task::perform(
                open_export_folder_dialog(self.export_path.clone()),
                Message::ExportPathLoaded,
            ),
            Message::AudioFilePathLoaded(path) => {
                if let Some(path) = path {
                    self.import_path = Some(path.clone().into());
                    self.extension = PathBuf::from(path.clone()).extension().map(|s| s.into());
                    Task::perform(open_audio_file(path), Message::AudioLoaded)
                } else {
                    Task::none()
                }
            }
            Message::ExportPathLoaded(path) => {
                if let Some(path) = path
                    && let Some(audio) = &self.audio
                {
                    self.export_path = Some(path.clone().into());
                    Task::perform(
                        save_audio_files(
                            self.import_path.clone().unwrap(),
                            path.into(),
                            self.extension.clone().unwrap(),
                            audio.spans.clone(),
                        ),
                        Message::AudioSaved,
                    )
                } else {
                    Task::none()
                }
            }
            Message::AudioSaved(r) => {
                r.unwrap();
                Task::none()
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
                    if let Some(last) = audio.spans.last()
                        && let Some(sub) = (last.end.checked_sub(audio.sink.get_pos()))
                        && sub.as_millis() < 300
                    {
                        println!("end");
                        audio.set_pos(0, 0.0);
                    }
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
            Message::WindowEvent(e) => match e {
                Event::FileDropped(f) => {
                    self.import_path = Some(f.clone());
                    Task::perform(open_audio_file(f), Message::AudioLoaded)
                }
                _ => Task::none(),
            },
            Message::Analyze => {
                if let Some(path) = self.import_path.clone() {
                    Task::perform(
                        detect_silence(
                            path,
                            self.threshold.parse().unwrap(),
                            Duration::from_secs_f32(self.duration.parse().unwrap()),
                        ),
                        |a| Message::Analyzed(a),
                    )
                } else {
                    Task::none()
                }
            }
            Message::Analyzed(s) => {
                if let Some(audio) = self.audio.as_mut() {
                    Audio::set_splices(&mut audio.spans, s.unwrap());
                }
                Task::none()
            }
            Message::ClickSplice(splice) => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.toggle_selected_splice(splice);
                }
                Task::none()
            }
            Message::UpdateDuration(s) => {
                self.duration = s;
                Task::none()
            }
            Message::UpdateThreshold(s) => {
                self.threshold = s;
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
            widget::text("threshold in dB:"),
            widget::text_input("", &self.threshold).on_input(Message::UpdateThreshold),
            widget::text("duration in sec:"),
            widget::text_input("", &self.duration).on_input(Message::UpdateDuration),
            widget::button("analyze").on_press(Message::Analyze),
            widget::button("split").on_press(Message::Split),
            widget::button("export").on_press(Message::OpenExportDialog),
        ]
        .spacing(5)
        .align_y(Vertical::Center)
        .into()
    }
    fn view_center(&self) -> Element<'_, Message> {
        if let Some(audio) = &self.audio {
            widget::container(widget::column![
                widget::space().height(Length::FillPortion(2)),
                audio.view(),
                widget::space().height(Length::FillPortion(3)),
            ])
        } else {
            widget::container(widget::text("Please open an audio file"))
        }
        .center(Length::Fill)
        .into()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick);
        Subscription::batch([
            tick,
            iced::window::events().map(|f| Message::WindowEvent(f.1)),
        ])
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
    splices: Vec<Duration>,
    selected_splices: Vec<Duration>,
}

impl AudioSpan {
    pub fn new(id: u32, start: Duration, end: Duration, name: String) -> Self {
        Self {
            id,
            start,
            end,
            name,
            position: start.as_secs_f32(),
            splices: Vec::new(),
            selected_splices: Vec::new(),
        }
    }
    pub fn view(&self) -> Element<'_, Message> {
        widget::container(
            widget::column![
                widget::canvas(self).width(self.calc_slider_length()),
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

    pub fn set_pos_and_get_info(&mut self, pos: f32) -> (i8, Duration) {
        if pos <= self.start.as_secs_f32() {
            self.position = self.start.as_secs_f32();
            (-1, self.start)
        } else if pos >= self.end.as_secs_f32() {
            self.position = self.end.as_secs_f32();
            (1, self.start)
        } else {
            self.position = pos;
            (0, self.start)
        }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn contains(&self, duration: Duration) -> bool {
        self.start < duration && self.end > duration
    }
    fn calc_slider_length(&self) -> Length {
        let size = f32::max(
            (self.end.as_secs_f32() - self.start.as_secs_f32()) * 10.,
            100.,
        );
        Length::Fixed(size)
    }
    pub fn insert_splice(&mut self, splice: Duration) -> bool {
        let fits = self.contains(splice);
        if fits {
            self.splices.insert(0, splice);
        }
        fits
    }
    pub fn toggle_splice_selection(&mut self, splice: Duration) -> bool {
        let fits = self.contains(splice);
        if fits {
            if let Some((i, _)) = self
                .selected_splices
                .iter()
                .enumerate()
                .find(|(_, s)| **s == splice)
            {
                self.selected_splices.remove(i);
            } else {
                self.selected_splices.push(splice);
            }
        }
        fits
    }
    pub fn clear_splices(&mut self) {
        self.splices.clear();
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
            .map(|s| s.set_pos_and_get_info(pos));
        self.sink.try_seek(Duration::from_secs_f32(pos)).unwrap();
    }
    pub fn update_position_info(&mut self) {
        let pos = self.sink.get_pos().as_secs_f32();
        let mut found_zero = false;
        let mut found_next = false;
        let mut skip_to = Duration::default();
        for (b, pos) in self.spans.iter_mut().map(|s| s.set_pos_and_get_info(pos)) {
            if b == 0 || found_zero == true {
                found_zero = true;
            } else if b == -1 && found_next == false {
                found_next = true;
                skip_to = pos;
            }
        }
        if found_next {
            println!("skip");
            self.sink.try_seek(skip_to).unwrap()
        }
    }
    pub fn set_play(&mut self) {
        self.sink.play();
    }
    pub fn set_pause(&mut self) {
        self.sink.pause();
    }
    pub fn split(&mut self) {
        let mut splits: Vec<Duration> = self
            .spans
            .iter()
            .map(|s| s.selected_splices.iter())
            .flatten()
            .map(|s| *s)
            .collect();
        splits.sort();
        for split in splits {
            self.split_at(split);
        }
    }
    fn split_at(&mut self, pos: Duration) {
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
        let splices = span.splices.clone();
        let mut span_1 = AudioSpan::new(id, start, pos, name);
        self.index_counter += 1;
        let mut span_2 = AudioSpan::new(
            self.index_counter,
            pos,
            end,
            format!("{}_{}", self.file_name, self.index_counter),
        );
        for splice in splices {
            span_1.insert_splice(splice);
            span_2.insert_splice(splice);
        }

        self.spans.remove(index);
        self.spans.insert(index, span_1);
        self.spans.insert(index + 1, span_2);
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
    // spices must be sorted
    pub fn set_splices(spans: &mut [AudioSpan], mut splices: Vec<Duration>) {
        spans.iter_mut().for_each(|s| s.clear_splices());
        for span in spans.iter_mut().rev() {
            while let Some(last) = splices.last() {
                if span.insert_splice(*last) {
                    splices.pop();
                } else {
                    break;
                }
            }
        }
    }
    pub fn toggle_selected_splice(&mut self, splice: Duration) {
        for span in self.spans.iter_mut() {
            if span.toggle_splice_selection(splice) == true {
                return;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenAudioFileDialog,
    OpenExportDialog,
    Tick,
    AudioFilePathLoaded(Option<String>),
    ExportPathLoaded(Option<String>),
    AudioLoaded(Result<Audio, Error>),
    AudioSpanPositionUpdate(u32, f32),
    Pause,
    Play,
    Split,
    DeleteAudioSpan(u32),
    SpanTextUpdate(u32, String),
    WindowEvent(iced::window::Event),
    Analyze,
    Analyzed(Result<Vec<Duration>, Error>),
    ClickSplice(Duration),
    UpdateDuration(String),
    UpdateThreshold(String),
    AudioSaved(Result<(), Error>),
}

pub async fn open_audio_file_dialog(starting_path: Option<PathBuf>) -> Option<String> {
    let mut dialog = AsyncFileDialog::new().set_title("Open Audio File");
    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }
    dialog
        .pick_file()
        .await
        .and_then(|h| h.path().to_str().map(|s| s.to_string()))
}
pub async fn open_export_folder_dialog(starting_path: Option<PathBuf>) -> Option<String> {
    let mut dialog = AsyncFileDialog::new()
        .set_title("Export Audio Files To Folder")
        .set_can_create_directories(true);
    if let Some(path) = starting_path {
        dialog = dialog.set_directory(path);
    }
    dialog
        .pick_folder()
        .await
        .and_then(|h| h.path().to_str().map(|s| s.to_string()))
}

pub async fn save_audio_files(
    source: PathBuf,
    export_path: PathBuf,
    path_extension: PathBuf,
    spans: Vec<AudioSpan>,
) -> Result<(), Error> {
    for span in spans {
        let mut export_path = export_path.join(span.name);
        export_path.add_extension(&path_extension);
        let base = export_path.parent().unwrap();
        tokio::fs::create_dir_all(base).await.unwrap();
        let output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-nostats")
            .arg("-y")
            .arg("-i")
            .arg(&source)
            .arg("-ss")
            .arg(fmt_duration(span.start))
            .arg("-to")
            .arg(fmt_duration(span.end))
            .arg(&export_path)
            .output()
            .await
            .unwrap();
        assert!(output.status.success());
    }
    Ok(())
}

fn fmt_duration(duration: Duration) -> String {
    format!("{:.6}", duration.as_secs_f32())
}

pub async fn open_audio_file(path: impl Into<PathBuf> + Send + 'static) -> Result<Audio, Error> {
    tokio::task::spawn_blocking(|| {
        let path: PathBuf = path.into();
        let file_name: String = path.file_prefix().unwrap().display().to_string();
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

    use crate::audio_split::{Audio, AudioSpan};

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
    #[test]
    fn set_splices_1() {
        let splices = vec![
            Duration::from_secs_f32(12.5),
            Duration::from_secs_f32(15.5),
            Duration::from_secs_f32(17.5),
            Duration::from_secs_f32(28.5),
        ];
        let control = splices.clone();
        let mut spans = vec![AudioSpan::new(
            0,
            Duration::from_secs_f32(0.0),
            Duration::from_secs_f32(30.0),
            String::new(),
        )];
        Audio::set_splices(&mut spans, splices);
        assert_eq!(spans[0].splices, control);
    }
}
