static TDP_TABLE: &[(&str, f64)] = &[
    // Intel Desktop (12th–14th gen)
    ("i9-14900", 125.0),
    ("i9-13900", 125.0),
    ("i9-12900", 125.0),
    ("i7-14700", 65.0),
    ("i7-13700", 65.0),
    ("i7-12700", 65.0),
    ("i5-14600", 65.0),
    ("i5-13600", 65.0),
    ("i5-12600", 65.0),
    ("i5-14400", 65.0),
    ("i5-13400", 65.0),
    ("i5-12400", 65.0),
    ("i3-14100", 60.0),
    ("i3-13100", 60.0),
    ("i3-12100", 60.0),
    // Intel Desktop (older)
    ("i7-8750H", 45.0),
    // Intel Mobile (U/P/H series)
    ("HX", 55.0),
    ("1370P", 28.0),
    ("1360P", 28.0),
    ("1355U", 15.0),
    ("1345U", 15.0),
    ("1335U", 15.0),
    ("1365U", 15.0),
    ("1265U", 15.0),
    ("1255U", 15.0),
    ("1235U", 15.0),
    // AMD Desktop (Ryzen 5000/7000/9000)
    ("7950X", 170.0),
    ("7900X", 170.0),
    ("5950X", 105.0),
    ("5900X", 105.0),
    ("7800X", 105.0),
    ("7700X", 105.0),
    ("5800X", 105.0),
    ("7600X", 105.0),
    ("5600X", 65.0),
    ("5600", 65.0),
    // AMD Mobile (U/HS series)
    ("7840U", 28.0),
    ("7840HS", 35.0),
    ("6800U", 28.0),
    ("7530U", 15.0),
    ("6600U", 15.0),
    ("7535HS", 35.0),
    // Apple Silicon
    ("Apple M1 Max", 60.0),
    ("Apple M1 Pro", 30.0),
    ("Apple M1", 10.0),
    ("Apple M2 Max", 75.0),
    ("Apple M2 Pro", 30.0),
    ("Apple M2", 15.0),
    ("Apple M3 Max", 92.0),
    ("Apple M3 Pro", 36.0),
    ("Apple M3", 15.0),
    ("Apple M4 Max", 100.0),
    ("Apple M4 Pro", 40.0),
    ("Apple M4", 20.0),
];

const DEFAULT_TDP: f64 = 65.0;
const DEFAULT_BOOST_MULTIPLIER: f64 = 1.25;

/// Looks up the TDP for a CPU model name, falling back to a default.
pub fn lookup_tdp(cpu_name: &str) -> f64 {
    let name_lower = cpu_name.to_lowercase();
    for (pattern, tdp) in TDP_TABLE {
        if name_lower.contains(&pattern.to_lowercase()) {
            return *tdp;
        }
    }
    DEFAULT_TDP
}

/// Non-linear power estimation.
///
/// `P = TDP_idle + (TDP_peak - TDP_idle) × usage^1.6`
///
/// Idle power is assumed to be ~20% of TDP.
pub fn estimate_power(tdp: f64, usage_percent: f64) -> f64 {
    let usage_frac = (usage_percent / 100.0).clamp(0.0, 1.0);
    let tdp_idle = tdp * 0.2;
    let tdp_peak = tdp * DEFAULT_BOOST_MULTIPLIER;
    tdp_idle + (tdp_peak - tdp_idle) * usage_frac.powf(1.6)
}

/// TDP-based CPU power estimator.
pub struct EstimationCPUSensor {
    tdp: f64,
}

impl EstimationCPUSensor {
    /// Creates an estimator with the given TDP value.
    pub fn new(tdp: f64) -> Self {
        Self { tdp }
    }

    /// Estimates current power draw from CPU usage percentage.
    pub fn estimate(&self, usage_percent: f64) -> f64 {
        estimate_power(self.tdp, usage_percent)
    }
}
