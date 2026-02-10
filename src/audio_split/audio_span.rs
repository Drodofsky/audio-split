use std::time::Duration;

use iced::{Element, Length, widget};

use super::{Message, debug_id::DebugId};

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
                widget::container(
                    widget::button("delete")
                        .style(widget::button::danger)
                        .on_press(Message::DeleteAudioSpan(self.id))
                )
                .id(DebugId::ButtonDelete(self.id))
            ]
            .spacing(5),
        )
        .padding(5)
        .width(self.calc_slider_length())
        .into()
    }
    pub fn start(&self) -> Duration {
        self.start
    }
    pub fn end(&self) -> Duration {
        self.end
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn splices(&self) -> &[Duration] {
        &self.splices
    }
    pub fn selected_splices(&self) -> &[Duration] {
        &self.selected_splices
    }
    pub fn position(&self) -> Duration {
        Duration::from_secs_f32(self.position)
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
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
    pub fn splices_selected(&self) -> bool {
        !self.selected_splices.is_empty()
    }
}
