use iced::{
    Color, Theme,
    theme::{Base, Mode, Palette, Style},
};

macro_rules! define_themes {
    (
        builtin: [$($builtin:ident => $iced:ident),+ $(,)?]
        custom: [$($custom:ident => $name:literal: $palette:expr),+ $(,)?]
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub enum AppTheme {
            #[default]
            $($builtin,)+
            $($custom,)+
        }

        impl AppTheme {
            pub fn to_iced_theme(self) -> Theme {
                match self {
                    $(AppTheme::$builtin => Theme::$iced,)+
                    $(AppTheme::$custom => Theme::custom(String::from($name), $palette),)+
                }
            }

            pub fn name(self) -> &'static str {
                match self {
                    $(AppTheme::$builtin => stringify!($builtin),)+
                    $(AppTheme::$custom => $name,)+
                }
            }

            pub const fn all() -> &'static [AppTheme] {
                &[$(AppTheme::$builtin,)+ $(AppTheme::$custom,)+]
            }

            pub const fn custom_themes() -> &'static [AppTheme] {
                &[$(AppTheme::$custom,)+]
            }
        }
    };
}

define_themes! {
    builtin: [
        Light => Light,
        Dark => Dark,
        Dracula => Dracula,
        Nord => Nord,
        SolarizedLight => SolarizedLight,
        SolarizedDark => SolarizedDark,
        GruvboxLight => GruvboxLight,
        GruvboxDark => GruvboxDark,
        CatppuccinLatte => CatppuccinLatte,
        CatppuccinFrappe => CatppuccinFrappe,
        CatppuccinMacchiato => CatppuccinMacchiato,
        CatppuccinMocha => CatppuccinMocha,
        TokyoNight => TokyoNight,
        TokyoNightStorm => TokyoNightStorm,
        TokyoNightLight => TokyoNightLight,
        KanagawaWave => KanagawaWave,
        KanagawaDragon => KanagawaDragon,
        KanagawaLotus => KanagawaLotus,
        Moonfly => Moonfly,
        Nightfly => Nightfly,
        Oxocarbon => Oxocarbon,
        Ferra => Ferra,
    ]
    custom: [
        EcoEnergy => "Eco Energy": Palette {
            background: Color::from_rgb(0.12, 0.14, 0.16),
            text: Color::from_rgb(0.9, 0.92, 0.94),
            primary: Color::from_rgb(0.2, 0.78, 0.35),
            success: Color::from_rgb(0.2, 0.6, 0.86),
            danger: Color::from_rgb(0.95, 0.45, 0.25),
            warning: Color::from_rgb(0.95, 0.75, 0.25),
        },
        EcoEnergyLight => "Eco Energy Light": Palette {
            background: Color::from_rgb(0.96, 0.97, 0.98),
            text: Color::from_rgb(0.15, 0.18, 0.22),
            primary: Color::from_rgb(0.13, 0.58, 0.26),
            success: Color::from_rgb(0.15, 0.45, 0.65),
            danger: Color::from_rgb(0.85, 0.35, 0.15),
            warning: Color::from_rgb(0.85, 0.65, 0.15),
        },
        PowerSaver => "Power Saver": Palette {
            background: Color::BLACK,
            text: Color::from_rgb(0.6, 0.62, 0.64),
            primary: Color::from_rgb(0.15, 0.5, 0.25),
            success: Color::from_rgb(0.15, 0.4, 0.55),
            danger: Color::from_rgb(0.6, 0.3, 0.15),
            warning: Color::from_rgb(0.6, 0.5, 0.15),
        },
        HighContrast => "High Contrast": Palette {
            background: Color::BLACK,
            text: Color::WHITE,
            primary: Color::from_rgb(0.0, 1.0, 0.0),
            success: Color::from_rgb(0.0, 0.8, 1.0),
            danger: Color::from_rgb(1.0, 0.4, 0.0),
            warning: Color::from_rgb(1.0, 0.9, 0.0),
        },
        DeepOcean => "Deep Ocean Energy": Palette {
            background: Color::from_rgb(0.04, 0.07, 0.17), // Deep Navy
            text: Color::from_rgb(0.88, 0.98, 0.99),       // Off-White / Ice
            primary: Color::from_rgb(0.0, 0.71, 0.85),     // Electric Cyan
            success: Color::from_rgb(0.22, 0.69, 0.0),     // Neon Green (Low Power)
            danger: Color::from_rgb(0.85, 0.02, 0.16),     // Crimson (High Power)
            warning: Color::from_rgb(1.0, 0.72, 0.01),     // Yellow Gold
        },
        ArcticIce => "Arctic Ice": Palette {
            background: Color::from_rgb(0.96, 0.97, 0.96), // Ice White / Light Gray
            text: Color::from_rgb(0.17, 0.18, 0.26),       // Dark Slate
            primary: Color::from_rgb(0.10, 0.40, 0.62),    // Vivid Blue
            success: Color::from_rgb(0.16, 0.62, 0.56),    // Emerald
            danger: Color::from_rgb(0.90, 0.22, 0.27),     // Coral Red
            warning: Color::from_rgb(0.96, 0.64, 0.38),    // Amber
        },
        ElectricNight => "Electric Night": Palette {
            background: Color::from_rgb(0.07, 0.07, 0.07), // Near Black (OLED friendly)
            text: Color::from_rgb(0.88, 0.88, 0.88),       // Soft White
            primary: Color::from_rgb(0.48, 0.17, 0.75),    // Electric Purple
            success: Color::from_rgb(0.0, 0.96, 0.83),     // Neon Cyan
            danger: Color::from_rgb(0.95, 0.36, 0.71),     // Hot Pink / Red
            warning: Color::from_rgb(1.0, 0.89, 0.25),     // Neon Yellow
        },
        SolarFlare => "Solar Flare": Palette {
            background: Color::from_rgb(1.0, 0.98, 0.98),  // Warm White / Off-White
            text: Color::from_rgb(0.24, 0.20, 0.55),       // Dark Indigo / Charcoal
            primary: Color::from_rgb(0.97, 0.72, 0.0),     // Energetic Orange / Gold
            success: Color::from_rgb(0.46, 0.78, 0.58),    // Soft Green
            danger: Color::from_rgb(0.90, 0.22, 0.23),     // Brick Red
            warning: Color::from_rgb(0.96, 0.64, 0.38),    // Peach
        },
        GlacierSeal => "Glacier Seal": Palette {
            background: Color::from_rgb(0.95, 0.96, 0.98), // Ice White
            text: Color::from_rgb(0.04, 0.10, 0.18),       // Deep Navy / Near Black
            primary: Color::from_rgb(0.29, 0.41, 0.52),    // Seal Coat Blue-Gray
            success: Color::from_rgb(0.18, 0.55, 0.34),    // Frosty Evergreen (Power saving)
            danger: Color::from_rgb(0.85, 0.28, 0.27),     // High-contrast Coral
            warning: Color::from_rgb(0.83, 0.63, 0.09),    // Deep Mustard
        },
        MarianaTrench => "Mariana Trench": Palette {
            background: Color::from_rgb(0.05, 0.07, 0.13), // Deep Ocean Blue
            text: Color::from_rgb(1.0, 1.0, 1.0),          // Pure White
            primary: Color::from_rgb(0.0, 0.90, 1.0),      // Bioluminescent Cyan
            success: Color::from_rgb(0.0, 1.0, 0.50),      // Seaweed Green (Eco)
            danger: Color::from_rgb(1.0, 0.20, 0.40),      // Red Tide Alert
            warning: Color::from_rgb(1.0, 0.84, 0.0),      // Bright Yellow Tang
        },
        SleepMode => "Sleep Mode": Palette {
            background: Color::from_rgb(0.10, 0.13, 0.17), // Slate Navy
            text: Color::from_rgb(0.93, 0.95, 0.97),       // Crisp Off-White
            primary: Color::from_rgb(0.22, 0.70, 0.67),    // Efficiency Teal
            success: Color::from_rgb(0.28, 0.73, 0.47),    // Eco Green
            danger: Color::from_rgb(0.96, 0.40, 0.40),     // Soft Alert Red
            warning: Color::from_rgb(0.93, 0.79, 0.29),    // Mellow Yellow
        },CoastalShallows => "Coastal Shallows": Palette {
            background: Color::from_rgb(0.90, 0.97, 1.0),  // Shallow Water Blue
            text: Color::from_rgb(0.18, 0.22, 0.28),       // Dark Charcoal
            primary: Color::from_rgb(0.19, 0.51, 0.81),    // Deep Water Blue
            success: Color::from_rgb(0.22, 0.63, 0.41),    // Reef Green
            danger: Color::from_rgb(0.90, 0.24, 0.24),     // Fire Coral Red
            warning: Color::from_rgb(0.87, 0.42, 0.13),    // Starfish Orange
        },
        GeothermalCore => "Geothermal Core": Palette {
            background: Color::from_rgb(0.10, 0.09, 0.09), // Dark Obsidian
            text: Color::from_rgb(0.95, 0.93, 0.91),       // Ash White
            primary: Color::from_rgb(1.0, 0.40, 0.10),     // Magma Orange
            success: Color::from_rgb(0.30, 0.69, 0.31),    // Mineral Green
            danger: Color::from_rgb(0.90, 0.15, 0.15),     // Lava Red
            warning: Color::from_rgb(1.0, 0.75, 0.05),     // Sulfur Yellow
        },
        DeepCanopy => "Deep Canopy": Palette {
            background: Color::from_rgb(0.07, 0.10, 0.08), // Deep Forest Night
            text: Color::from_rgb(0.91, 0.96, 0.91),       // Pale Leaf White
            primary: Color::from_rgb(0.0, 0.90, 0.46),     // Vibrant Flora Green
            success: Color::from_rgb(0.25, 0.70, 0.65),    // River Teal
            danger: Color::from_rgb(1.0, 0.32, 0.32),      // Poison Dart Red
            warning: Color::from_rgb(1.0, 0.92, 0.0),      // Sunbeam Yellow
        },
        AtmosphericStatic => "Atmospheric Static": Palette {
            background: Color::from_rgb(0.09, 0.09, 0.13), // Thundercloud Navy
            text: Color::from_rgb(0.97, 0.97, 1.0),        // Crisp Lightning White
            primary: Color::from_rgb(0.70, 0.53, 1.0),     // Plasma Violet
            success: Color::from_rgb(0.0, 0.75, 0.65),     // Ozone Green
            danger: Color::from_rgb(1.0, 0.09, 0.27),      // Sprite Lightning Red
            warning: Color::from_rgb(1.0, 0.77, 0.0),      // Flash Yellow
        },
        SolarUmbra => "Solar Umbra": Palette {
            background: Color::from_rgb(0.08, 0.07, 0.06), // Eclipse Black
            text: Color::from_rgb(1.0, 0.99, 0.97),        // Starlight White
            primary: Color::from_rgb(1.0, 0.84, 0.31),     // Solar Gold
            success: Color::from_rgb(0.30, 0.71, 0.67),    // Earth Blue-Green
            danger: Color::from_rgb(0.90, 0.22, 0.21),     // Supernova Red
            warning: Color::from_rgb(1.0, 0.60, 0.0),      // Solar Wind Orange
        },
    ]
}

impl Base for AppTheme {
    fn default(preference: Mode) -> Self {
        match preference {
            Mode::Light => AppTheme::Light,
            Mode::Dark | Mode::None => AppTheme::Dark,
        }
    }

    fn mode(&self) -> Mode {
        let pal = AppTheme::palette(*self);
        let luminance = 0.299 * pal.background.r + 0.587 * pal.background.g + 0.114 * pal.background.b;
        if luminance < 0.5 { Mode::Dark } else { Mode::Light }
    }

    fn base(&self) -> Style {
        let pal = AppTheme::palette(*self);
        Style {
            background_color: pal.background,
            text_color: pal.text,
        }
    }

    fn palette(&self) -> Option<Palette> {
        Some(self.to_iced_theme().palette())
    }

    fn name(&self) -> &str {
        AppTheme::name(*self)
    }
}

impl AppTheme {
    pub fn palette(self) -> Palette {
        self.to_iced_theme().palette()
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
