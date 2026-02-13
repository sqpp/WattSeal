use chrono::{DateTime, Local};
use common::{MetricType, SensorData};

use crate::{pages::Page, themes::AppTheme, types::TimeRange};

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    NavigateTo(Page),
    ChangeTheme(AppTheme),
    ChangeChartMetricType(String, MetricType),
    ChangeChartTimeRange(String, TimeRange),
    UpdateChartData(Vec<(DateTime<Local>, SensorData)>),
    ReplaceChartData(String, Vec<(DateTime<Local>, SensorData)>),
    FetchChartData(String, TimeRange),
    FetchAllChartsData(TimeRange),
    Redraw,
    LoadChartEvents(i64),
}
