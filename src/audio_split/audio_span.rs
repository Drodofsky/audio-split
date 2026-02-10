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
    split_points: Vec<Duration>,
    selected_split_points: Vec<Duration>,
}

impl AudioSpan {
    pub fn new(id: u32, start: Duration, end: Duration, name: String) -> Self {
        Self {
            id,
            start,
            end,
            name,
            position: start.as_secs_f32(),
            split_points: Vec::new(),
            selected_split_points: Vec::new(),
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
    pub fn split_points(&self) -> &[Duration] {
        &self.split_points
    }
    pub fn selected_split_points(&self) -> &[Duration] {
        &self.selected_split_points
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
    pub fn insert_split_point(&mut self, split_point: Duration) -> bool {
        let fits = self.contains(split_point);
        if fits {
            self.split_points.insert(0, split_point);
        }
        fits
    }
    pub fn toggle_split_point_selection(&mut self, split_point: Duration) -> bool {
        let fits = self.contains(split_point);
        if fits {
            if let Some((i, _)) = self
                .selected_split_points
                .iter()
                .enumerate()
                .find(|(_, s)| **s == split_point)
            {
                self.selected_split_points.remove(i);
            } else {
                self.selected_split_points.push(split_point);
            }
        }
        fits
    }
    pub fn clear_split_points(&mut self) {
        self.split_points.clear();
    }
    pub fn split_points_selected(&self) -> bool {
        !self.selected_split_points.is_empty()
    }
}
