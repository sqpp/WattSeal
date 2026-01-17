use iced::{Element, widget::Text};

use crate::{message::Message, themes::AppTheme};

pub struct SettingsPage {}

impl SettingsPage {
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
        Text::new("Settings Page Content").into()
    }
}
