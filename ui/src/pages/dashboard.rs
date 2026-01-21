use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::{CPUData, DatabaseEntry, SensorData, TotalData};
use iced::{
    Alignment, Color, Element, Length, Padding, Task,
    alignment::{Horizontal, Vertical},
    time::Duration,
    widget::{Column, Container, Row, Text},
};

use crate::{
    components::chart::{AxisType, LineType, SensorChart},
    message::Message,
    styles::{
        container::ContainerStyle,
        style_constants::{
            FONT_BOLD, FONT_SIZE_BODY, FONT_SIZE_HUGE, FONT_SIZE_SUBTITLE, FONT_SIZE_TITLE, PADDING_LARGE,
            SPACING_LARGE, SPACING_MEDIUM, SPACING_XLARGE,
        },
        text::TextStyle,
    },
    themes::AppTheme,
};

const SAMPLE_EVERY: Duration = Duration::from_millis(1000);

#[derive(Debug, Clone, Default)]
pub struct PowerSnapshot {
    pub components_power: HashMap<String, Option<SensorData>>,
}

impl PowerSnapshot {
    pub fn from_components_list(components: &Vec<String>) -> Self {
        let mut readings = PowerSnapshot::default();

        for component in components.iter() {
            let sensor_data =
                SensorData::get_matching_sensor_data(component).unwrap_or(SensorData::Total(TotalData::default()));
            readings
                .components_power
                .insert(sensor_data.sensor_type().to_string(), Some(sensor_data));
        }
        readings
    }

    pub fn update_from_sensor_data(&mut self, data: &[(DateTime<Utc>, SensorData)]) {
        for (_, sensor) in data.iter() {
            self.components_power
                .insert(sensor.sensor_type().to_string(), Some(sensor.clone()));
        }
    }

    pub fn total_power(&self) -> f64 {
        match self.components_power.get(TotalData::generic_name()) {
            Some(Some(SensorData::Total(p))) => p.total_power_watts,
            _ => 0.0,
        }
    }

    pub fn power_details(&self) -> Vec<(&String, &Option<SensorData>)> {
        let mut details: Vec<(&String, &Option<SensorData>)> = self
            .components_power
            .iter()
            .filter(|(_, data)| {
                if let Some(SensorData::Total(_)) = data {
                    false
                } else {
                    true
                }
            })
            .collect();
        details.sort_by(|a, b| a.0.cmp(b.0));
        details
    }
}

pub struct DashboardPage {
    chart: SensorChart,
    current_readings: PowerSnapshot,
}

impl DashboardPage {
    pub fn new(theme: AppTheme, components: Vec<String>) -> (Self, Task<Message>) {
        let series = vec![
            (
                "CPU Power".to_string(),
                LineType::Line,
                AxisType::Primary("CPU Power".to_string(), "W".to_string()),
            ),
            (
                "GPU Power".to_string(),
                LineType::Line,
                AxisType::Primary("GPU Power".to_string(), "W".to_string()),
            ),
            (
                "CPU Usage".to_string(),
                LineType::Dashed,
                AxisType::Secondary("CPU Usage".to_string(), "%".to_string()),
            ),
            (
                "GPU Usage".to_string(),
                LineType::Dashed,
                AxisType::Secondary("GPU Usage".to_string(), "%".to_string()),
            ),
        ];
        let x_axis = AxisType::Primary("Time".to_string(), "s".to_string());
        let y_axes = (
            AxisType::Primary("Power".to_string(), "W".to_string()),
            AxisType::Secondary("Usage".to_string(), "%".to_string()),
        );
        let chart = SensorChart::new(series, None, None, theme, x_axis, y_axes);
        (
            Self {
                chart,
                current_readings: PowerSnapshot::from_components_list(&components),
            },
            Task::none(),
        )
    }

    pub fn update_theme(&mut self, theme: AppTheme) {
        self.chart.update_style(theme);
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::UpdateChartData(data) => {
                self.current_readings.update_from_sensor_data(&data);
                self.chart.push_data(data);
            }
            _ => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let content = Column::new()
            .spacing(SPACING_XLARGE)
            .padding(Padding::from(PADDING_LARGE))
            .width(Length::Fill)
            .height(Length::Fill)
            .push(self.view_power_summary())
            .push(self.view_component_cards())
            .push(self.view_chart_section());

        Container::new(content).width(Length::Fill).height(Length::Fill).into()
    }

    fn view_power_summary(&self) -> Element<'_, Message, AppTheme> {
        let power_value = format!("{:.1}", self.current_readings.total_power());
        let power_unit = "W";

        let title = Text::new("Total Power Consumption")
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .class(TextStyle::Subtitle);

        let power_display = Row::new()
            .align_y(Alignment::End)
            .spacing(4)
            .push(
                Text::new(power_value)
                    .size(FONT_SIZE_HUGE)
                    .font(FONT_BOLD)
                    .class(TextStyle::Primary),
            )
            .push(Text::new(power_unit).size(FONT_SIZE_TITLE).class(TextStyle::Muted));

        let content = Column::new()
            .spacing(SPACING_MEDIUM)
            .align_x(Alignment::Center)
            .push(title)
            .push(power_display);

        Container::new(content)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .class(ContainerStyle::PowerCard)
            .into()
    }

    fn view_component_cards(&self) -> Element<'_, Message, AppTheme> {
        let mut column = Column::new().spacing(SPACING_LARGE).width(Length::Fill);
        let mut row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
        let mut items_in_row = 0;

        for (i, sensor) in self.current_readings.power_details().iter().enumerate() {
            let power = sensor.1.as_ref().and_then(|data| data.total_power_watts());
            let usage = sensor.1.as_ref().and_then(|data| data.usage_percent());
            let card = self.component_snapshot_card(sensor.0, power, usage);

            row = row.push(card);
            items_in_row += 1;

            if i % 2 == 1 {
                column = column.push(row);
                row = Row::new().spacing(SPACING_LARGE).width(Length::Fill);
                items_in_row = 0;
            }
        }

        if items_in_row > 0 {
            column = column.push(row);
        }

        Container::new(column)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::Card)
            .into()
    }

    fn component_snapshot_card(
        &self,
        name: &str,
        power: Option<f64>,
        usage: Option<f64>,
    ) -> Element<'_, Message, AppTheme> {
        let name_owned = name.to_string();
        let power_text = power
            .map(|p| format!("{:.1} W", p))
            .unwrap_or_else(|| "N/A".to_string());

        let usage_text = usage.map(|u| format!("{:.1}%", u)).unwrap_or_else(|| "N/A".to_string());

        let title = Text::new(name_owned).size(FONT_SIZE_SUBTITLE).font(FONT_BOLD);

        let power_style = if power.is_some() {
            TextStyle::Primary
        } else {
            TextStyle::Muted
        };

        let usage_style = if usage.is_some() {
            TextStyle::Success
        } else {
            TextStyle::Muted
        };

        let power_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(Text::new("Power:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
            .push(
                Text::new(power_text)
                    .size(FONT_SIZE_BODY)
                    .font(FONT_BOLD)
                    .class(power_style),
            );

        let usage_row = Row::new()
            .spacing(SPACING_MEDIUM)
            .align_y(Alignment::Center)
            .push(Text::new("Usage:").size(FONT_SIZE_BODY).class(TextStyle::Muted))
            .push(
                Text::new(usage_text)
                    .size(FONT_SIZE_BODY)
                    .font(FONT_BOLD)
                    .class(usage_style),
            );

        let content = Column::new()
            .spacing(SPACING_LARGE)
            .push(title)
            .push(power_row)
            .push(usage_row);

        Container::new(content)
            .width(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::ComponentCard)
            .into()
    }

    fn view_chart_section(&self) -> Element<'_, Message, AppTheme> {
        let title = Text::new("Power History")
            .size(FONT_SIZE_SUBTITLE)
            .font(FONT_BOLD)
            .class(TextStyle::Subtitle);

        let chart_container = Container::new(self.chart.view(300.0))
            .width(Length::Fill)
            .height(Length::Fill);

        let content = Column::new()
            .spacing(SPACING_MEDIUM)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(title)
            .push(chart_container);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from(PADDING_LARGE))
            .class(ContainerStyle::Card)
            .into()
    }
}
