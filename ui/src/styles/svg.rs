use iced::{
    Color,
    widget::svg::{self, Catalog, Status, Style},
};

use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
pub enum SvgStyle {
    #[default]
    Default,
    Tinted(Color),
}

impl Catalog for AppTheme {
    type Class<'a> = SvgStyle;

    fn default<'a>() -> Self::Class<'a> {
        SvgStyle::Default
    }

    fn style(&self, class: &Self::Class<'_>, _status: Status) -> Style {
        match class {
            SvgStyle::Default => Style::default(),
            SvgStyle::Tinted(color) => Style { color: Some(*color) },
        }
    }
}
