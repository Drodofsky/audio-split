use iced::{Element, widget};

pub struct AudioSplit {}

impl AudioSplit {
    pub fn init() -> Self {
        Self {}
    }
    pub fn update(&mut self, _message: Message) {}
    pub fn view(&self) -> Element<'_, Message> {
        widget::text("Hello World").into()
    }
}

pub enum Message {}
