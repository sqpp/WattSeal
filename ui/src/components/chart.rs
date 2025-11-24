use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use iced::{
    Element, Length, Size,
    alignment::Alignment,
    time::Duration,
    widget::{
        Column, Text,
        canvas::{Cache, Frame, Geometry},
    },
};
use plotters::{
    coord::Shift,
    prelude::ChartBuilder,
    style::{Color, RGBColor},
};
use plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartWidget, DrawingArea, Renderer, plotters_backend};

use crate::message::Message;

const PLOT_SECONDS: usize = 60;

pub struct SensorChart<const N: usize> {
    cache: Cache,
    data_series: Vec<TimeSeries>,
    limit: Duration,
    color: RGBColor,
}

struct TimeSeries {
    label: String,
    data: VecDeque<(DateTime<Utc>, f32)>,
    color: RGBColor,
}

impl TimeSeries {
    pub fn push_front(&mut self, value: (DateTime<Utc>, f32)) {
        self.data.push_front(value);
    }

    pub fn pop_back(&mut self) {
        self.data.pop_back();
    }
}

impl<const N: usize> SensorChart<N> {
    pub fn new(series: [(String, RGBColor); N], color: RGBColor) -> Self {
        let data: Vec<TimeSeries> = series
            .into_iter()
            .map(|(label, color)| TimeSeries {
                label,
                data: VecDeque::new(),
                color,
            })
            .collect();

        Self {
            cache: Cache::new(),
            data_series: data,
            limit: Duration::from_secs(PLOT_SECONDS as u64),
            color,
        }
    }

    pub fn push_data(&mut self, time: DateTime<Utc>, series: [Option<f32>; N]) {
        let cur_ms = time.timestamp_millis();
        for (ts, value) in self.data_series.iter_mut().zip(series) {
            let value = match value {
                Some(v) => v,
                None => continue,
            };
            ts.push_front((time, value));
            while let Some(&(old_time, _)) = ts.data.back() {
                if cur_ms - old_time.timestamp_millis() > self.limit.as_millis() as i64 {
                    ts.pop_back();
                } else {
                    break;
                }
            }
        }
        self.cache.clear();
    }

    pub fn view(&self, chart_height: f32) -> Element<'_, Message> {
        Column::new()
            .width(Length::Fill)
            .height(Length::Shrink)
            .spacing(5)
            .align_x(Alignment::Center)
            .push(Text::new("Sensor Chart"))
            .push(ChartWidget::new(self).height(Length::Fixed(chart_height)))
            .into()
    }
}

impl<const N: usize> Chart<Message> for SensorChart<N> {
    type State = ();

    #[inline]
    fn draw<R: Renderer, F: Fn(&mut Frame)>(&self, renderer: &R, bounds: Size, draw_fn: F) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, chart: ChartBuilder<DB>) {
        build_chart_2D(chart, &self.data_series, &self.color);
    }
}

fn build_chart_2D<DB: DrawingBackend>(mut chart: ChartBuilder<DB>, data_series: &[TimeSeries], color: &RGBColor) {
    use plotters::prelude::*;
    const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

    // Acquire time range
    let newest_time = data_series
        .iter()
        .flat_map(|series| series.data.front().map(|(time, _)| *time))
        .max()
        .unwrap_or_else(|| Utc::now());
    let oldest_time = newest_time - chrono::Duration::seconds(PLOT_SECONDS as i64);

    let mut chart = chart
        .x_label_area_size(0)
        .y_label_area_size(28)
        .margin(20)
        .build_cartesian_2d(oldest_time..newest_time, 0.0f32..100.0f32)
        .expect("failed to build chart");

    chart
        .configure_mesh()
        .bold_line_style(color.mix(0.1))
        .light_line_style(color.mix(0.05))
        .axis_style(ShapeStyle::from(color.mix(0.45)).stroke_width(1))
        .y_labels(10)
        .y_label_style(
            ("sans-serif", 15)
                .into_font()
                .color(&color.mix(0.65))
                .transform(FontTransform::Rotate90),
        )
        .y_label_formatter(&|y: &f32| format!("{}%", y))
        .draw()
        .expect("failed to draw chart mesh");

    for series in data_series {
        chart
            .draw_series(LineSeries::new(
                series.data.iter().map(|(time, value)| (*time, *value)),
                series.color,
            ))
            .expect("failed to draw chart data")
            .label(&series.label)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], series.color.stroke_width(2)));
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(&WHITE.mix(0.8))
        .draw()
        .expect("failed to draw legend");
}
