use iced::{Color, widget::Text};

use crate::{
    styles::{style_constants::ICONS, text::TextStyle},
    themes::AppTheme,
};

/// Icon font glyphs.
/// Each variant maps to a codepoint in the embedded `wattseal-icons.ttf` font.
#[derive(Clone, Copy)]
pub enum Icon {
    Settings,
    Windows,
    Android,
    Apple,
    MacOS,
    Battery,
    CPU,
    Display,
    GitHub,
    GPU,
    RAM,
    Storage,
    System,
    Linux,
    Seal,
    SealGraph,
}

impl Icon {
    /// Returns the Unicode codepoint assigned to this icon in the font.
    pub fn codepoint(self) -> char {
        match self {
            Icon::Settings => 'A',
            Icon::Windows => 'B',
            Icon::Android => 'C',
            Icon::Apple => 'D',
            Icon::MacOS => 'E',
            Icon::Battery => 'F',
            Icon::CPU => 'G',
            Icon::Display => 'H',
            Icon::GitHub => 'I',
            Icon::GPU => 'J',
            Icon::RAM => 'K',
            Icon::Storage => 'L',
            Icon::System => 'M',
            Icon::Linux => 'N',
            Icon::Seal => 'O',
            Icon::SealGraph => 'P',
        }
    }

    /// Renders this icon as an iced `Text` widget using the icon font.
    pub fn to_text(self) -> Text<'static, AppTheme> {
        Text::new(self.codepoint().to_string()).font(ICONS)
    }

    /// Renders this icon as a colored `Text` widget.
    pub fn to_text_colored(self, color: Color) -> Text<'static, AppTheme> {
        self.to_text().class(TextStyle::Colored(color))
    }
}
