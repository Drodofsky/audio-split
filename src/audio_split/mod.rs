use std::{path::PathBuf, time::Duration};

use iced::{Element, Length, Subscription, Task, alignment::Vertical, widget, window::Event};

use crate::audio_split::{
    analyze::detect_silence,
    audio::Audio,
    audio_span::AudioSpan,
    error::Error,
    user_info::{UserInfo, info::AUDIO_LOADED, warning::NO_AUDIO_LOADED},
    utils::{open_audio_file, open_audio_file_dialog, open_export_folder_dialog, save_audio_files},
};
mod analyze;
mod audio;
mod audio_span;
mod canvas;
mod debug_id;
mod error;
mod user_info;
mod utils;

pub use debug_id::DebugId;

#[derive(Debug, Default)]
pub struct AudioSplit {
    audio: Option<Audio>,
    is_playing: bool,
    import_path: Option<PathBuf>,
    export_path: Option<PathBuf>,
    threshold: String,
    duration: String,
    extension: Option<PathBuf>,
    info: UserInfo,
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
                            audio.spans().to_vec(),
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
                self.apply_result_and(audio, |this, audio| this.set_audio(audio));
                Task::none()
            }
            Message::AudioSpanPositionUpdate(id, pos) => {
                self.audio.as_mut().unwrap().set_pos(id, pos);
                Task::none()
            }
            Message::Tick => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.update_position_info();
                    if let Some(last) = audio.spans().last()
                        && let Some(sub) = (last.end().checked_sub(audio.get_pos()))
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
                } else {
                    self.set_warning(NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
                }
                Task::none()
            }
            Message::Play => {
                self.is_playing = true;
                if let Some(audio) = self.audio.as_mut() {
                    audio.set_play();
                } else {
                    self.set_warning(NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
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
                    Audio::set_splices(audio.spans_mut(), s.unwrap());
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
        widget::column![self.view_top(), self.view_center(), self.view_info()].into()
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
    fn view_info(&self) -> Element<'_, Message> {
        match &self.info {
            UserInfo::None => widget::container(widget::space()),
            UserInfo::Info(text, id) => {
                widget::container(widget::text(text).style(widget::text::base)).id(*id)
            }
            UserInfo::Waring(text, id) => {
                widget::container(widget::text(text).style(widget::text::warning)).id(*id)
            }
            UserInfo::Error(e) => {
                widget::container(widget::text(format!("{e}")).style(widget::text::danger))
                    .id(e.id())
            }
        }
        .padding(5)
        .into()
    }
    pub fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick);
        Subscription::batch([
            tick,
            iced::window::events().map(|f| Message::WindowEvent(f.1)),
        ])
    }
    pub fn set_audio(&mut self, audio: Audio) {
        self.audio = Some(audio);
        self.is_playing = true;
        self.set_info(AUDIO_LOADED, DebugId::InfoAudioLoaded);
    }
    fn apply_result_and<T>(&mut self, value: Result<T, Error>, mut then: impl FnMut(&mut Self, T)) {
        match value {
            Ok(v) => then(self, v),
            Err(e) => self.info = UserInfo::Error(e),
        }
    }
    fn set_warning(&mut self, warning: impl Into<String>, id: DebugId) {
        self.info = UserInfo::Waring(warning.into(), id)
    }
    fn set_info(&mut self, info: impl Into<String>, id: DebugId) {
        self.info = UserInfo::Info(info.into(), id)
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
        assert_eq!(spans[0].splices(), control);
    }
}
