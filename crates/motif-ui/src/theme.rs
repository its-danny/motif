use iced::Color;

// Zinc palette (tailwind/shadcn inspired)
pub const ZINC_950: Color = Color::from_rgb(0.035, 0.035, 0.043); // #09090b
pub const ZINC_900: Color = Color::from_rgb(0.094, 0.094, 0.106); // #18181b
pub const ZINC_800: Color = Color::from_rgb(0.153, 0.153, 0.165); // #27272a
pub const ZINC_700: Color = Color::from_rgb(0.247, 0.247, 0.271); // #3f3f46
pub const ZINC_500: Color = Color::from_rgb(0.443, 0.443, 0.478); // #71717a
pub const ZINC_400: Color = Color::from_rgb(0.631, 0.631, 0.667); // #a1a1aa
pub const ZINC_200: Color = Color::from_rgb(0.894, 0.894, 0.906); // #e4e4e7
pub const ZINC_50: Color = Color::from_rgb(0.980, 0.980, 0.984); // #fafafa

// Mode accent colors
pub const BLUE_500: Color = Color::from_rgb(0.231, 0.510, 0.965); // #3b82f6
pub const GREEN_500: Color = Color::from_rgb(0.133, 0.804, 0.475); // #22c55e
pub const AMBER_500: Color = Color::from_rgb(0.961, 0.620, 0.043); // #f59e0b
pub const PURPLE_500: Color = Color::from_rgb(0.541, 0.361, 0.949); // #8b5cf6
pub const ROSE_500: Color = Color::from_rgb(0.957, 0.259, 0.388); // #f43f62

use crate::app::Mode;

impl Mode {
    pub fn label(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Mode::Normal => BLUE_500,
        }
    }
}
