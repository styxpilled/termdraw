mod brush;
mod content_brush;
mod eyedropper;
mod insert;
mod pencil;

use std::fmt;

#[derive(Debug, Clone)]
pub enum Mode {
    Brush(BrushData),
    Pencil(PencilData),
    Insert,
    Command,
    Eyedropper,
    ContentBrush,
}

#[derive(Debug, Clone)]
pub struct PencilData {
    pub pencil: char,
}

#[derive(Debug, Clone)]
pub struct BrushData {
    pub size: u8,
    pub mode: BrushMode,
}

#[derive(Debug, Clone)]
pub enum BrushMode {
    Add,
    Subtract,
}

impl Mode {
    pub fn get_color(&self) -> Color {
        match self {
            Mode::Brush(_) => Color::DarkGreen,
            Mode::Eyedropper => Color::DarkMagenta,
            Mode::Command => Color::DarkRed,
            Mode::Insert => Color::DarkCyan,
            Mode::Pencil(_) => Color::DarkYellow,
            Mode::ContentBrush => Color::Green,
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::Brush(_) => "BRUSH",
                Mode::Eyedropper => "EYEDROPPER",
                Mode::Command => "COMMAND",
                Mode::Insert => "INSERT",
                Mode::Pencil(_) => "PENCIL",
                Mode::ContentBrush => "CONTENT BRUSH",
            }
        )
    }
}

use std::fmt::Display;

pub use brush::brush;
pub use content_brush::content_brush;
use crossterm::style::Color;
pub use eyedropper::eyedropper;
pub use insert::insert;
pub use pencil::pencil;
