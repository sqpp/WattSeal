// ## Architecture des données

// Événements (timestamp, valeur):
//     - POWER :
//         - Intel RAPL (PKG, PP0, PP1, DRAM)
//         - AMD RAPL
//         - NVSMI
//         - RAM (estimation)
//         - Disques, périphériques (estimation)
//         - Autres
//         - TOTAL
//     - UTILISATION :
//         - CPU
//         - GPU (NVSMI)
//         - RAM

// Configuration

use std::{fmt::Display, time::SystemTime};

#[derive(Debug, Clone)]
pub struct Event<T> {
    time: SystemTime,
    value: T,
}

impl Display for Event<f64> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {:?}", self.value, self.time)
    }
}

impl<T> Event<T> {
    pub fn new(value: T) -> Self {
        Event {
            time: SystemTime::now(),
            value,
        }
    }

    pub fn time(&self) -> SystemTime {
        self.time
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

pub enum OS {
    Windows,
    Linux,
    MacOS,
}