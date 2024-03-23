use crate::modes::{self, Mode};
use crossterm::{event::Event, style::Color};
use std::{fmt, fmt::Display, io::Stdout};

pub struct State {
    pub repaint_counter: u32,
    pub mode: Mode,
    pub color: Color,
    pub pos: (u16, u16),
    pub command: Command,
    pub drag_pos: (u16, u16),
    pub colors: Vec<Color>,
    // pub history: Vec<HistoryPage>,
    pub virtual_display: Canvas,
    // pub redo_layers: Vec<HistoryPage>,
}

impl State {
    pub fn run(&mut self, event: &Event, stdout: &mut Stdout) {
        match &self.mode {
            Mode::Command => {}
            Mode::Insert => {
                modes::insert(&event, stdout, self);
            }
            Mode::Pencil(_) => {
                modes::pencil(&event, stdout, self);
            }
            Mode::ContentBrush => {
                modes::content_brush(&event, stdout, self);
            }
            Mode::Eyedropper => {
                modes::eyedropper(&event, self);
            }
            Mode::Brush(_) => {
                modes::brush(&event, stdout, self);
            }
        }
    }
}

pub struct Canvas {
    pub vd: Vec<Vec<Layer>>,
    pub need_repaint: bool,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Canvas {
        let width = width as usize;
        let height = height as usize;
        let mut virtual_display = Vec::with_capacity(width);
        for _ in 0..width {
            let mut nested = Vec::with_capacity(height);
            for _ in 0..height {
                nested.push(Layer {
                    brush: ' ',
                    brush_color: Color::White,
                    changed: false,
                })
            }
            virtual_display.push(nested);
        }
        Canvas {
            vd: virtual_display,
            need_repaint: false,
        }
    }
    pub fn set(&mut self, col: u16, row: u16, layer: Layer) {
        let col = col as usize;
        let row = row as usize;
        if col < self.vd.len() - 1 && row < self.vd[0].len() - 1 {
            self.vd[col][row] = layer;
            self.need_repaint = true;
        }
    }
    pub fn get(&self, col: u16, row: u16) -> Option<&Layer> {
        let col = col as usize;
        let row = row as usize;
        self.vd.get(col)?.get(row)
    }
}

#[derive(Copy, Clone)]
pub struct Layer {
    pub brush: char,
    pub brush_color: Color,
    pub changed: bool,
}

#[derive(Copy, Clone)]
pub struct TextLayer {
    pub brush: char,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub enum Command {
    Enter(Mode),
    _Save,
    Clear,
    _Undo,
    _Redo,
    None,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Command::Enter(mode) => format!("ENTER {}", mode),
                Command::Clear => "CLEAR".to_string(),
                Command::None => "REDO".to_string(),
                // Command::Undo => "UNDO".to_string(),
                // Command::Redo => "REDO".to_string(),
                _ => "".to_string(),
            }
        )
    }
}
