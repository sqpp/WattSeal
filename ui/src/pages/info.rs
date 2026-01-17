use iced::{Element, widget::Text};

use crate::{message::Message, themes::AppTheme};

pub struct InfoPage {}

impl InfoPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, message: Message) {
        match message {
            _ => {
                todo!()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        Text::new("Info Page Content").into()
    }
}
