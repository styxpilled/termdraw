mod command;
mod content_brush;
mod eyedropper;
mod insert;
mod pencil;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Pencil,
    Insert,
    Command,
    Eyedropper,
    ContentBrush,
}

pub use command::command;
pub use content_brush::content_brush;
pub use eyedropper::eyedropper;
pub use insert::insert;
pub use pencil::pencil;
