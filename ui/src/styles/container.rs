use iced::{
    Background, Border, Color, Shadow, Vector,
    widget::container::{self, Catalog},
};

use super::{
    colors::{ExtendedPalette, with_alpha},
    style_constants::{BORDER_RADIUS_LARGE, BORDER_RADIUS_MEDIUM, BORDER_WIDTH},
};
use crate::themes::AppTheme;

#[derive(Debug, Clone, Copy, Default)]
/// Container appearance variants.
pub enum ContainerStyle {
    #[default]
    Transparent,
    Card,
    Header,
    Footer,
    PowerCard,
    ComponentCard,
    IconBadge(Color),
    ModalBackdrop,
    ModalCard,
}

impl Catalog for AppTheme {
    type Class<'a> = ContainerStyle;

    fn default<'a>() -> Self::Class<'a> {
        Self::Class::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let ext = ExtendedPalette::from_theme(self);

        match class {
            ContainerStyle::Transparent => container::Style::default(),

            ContainerStyle::Card => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border {
                    color: ext.border_subtle,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow::default(),
                ..Default::default()
            },

            ContainerStyle::Header => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border::default(),
                text_color: Some(ext.text),
                shadow: Shadow {
                    color: with_alpha(Color::BLACK, 0.15),
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                ..Default::default()
            },

            ContainerStyle::Footer => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border {
                    color: ext.border_subtle,
                    width: BORDER_WIDTH,
                    ..Border::default()
                },
                text_color: Some(ext.text_muted),
                shadow: Shadow::default(),
                ..Default::default()
            },

            ContainerStyle::PowerCard => container::Style {
                background: Some(Background::Color(ext.elevated_background)),
                border: Border {
                    color: with_alpha(ext.primary, 0.4),
                    width: BORDER_WIDTH * 1.5,
                    radius: BORDER_RADIUS_LARGE.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow {
                    color: with_alpha(ext.primary, 0.2),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            },

            ContainerStyle::ComponentCard => container::Style {
                background: Some(Background::Color(ext.card_background)),
                border: Border {
                    color: ext.border_subtle,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow::default(),
                ..Default::default()
            },

            ContainerStyle::IconBadge(accent) => container::Style {
                background: Some(Background::Color(with_alpha(*accent, 0.15))),
                border: Border {
                    color: with_alpha(*accent, 0.3),
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_MEDIUM.into(),
                },
                ..Default::default()
            },

            ContainerStyle::ModalBackdrop => container::Style {
                background: Some(Background::Color(with_alpha(Color::BLACK, 0.35))),
                ..Default::default()
            },

            ContainerStyle::ModalCard => container::Style {
                background: Some(Background::Color(ext.elevated_background)),
                border: Border {
                    color: ext.border,
                    width: BORDER_WIDTH,
                    radius: BORDER_RADIUS_LARGE.into(),
                },
                text_color: Some(ext.text),
                shadow: Shadow {
                    color: with_alpha(Color::BLACK, 0.25),
                    offset: Vector::new(0.0, 8.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            },
        }
    }
}
