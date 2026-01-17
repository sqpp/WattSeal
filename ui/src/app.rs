use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::{CPUData, Database, DatabaseEntry, GPUData, SensorData};
use iced::{
    Element, Subscription, Task,
    time::{Duration, every},
    widget::{Column, Container, pick_list},
};

use crate::{
    components::header::Header,
    message::Message,
    pages::{Page, dashboard::DashboardPage, info::InfoPage, optimization::OptimizationPage, settings::SettingsPage},
    styles::container::ContainerStyle,
    themes::AppTheme,
};

const FPS: u64 = 1;

pub struct App {
    current_page: Page,
    dashboard_page: DashboardPage,
    info_page: InfoPage,
    optimization_page: OptimizationPage,
    settings_page: SettingsPage,
    header: Header,
    theme: AppTheme,
    database: Database,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let theme = AppTheme::Dracula;
        let current_page = Page::Dashboard;
        let database = Database::new().unwrap();
        let (dashboard_page, task) = DashboardPage::new(theme);

        (
            Self {
                current_page,
                dashboard_page,
                header: Header::new(Page::all(), current_page),
                info_page: InfoPage::new(),
                optimization_page: OptimizationPage::new(),
                settings_page: SettingsPage::new(),
                theme,
                database,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::LoadChartEvents(number) => {
                let chart_data = self.load_latest_chart_data(number);
                self.dashboard_page.update(Message::UpdateChartData(chart_data));
            }
            Message::Tick => {
                let chart_data = self.load_latest_chart_data(1);
                self.dashboard_page.update(Message::UpdateChartData(chart_data));
            }
            Message::NavigateTo(page) => {
                self.current_page = page;
                self.header.change_page(page);
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.dashboard_page.update_theme(theme);
            }
            _ => {}
        }
    }

    fn load_latest_chart_data(&mut self, number: i64) -> Vec<(DateTime<Utc>, SensorData)> {
        self.database
            .select_last_n_records(number)
            .unwrap_or_default()
            .into_iter()
            .map(|(ts, data)| (ts.into(), data))
            .collect()
    }

    pub fn view(&self) -> Element<'_, Message, AppTheme> {
        let page_content = match self.current_page {
            Page::Dashboard => self.dashboard_page.view(),
            Page::Info => self.info_page.view(),
            Page::Optimization => self.optimization_page.view(),
            Page::Settings => self.settings_page.view(),
        };

        let theme_picker =
            Container::new(pick_list(AppTheme::all(), Some(self.theme), Message::ChangeTheme)).padding(10);

        Column::new()
            .push(self.header.view())
            .push(page_content)
            .push(theme_picker)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        every(Duration::from_millis(1000 / FPS)).map(|_| Message::Tick)
    }

    pub fn theme(&self) -> AppTheme {
        self.theme
    }
}
