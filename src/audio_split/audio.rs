use std::{fmt, sync::Arc, time::Duration};

use iced::{Element, Length, widget};

use super::{Message, audio_span::AudioSpan};

#[derive(Clone)]
pub struct Audio {
    sink: Arc<rodio::Sink>,
    _stream_handle: Arc<rodio::OutputStream>,
    spans: Vec<AudioSpan>,
    file_name: String,
    index_counter: u32,
}

impl Audio {
    pub fn new(
        sink: rodio::Sink,
        stream_handle: rodio::OutputStream,
        span: AudioSpan,
        file_name: String,
    ) -> Self {
        Self {
            sink: Arc::new(sink),
            _stream_handle: Arc::new(stream_handle),
            spans: vec![span],
            file_name,
            index_counter: 0,
        }
    }
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
    pub fn spans(&self) -> &[AudioSpan] {
        &self.spans
    }
    pub fn spans_mut(&mut self) -> &mut [AudioSpan] {
        &mut self.spans
    }
    pub fn get_pos(&self) -> Duration {
        self.sink.get_pos()
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
            .map(|s| s.selected_splices().iter())
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
        let start = span.start();
        let end = span.end();
        let name = span.name().to_string();
        let id = span.id();
        let splices = span.splices().to_vec();
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
            span.set_name(text);
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

impl fmt::Debug for Audio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Audio")
    }
}
