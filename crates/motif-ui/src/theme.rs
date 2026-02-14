use iced::Color;

// Blue-steel DAW palette (dark, slightly desaturated)
pub const ZINC_950: Color = Color::from_rgb(0.094, 0.098, 0.145); // #181925
pub const ZINC_900: Color = Color::from_rgb(0.117, 0.125, 0.188); // #1e2030
pub const ZINC_800: Color = Color::from_rgb(0.196, 0.207, 0.286); // #32354a
pub const ZINC_700: Color = Color::from_rgb(0.266, 0.282, 0.380); // #444860
pub const ZINC_500: Color = Color::from_rgb(0.458, 0.482, 0.596); // #757b98
pub const ZINC_400: Color = Color::from_rgb(0.651, 0.675, 0.776); // #a6acc6
pub const ZINC_200: Color = Color::from_rgb(0.828, 0.847, 0.914); // #d3d8e9
pub const ZINC_50: Color = Color::from_rgb(0.925, 0.933, 0.965); // #ecedf6

// Mode accent colors
pub const BLUE_500: Color = Color::from_rgb(0.400, 0.620, 0.980); // #669ef9
pub const GREEN_500: Color = Color::from_rgb(0.367, 0.812, 0.620); // #5dcf9e
pub const AMBER_500: Color = Color::from_rgb(0.929, 0.722, 0.319); // #edb851
pub const PURPLE_500: Color = Color::from_rgb(0.612, 0.557, 0.871); // #9c8eda
pub const ROSE_500: Color = Color::from_rgb(0.914, 0.463, 0.576); // #e97693

use crate::app::Mode;

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Play => "PLAY",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Mode::Normal => BLUE_500,
            Mode::Play => GREEN_500,
        }
    }
}
