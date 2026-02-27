use common::HardwareInfo;
use iced::{
    Color, Element, Length, Padding,
    widget::{Column, Container, Row, Scrollable, Space},
};

use crate::{
    components::hardware_card::{InfoCard, InfoField, hardware_card},
    icons,
    message::Message,
    styles::{
        scrollable::ScrollableStyle,
        style_constants::{PADDING_LARGE, SPACING_LARGE},
    },
    themes::AppTheme,
};

const CARD_HEIGHT: f32 = 180.0;

pub struct InfoPage;

impl InfoPage {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self, hw: &HardwareInfo, theme: AppTheme) -> Element<'_, Message, AppTheme> {
        let pal = theme.palette();

        let mut specs: Vec<InfoCard> = Vec::new();

        specs.push(InfoCard::new(
            icons::CPU,
            pal.primary,
            "CPU".to_string(),
            "Processor Information".to_string(),
            InfoField::new("Model", &hw.cpu.name),
            Some(InfoField::new(
                "Cores",
                format!("{} cores / {} threads", hw.cpu.physical_cores, hw.cpu.logical_cores),
            )),
        ));

        if hw.gpus.is_empty() {
            specs.push(InfoCard::new(
                icons::GPU,
                pal.danger,
                "GPU".to_string(),
                "Graphics Information".to_string(),
                InfoField::new("Model", "N/A"),
                None,
            ));
        } else {
            for (i, gpu) in hw.gpus.iter().enumerate() {
                let subtitle = format!("Graphics Processor {}", i + 1);
                specs.push(InfoCard::new(
                    icons::GPU,
                    pal.danger,
                    "GPU".to_string(),
                    subtitle,
                    InfoField::new("Model", gpu.as_str()),
                    None,
                ));
            }
        }

        specs.push(InfoCard::new(
            icons::RAM,
            pal.warning,
            "Memory".to_string(),
            "RAM Information".to_string(),
            InfoField::new("Total Memory", format_bytes_gb(hw.memory.total_ram_bytes)),
            Some(InfoField::new("Swap", format_bytes_gb(hw.memory.total_swap_bytes))),
        ));

        specs.push(InfoCard::new(
            icons::SYSTEM,
            pal.success,
            "System".to_string(),
            "OS Information".to_string(),
            InfoField::new("Operating System", &hw.system.os),
            Some(InfoField::new("Hostname", &hw.system.hostname)),
        ));

        if hw.disks.is_empty() {
            specs.push(InfoCard::new(
                icons::STORAGE,
                pal.primary,
                "Storage".to_string(),
                "Disk Information".to_string(),
                InfoField::new("Disk", "N/A"),
                Some(InfoField::new("Space", "N/A")),
            ));
        } else {
            for (i, disk) in hw.disks.iter().enumerate() {
                let subtitle = format!("Disk {}", i + 1);
                specs.push(InfoCard::new(
                    icons::STORAGE,
                    pal.primary,
                    "Storage".to_string(),
                    subtitle,
                    InfoField::new("Disk", format!("{} ({})", disk.mount_point, disk.name)),
                    Some(InfoField::new(
                        "Space",
                        format!(
                            "{} / {}",
                            format_bytes_gb(disk.used_bytes),
                            format_bytes_gb(disk.total_bytes)
                        ),
                    )),
                ));
            }
        }

        if hw.battery.present {
            let capacity = match (hw.battery.design_capacity_wh, hw.battery.cycle_count) {
                (Some(cap), Some(cycles)) => format!("{:.1} Wh ({} cycles)", cap, cycles),
                (Some(cap), None) => format!("{:.1} Wh", cap),
                (None, Some(cycles)) => format!("N/A ({} cycles)", cycles),
                (None, None) => "N/A".to_string(),
            };

            specs.push(InfoCard::new(
                icons::BATTERY,
                pal.warning,
                "Battery".to_string(),
                "Battery Status".to_string(),
                InfoField::new("Name", hw.battery.name.as_deref().unwrap_or("N/A")),
                Some(InfoField::new("Capacity", capacity)),
            ));
        } else {
            specs.push(InfoCard::new(
                icons::BATTERY,
                pal.warning,
                "Battery".to_string(),
                "Battery Status".to_string(),
                InfoField::new("Name", "N/A"),
                Some(InfoField::new("Capacity", "N/A")),
            ));
        }

        if hw.displays.is_empty() {
            specs.push(InfoCard::new(
                icons::DISPLAY,
                pal.success,
                "Display".to_string(),
                "Screen Information".to_string(),
                InfoField::new("Model", "N/A"),
                Some(InfoField::new("Mode", "N/A")),
            ));
        } else {
            let mut displays = hw.displays.iter().collect::<Vec<_>>();
            displays.sort_by_key(|d| !d.is_primary);

            for d in displays {
                let subtitle = if d.is_primary {
                    "Primary Display"
                } else {
                    "Secondary Display"
                };
                specs.push(InfoCard::new(
                    icons::DISPLAY,
                    pal.success,
                    "Display".to_string(),
                    subtitle.to_string(),
                    InfoField::new("Model", &d.model),
                    Some(InfoField::new(
                        "Mode",
                        format_display_mode(&d.resolution, d.refresh_rate_hz),
                    )),
                ));
            }
        }

        let cards = specs
            .into_iter()
            .map(|card| {
                hardware_card(
                    card.icon_svg,
                    card.accent,
                    &card.title,
                    &card.subtitle,
                    card.field,
                    card.optional_field,
                )
            })
            .collect::<Vec<_>>();

        let mut card_rows = Column::new().spacing(SPACING_LARGE);

        let mut row = Row::new().spacing(SPACING_LARGE);

        for (i, card) in cards.into_iter().enumerate() {
            if i % 3 == 0 && i != 0 {
                card_rows = card_rows.push(row);
                row = Row::new().spacing(SPACING_LARGE);
            }

            row = row.push(
                Container::new(card)
                    .width(Length::FillPortion(1))
                    .height(Length::Fixed(CARD_HEIGHT)),
            );
        }

        Scrollable::new(
            Container::new(card_rows)
                .width(Length::Fill)
                .padding(Padding::from(PADDING_LARGE)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .class(ScrollableStyle::Standard)
        .into()
    }
}

fn format_bytes_gb(bytes: u64) -> String {
    if bytes == 0 {
        return "N/A".to_string();
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    format!("{:.2} GB", gb)
}

fn format_display_mode(resolution: &str, refresh_rate_hz: u32) -> String {
    if refresh_rate_hz > 0 {
        format!("{} @ {} Hz", resolution, refresh_rate_hz)
    } else {
        resolution.to_string()
    }
}
