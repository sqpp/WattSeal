use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::Database;
use iced::{
    Element, Subscription, Task, Theme,
    time::{Duration, every},
    widget::{Column, pick_list},
};

use crate::{
    components::{chart::ChartData, header::Header},
    message::Message,
    pages::{Page, chart::ChartPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    themes::AppTheme,
};

const FPS: u64 = 1;

pub struct App {
    current_page: Page,
    chart_page: ChartPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    header: Header,
    theme: AppTheme,
    database: Database,
    last_timestamp: Option<DateTime<Utc>>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::Dracula;
        let (chart_page, task) = ChartPage::new(theme);
        let current_page = Page::Chart;
        let database = Database::new().unwrap();

        (
            Self {
                current_page,
                chart_page,
                header: Header::new(&current_page.to_string(), Page::all()),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                theme,
                database,
                last_timestamp: None,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let chart_data = self.load_latest_chart_data();
                self.chart_page.update(Message::UpdateChartData(chart_data));
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.set_title(&page.to_string());
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.chart_page.update_theme(theme);
            }
            _ => {}
        }
    }

    fn load_latest_chart_data(&mut self) -> ChartData {
        let mut chart_data: ChartData = HashMap::new();
        let now = Utc::now();
        let res = self.database.select_last_n_events(1).unwrap();
        println!("{:?}", res);

        chart_data.insert("CPU Usage".into(), (now, rand::random::<f32>() * 100.0));
        chart_data.insert("CPU Power".into(), (now, rand::random::<f32>() * 65.0));
        chart_data.insert("GPU Usage".into(), (now, rand::random::<f32>() * 100.0));
        chart_data.insert("GPU Power".into(), (now, rand::random::<f32>() * 75.0));

        chart_data
    }

    pub fn view(&self) -> Element<'_, Message> {
        let page_content = match self.current_page {
            Page::Chart => self.chart_page.view(),
            Page::Info => self.info_page.view(),
            Page::Optimization => self.optimization_page.view(),
            Page::Settings => self.settings_page.view(),
        };

        Column::new()
            .push(self.header.view())
            .push(page_content)
            .push(pick_list(AppTheme::all(), Some(self.theme), Message::ChangeTheme))
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    pub fn theme(&self) -> Theme {
        self.theme.to_iced_theme()
    }
}
