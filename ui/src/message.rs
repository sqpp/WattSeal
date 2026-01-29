use chrono::{DateTime, Utc};
use common::SensorData;

use crate::{
    pages::{Page, dashboard::TimeRange},
    themes::AppTheme,
};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    ChangeChartMetricType(String),
    ChangeChartTimeRange(String, TimeRange),
    UpdateChartData(Vec<(DateTime<Utc>, SensorData)>),
    Redraw,
    LoadChartEvents(i64),
}
