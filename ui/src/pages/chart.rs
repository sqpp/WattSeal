use chrono::Utc;
use iced::{
    Element, Font, Length, Task,
    alignment::Alignment,
    font,
    time::Duration,
    widget::{Column, Text},
};

use crate::{components::chart::SensorChart, message::Message};

const TITLE_FONT_SIZE: u16 = 22;
const SAMPLE_EVERY: Duration = Duration::from_millis(1000);
const FONT_BOLD: Font = Font {
    family: font::Family::Name("Noto Sans"),
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

pub struct ChartPage {
    chart: SensorChart,
}

impl ChartPage {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                chart: SensorChart::new(std::iter::empty()),
            },
            Task::done(Message::Tick),
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let now = Utc::now();
                let percent = rand::random::<f32>() * 100.0;
                let percent2 = rand::random::<f32>() * 100.0;
                self.chart.push_data(now, percent, percent2);
            }
            _ => {
                todo!("Add full message match");
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content = Column::new()
            .spacing(20)
            .align_x(Alignment::Start)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(Text::new("Iced test chart").size(TITLE_FONT_SIZE).font(FONT_BOLD))
            .push(self.chart.view(300.0));

        content.into()
    }
}
