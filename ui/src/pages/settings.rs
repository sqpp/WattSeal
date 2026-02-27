use iced::{
    Alignment, Element, Length,
    widget::{Button, Column, Container, Row, Text, button, pick_list},
};

use crate::{
    message::Message,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_SUBTITLE, PADDING_LARGE, PADDING_MEDIUM,
            PADDING_XLARGE, SPACING_LARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
    types::AppLanguage,
};

pub struct SettingsPage {}

impl SettingsPage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self, theme: AppTheme, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let title = Text::new("Settings")
            .size(FONT_SIZE_HEADER)
            .font(FONT_BOLD)
            .width(Length::Fill);

        let subtitle = Text::new("General").size(FONT_SIZE_SUBTITLE).class(TextStyle::Muted);

        let theme_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(Text::new("Theme").size(FONT_SIZE_BODY).width(Length::FillPortion(2)))
            .push(
                pick_list(AppTheme::all(), Some(theme), Message::ChangeTheme)
                    .width(Length::FillPortion(3))
                    .padding(PADDING_MEDIUM),
            );

        let language_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(Text::new("Language").size(FONT_SIZE_BODY).width(Length::FillPortion(2)))
            .push(
                pick_list(AppLanguage::all(), Some(language), Message::ChangeLanguage)
                    .width(Length::FillPortion(3))
                    .padding(PADDING_MEDIUM),
            );

        let close_button: Button<'_, Message, AppTheme> = button(Text::new("Close").size(FONT_SIZE_BODY))
            .class(ButtonStyle::Standard)
            .on_press(Message::CloseSettings);

        let top_row = Row::new()
            .spacing(SPACING_LARGE)
            .align_y(Alignment::Center)
            .push(title)
            .push(close_button);

        let content = Column::new()
            .spacing(SPACING_LARGE)
            .align_x(Alignment::Start)
            .push(top_row)
            .push(subtitle)
            .push(theme_row)
            .push(language_row);

        Container::new(content)
            .width(Length::Fixed(520.0))
            .padding(PADDING_XLARGE)
            .class(ContainerStyle::ModalCard)
            .into()
    }
}
