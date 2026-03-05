use iced::{
    Alignment, Element, Length, Padding,
    widget::{Container, Row, Space, Text, button},
};

use crate::{
    icons::Icon,
    message::Message,
    pages::Page,
    styles::{
        button::ButtonStyle,
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HEADER, FONT_SIZE_HUGE, PADDING_LARGE, PADDING_MEDIUM, SPACING_MEDIUM,
        },
    },
    themes::AppTheme,
    types::AppLanguage,
};

/// Navigation bar with logo placeholder, page title, tabs, and settings button.
pub struct Header {
    nav_pages: Vec<Page>,
    active_page: Page,
}

impl Header {
    /// Creates a header with the given pages and initial active page.
    pub fn new(nav_pages: Vec<Page>, active_page: Page) -> Self {
        Self { nav_pages, active_page }
    }

    /// Sets the active navigation tab.
    pub fn change_page(&mut self, new_page: Page) {
        self.active_page = new_page;
    }

    /// Renders the header bar.
    pub fn view(&self, language: AppLanguage) -> Element<'_, Message, AppTheme> {
        let logo = Icon::SealGraph
            .to_text()
            .size(FONT_SIZE_HUGE)
            .align_y(Alignment::Center);

        let title = Container::new(
            Text::new(self.active_page.translated_name(language))
                .size(FONT_SIZE_HEADER)
                .font(FONT_BOLD),
        )
        .height(Length::Fixed(FONT_SIZE_HUGE))
        .align_y(Alignment::Center);

        let left_section = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(logo)
            .push(title);

        // Use gear icon instead of text for settings
        let settings_button = button(Icon::Settings.to_text().size(FONT_SIZE_BODY))
            .padding(Padding::from([8.0, 16.0]))
            .class(ButtonStyle::Standard)
            .on_press(Message::OpenSettings);

        let nav_buttons = self.nav_pages.iter().fold(Row::new().spacing(8), |row, page| {
            let is_active = self.active_page == *page;
            let button_style = if is_active {
                ButtonStyle::NavActive
            } else {
                ButtonStyle::Nav
            };

            row.push(
                button(Text::new(page.translated_name(language)).size(FONT_SIZE_BODY))
                    .padding(Padding::from([8.0, 16.0]))
                    .class(button_style)
                    .on_press(Message::NavigateTo(*page)),
            )
        });

        let content = Row::new()
            .padding(Padding::from([PADDING_MEDIUM, PADDING_LARGE]))
            .spacing(20)
            .align_y(Alignment::Center)
            .push(left_section)
            .push(Space::new().width(Length::Fill))
            .push(nav_buttons)
            .push(settings_button);

        Container::new(content)
            .width(Length::Fill)
            .class(ContainerStyle::Header)
            .into()
    }
}
