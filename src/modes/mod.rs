mod brush;
mod command;
mod content_brush;
mod eyedropper;
mod insert;
mod pencil;

use std::fmt;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Brush,
    Pencil,
    Insert,
    Command,
    Eyedropper,
    ContentBrush,
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mode::Brush => "BRUSH",
                Mode::Eyedropper => "EYEDROPPER",
                Mode::Command => "COMMAND",
                Mode::Insert => "INSERT",
                Mode::Pencil => "PENCIL",
                Mode::ContentBrush => "CONTENT BRUSH",
            }
        )
    }
}

use std::fmt::Display;

pub use brush::brush;
pub use command::command;
pub use content_brush::content_brush;
pub use eyedropper::eyedropper;
pub use insert::insert;
pub use pencil::pencil;
