use iced::{Element, Length, widget};

pub struct AudioSplit {}

impl AudioSplit {
    pub fn init() -> Self {
        Self {}
    }
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'_, Message> {
        widget::column![self.view_top(), self.view_center()].into()
    }
    fn view_top(&self) -> Element<'_, Message> {
        widget::row![widget::button("open audio file")].into()
    }
    fn view_center(&self) -> Element<'_, Message> {
        widget::container(widget::text("Hello World"))
            .center(Length::Fill)
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum Message {}
