use std::fmt::Display;

pub mod chart;
pub mod info;
pub mod optimization;
pub mod settings;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Page {
    Chart,
    Info,
    Optimization,
    Settings,
}

impl Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Chart => write!(f, "Chart"),
            Page::Info => write!(f, "Info"),
            Page::Optimization => write!(f, "Optimization"),
            Page::Settings => write!(f, "Settings"),
        }
    }
}
