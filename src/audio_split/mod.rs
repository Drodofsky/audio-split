pub mod audio_player;
use std::{num::ParseFloatError, path::PathBuf, time::Duration};

use iced::{Element, Length, Subscription, Task, alignment::Vertical, widget, window::Event};

use crate::audio_split::{
    analyze::detect_silence,
    audio::Audio,
    audio_player::AudioPlayer,
    audio_span::AudioSpan,
    error::Error,
    user_info::{UserInfo, info, warning},
    utils::{open_audio_file, open_audio_file_dialog, open_export_folder_dialog, save_audio_files},
};
mod analyze;
mod audio;
mod audio_span;
mod canvas;
mod debug_id;
pub mod error;
mod user_info;
mod utils;

pub use debug_id::DebugId;

#[derive(Debug)]
pub struct AudioSplit<P: AudioPlayer> {
    audio_player: P,
    audio: Option<Audio>,
    is_playing: bool,
    import_path: Option<PathBuf>,
    export_path: Option<PathBuf>,
    threshold: String,
    duration: String,
    extension: Option<PathBuf>,
    info: UserInfo,
}

impl<P: AudioPlayer> AudioSplit<P> {
    pub fn init(audio_player: P) -> Self {
        Self {
            audio_player,
            audio: None,
            is_playing: false,
            import_path: None,
            export_path: None,
            duration: "0.3".to_string(),
            threshold: "-45.0".to_string(),
            extension: None,
            info: UserInfo::None,
        }
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
            Message::OpenExportDialog => {
                if self.audio.is_some() {
                    Task::perform(
                        open_export_folder_dialog(self.export_path.clone()),
                        Message::ExportPathLoaded,
                    )
                } else {
                    self.set_warning(warning::NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
                    Task::none()
                }
            }
            Message::AudioFilePathLoaded(path) => {
                if let Some(path) = path {
                    self.import_path = Some(path.clone().into());
                    self.extension = PathBuf::from(path.clone()).extension().map(|s| s.into());
                    Task::perform(
                        open_audio_file(path, self.audio_player.get_sink()),
                        Message::AudioLoaded,
                    )
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
                    self.set_warning(warning::NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
                }
                Task::none()
            }
            Message::Play => {
                self.is_playing = true;
                if let Some(audio) = self.audio.as_mut() {
                    audio.set_play();
                } else {
                    self.set_warning(warning::NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
                }
                Task::none()
            }
            Message::Split => {
                if let Some(audio) = self.audio.as_mut() {
                    if audio.split_points_selected() {
                        audio.split();
                    } else {
                        self.set_warning(
                            warning::NO_SPLIT_POINT_SELECTED,
                            DebugId::WarningNoSplitPointSelected,
                        );
                    }
                } else {
                    self.set_warning(warning::NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
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
                    Task::perform(
                        open_audio_file(f, self.audio_player.get_sink()),
                        Message::AudioLoaded,
                    )
                }
                _ => Task::none(),
            },
            Message::Analyze => {
                if let Some(path) = self.import_path.clone() {
                    if let Some(duration) = self
                        .apply_result(self.duration.parse().map_err(|e: ParseFloatError| e.into()))
                        && let Some(threshold) = self.apply_result(
                            self.threshold
                                .parse()
                                .map_err(|e: ParseFloatError| e.into()),
                        )
                        && self.check_duration(duration)
                    {
                        Task::perform(
                            detect_silence(path, threshold, Duration::from_secs_f32(duration)),
                            |a| Message::Analyzed(a),
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    self.set_warning(warning::NO_AUDIO_LOADED, DebugId::WarningNoAudioLoaded);
                    Task::none()
                }
            }
            Message::Analyzed(s) => {
                if let Some(audio) = self.audio.as_mut() {
                    let split_points = s.unwrap();
                    let len = split_points.len();
                    Audio::set_split_points(audio.spans_mut(), split_points);
                    if len == 0 {
                        self.set_warning(
                            warning::NO_SPLIT_POINTS_FOUND,
                            DebugId::WarningNoSplitPointFound,
                        );
                    } else {
                        self.set_info(
                            info::SPLIT_POINTS_DETECTED.replace("{}", &len.to_string()),
                            DebugId::InfoSplitPointsSelected(len),
                        );
                    }
                }
                Task::none()
            }
            Message::ClickSplitPoint(split_point) => {
                if let Some(audio) = self.audio.as_mut() {
                    audio.toggle_selected_split_points(split_point);
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
            widget::container(
                widget::button("open audio file").on_press(Message::OpenAudioFileDialog)
            )
            .id(DebugId::ButtonOpen),
            if self.is_playing {
                widget::container(widget::button("pause").on_press(Message::Pause))
                    .id(DebugId::ButtonPause)
            } else {
                widget::container(widget::button("play").on_press(Message::Play))
                    .id(DebugId::ButtonPlay)
            },
            widget::text("threshold in dB:"),
            widget::text_input("", &self.threshold)
                .on_input(Message::UpdateThreshold)
                .id(DebugId::TextInputThreshold),
            widget::text("duration in sec:"),
            widget::text_input("", &self.duration)
                .on_input(Message::UpdateDuration)
                .id(DebugId::TextInputDuration),
            widget::container(widget::button("analyze").on_press(Message::Analyze))
                .id(DebugId::ButtonAnalyze),
            widget::container(widget::button("split").on_press(Message::Split))
                .id(DebugId::ButtonSplit),
            widget::container(widget::button("export").on_press(Message::OpenExportDialog))
                .id(DebugId::ButtonExport),
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
        let tick = iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick);
        Subscription::batch([
            tick,
            iced::window::events().map(|f| Message::WindowEvent(f.1)),
        ])
    }
    pub fn set_audio(&mut self, audio: Audio) {
        self.audio = Some(audio);
        self.is_playing = true;
        self.set_info(info::AUDIO_LOADED, DebugId::InfoAudioLoaded);
    }
    fn apply_result<T>(&mut self, value: Result<T, Error>) -> Option<T> {
        match value {
            Ok(v) => Some(v),
            Err(e) => {
                self.info = UserInfo::Error(e);
                None
            }
        }
    }
    fn apply_result_and<T>(&mut self, value: Result<T, Error>, mut then: impl FnMut(&mut Self, T)) {
        match value {
            Ok(v) => then(self, v),
            Err(e) => self.info = UserInfo::Error(e),
        }
    }
    fn check_duration(&mut self, duration: f32) -> bool {
        if duration.is_sign_negative() {
            self.apply_result::<()>(Err(Error::new(
                error::ErrorKind::NegativeDuration,
                DebugId::ErrorNegativeDuration,
            )));
            false
        } else {
            true
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
    ClickSplitPoint(Duration),
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
    fn set_split_points_1() {
        let split_points = vec![
            Duration::from_secs_f32(12.5),
            Duration::from_secs_f32(15.5),
            Duration::from_secs_f32(17.5),
            Duration::from_secs_f32(28.5),
        ];
        let control = split_points.clone();
        let mut spans = vec![AudioSpan::new(
            0,
            Duration::from_secs_f32(0.0),
            Duration::from_secs_f32(30.0),
            String::new(),
        )];
        Audio::set_split_points(&mut spans, split_points);
        assert_eq!(spans[0].split_points(), control);
    }
}
