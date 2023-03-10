use crate::modes::Mode;
use crossterm::style::Color;

pub struct State {
    pub repaint_counter: u32,
    pub mode: Mode,
    pub brush: char,
    pub brush_color: Color,
    pub brush_mode: BrushMode,
    pub pos: (u16, u16),
    pub command: Command,
    pub drag_pos: (u16, u16),
    // pub history: Vec<HistoryPage>,
    pub virtual_display: Vec<Vec<Layer>>,
    // pub redo_layers: Vec<HistoryPage>,
}

pub struct FrameState {
    pub need_repaint: bool,
}

#[derive(Copy, Clone)]
pub struct Layer {
    pub brush: char,
    pub brush_color: Color,
    pub x: u16,
    pub y: u16,
    pub changed: bool,
}

#[derive(Copy, Clone)]
pub struct TextLayer {
    pub brush: char,
    pub color: Color,
}

#[derive(Copy, Clone, Debug)]
pub enum BrushMode {
    Add,
    Subtract,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Command {
    EnterContentBrushMode,
    EnterEyedropperMode,
    EnterCommandMode,
    EnterInsertMode,
    EnterPencilMode,
    EnterBrushMode,
    Save,
    Clear,
    Undo,
    Redo,
    None,
}
