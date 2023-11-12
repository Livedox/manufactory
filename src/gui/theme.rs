use egui::Color32;

pub struct Theme {
    pub primary: Color32,
    pub on_primary: Color32,
    pub primary_container: Color32,
    pub on_primary_container: Color32,

    pub secondary: Color32,
    pub on_secondary: Color32,
    pub secondary_container: Color32,
    pub on_secondary_container: Color32,

    pub tertiary: Color32,
    pub on_tertiary: Color32,
    pub secondary_tertiary: Color32,
    pub on_secondary_tertiary: Color32,

    pub error: Color32,
    pub on_error: Color32,
    pub secondary_error: Color32,
    pub on_secondary_error: Color32,

    pub background: Color32,
    pub on_background: Color32,

    pub surface: Color32,
    pub on_surface: Color32,

    pub surface_variant: Color32,
    pub on_surface_variant: Color32,

    pub outline: Color32,
}

pub const DEFAULT_THEME: Theme = Theme {
    primary: Color32::from_rgb(208,188,255),
    on_primary: Color32::from_rgb(56,30,114),
    primary_container: Color32::from_rgb(79,55,138),
    on_primary_container: Color32::from_rgb(233,221,255),

    secondary: Color32::from_rgb(204,194,219),
    on_secondary: Color32::from_rgb(51,45,65),
    secondary_container: Color32::from_rgb(74,68,88),
    on_secondary_container: Color32::from_rgb(232,222,248),

    tertiary: Color32::from_rgb(239,184,200),
    on_tertiary: Color32::from_rgb(74,37,50),
    secondary_tertiary: Color32::from_rgb(99,59,72),
    on_secondary_tertiary: Color32::from_rgb(255,217,227),

    error: Color32::from_rgb(255,180,171),
    on_error: Color32::from_rgb(105,0,5),
    secondary_error: Color32::from_rgb(147,0,10),
    on_secondary_error: Color32::from_rgb(255,218,214),

    background: Color32::from_rgb(28,27,30),
    on_background: Color32::from_rgb(230,225,230),

    surface: Color32::from_rgb(20,19,22),
    on_surface: Color32::from_rgb(202,197,202),

    surface_variant: Color32::from_rgb(73,69,78),
    on_surface_variant: Color32::from_rgb(202,196,207),

    outline: Color32::from_rgb(148,143,153),
};