use iced::font::Font;

pub mod app;
pub mod components;
pub mod icons;
pub mod message;
pub mod pages;
pub mod styles;
pub mod themes;
pub mod translations;
pub mod types;

use std::borrow::Cow;

use app::App;
use styles::style_constants::{FONT_MEDIUM, FONT_SIZE_BODY, ICONS_BYTES};

/// Launches the WattSeal GUI application.
pub fn run() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(App::title)
        .settings(iced::Settings {
            id: Some(String::from(env!("CARGO_PKG_NAME"))),
            fonts: vec![Cow::Borrowed(ICONS_BYTES)],
            default_font: FONT_MEDIUM,
            default_text_size: FONT_SIZE_BODY.into(),
            antialiasing: true,
            vsync: true,
        })
        .antialiasing(true)
        .default_font(Font::with_name("Roboto"))
        .subscription(App::subscription)
        .theme(App::theme)
        .exit_on_close_request(false)
        .run()
}
