use std::time::Duration;

use iced::{
    Point, Renderer, Size,
    mouse::Cursor,
    widget::canvas::{self, Path},
};

use crate::audio_split::{AudioSpan, Message};

#[derive(Debug, Default, Clone)]
pub struct MouseInteraction {
    hovered: Option<usize>,
}

impl canvas::Program<Message> for AudioSpan {
    type State = MouseInteraction;
    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: iced::Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<Message>> {
        let mut hovered = false;
        if let Some(cursor_position) = cursor.position_in(bounds) {
            if f32::abs(cursor_position.y - (bounds.height / 2.)) > 15. {
                state.hovered = None;
                return None;
            }
            for (index, splice) in self.splices.iter().enumerate() {
                let x_percentage = get_x_percentage(*splice, self.start, self.end);
                if f32::abs(cursor_position.x - (x_percentage * bounds.width)) <= 5.5 {
                    if let iced::Event::Mouse(iced::mouse::Event::ButtonPressed(_)) = event {
                        return Some(canvas::Action::publish(Message::ClickSplice(*splice)));
                    } else {
                        state.hovered = Some(index);
                        hovered = true;
                    }
                }
            }
        }
        if !hovered {
            state.hovered = None;
        }

        None
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry<Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let y_center = bounds.height / 2.0;

        let base_line = Path::rectangle(
            Point::new(0.0, y_center - 5.0),
            Size::new(bounds.width, 10.0),
        );
        frame.fill(&base_line, theme.extended_palette().secondary.weak.color);

        let end_pos = self.end - self.start;
        // I don't want to divide by zero
        if end_pos == Duration::default() {
            return vec![frame.into_geometry()];
        }
        let played_percentage =
            get_x_percentage(Duration::from_secs_f32(self.position), self.start, self.end);

        let played_line = Path::rectangle(
            Point::new(0.0, y_center - 5.0),
            Size::new(bounds.width * played_percentage, 10.0),
        );
        frame.fill(
            &played_line,
            theme.extended_palette().secondary.strong.color,
        );

        for (index, splice) in self.splices.iter().enumerate() {
            let x_pos_percentage = get_x_percentage(*splice, self.start, self.end);
            let splice_line = Path::rectangle(
                Point::new(bounds.width * x_pos_percentage - 5.0, y_center - 10.0),
                Size::new(10.0, 20.0),
            );
            if self.selected_splices.contains(splice) {
                frame.fill(&splice_line, theme.extended_palette().danger.base.color);
            } else {
                frame.fill(&splice_line, theme.extended_palette().primary.base.color);
                if let Some(i) = state.hovered {
                    if i == index {
                        frame.fill(&splice_line, theme.extended_palette().primary.strong.color);
                    }
                }
            }
        }

        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }
}

fn get_x_percentage(value: Duration, start: Duration, end: Duration) -> f32 {
    if end == Duration::default() {
        return 0.0;
    }
    value.saturating_sub(start).as_secs_f32() / end.saturating_sub(start).as_secs_f32()
}
