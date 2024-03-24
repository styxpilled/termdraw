use crate::modes::{self, Mode};
use crossterm::{
    event::Event,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
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
    pub ui: UI,
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
                modes::pencil(&event, self);
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
            Mode::Hex(_) => {
                modes::hex(&event, self);
            }
        }
    }

    pub fn eyedrop(&mut self, col: u16, row: u16) {
        self.color = self
            .virtual_display
            .get(col, row)
            .and_then(|el| Some(el.brush_color))
            .unwrap_or_else(|| crossterm::style::Color::Black);
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
    _Hex,
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Command::Enter(mode) => format!("ENTER {}", mode),
                Command::Clear => "CLEAR".to_string(),
                Command::None => "REDO".to_string(),
                // Command::Hex => "HEX".to_string(),
                // Command::Undo => "UNDO".to_string(),
                // Command::Redo => "REDO".to_string(),
                _ => "".to_string(),
            }
        )
    }
}

pub struct UI {
    pub elements: Vec<Element>,
}

impl UI {
    pub fn draw(&self, stdout: &mut Stdout, max_width: usize, bg_color: Color) {
        let used_width: usize = self.elements.iter().map(|el| el.get_width()).sum();
        let free_space = if used_width > max_width {
            0
        } else {
            max_width - used_width
        };
        let pad_len = free_space / (self.elements.len() - 1);
        let final_pad_len = free_space % (self.elements.len() - 1);
        let len = self.elements.len();
        for (i, element) in self.elements.iter().enumerate() {
            element.draw(stdout, bg_color);
            let pad = if i + 1 == len { final_pad_len } else { pad_len };
            queue!(
                stdout,
                SetBackgroundColor(Color::DarkGrey),
                Print(" ".repeat(pad))
            )
            .unwrap();
        }
    }
}

pub struct Element {
    pub nodes: Vec<Node>,
}

impl Element {
    pub fn draw(&self, stdout: &mut Stdout, bg_color: Color) {
        let len = self.nodes.len();
        for (i, node) in self.nodes.iter().enumerate() {
            let bg_color = if let Some(color) = node.bg {
                color
            } else {
                bg_color
            };
            queue!(
                stdout,
                SetBackgroundColor(bg_color),
                SetForegroundColor(node.color),
                Print(if i == 0 { " " } else { "" }),
                Print(&node.value),
                Print(if i == len - 1 { " " } else { "" })
            )
            .unwrap();
        }
    }
    pub fn get_width(&self) -> usize {
        let width: usize = self.nodes.iter().map(|node| node.value.len()).sum();
        width + 2
    }
}

pub struct Node {
    pub value: String,
    pub color: Color,
    pub bg: Option<Color>,
}

impl Node {
    pub fn new(value: String, color: Color) -> Self {
        Node {
            value,
            color,
            bg: None,
        }
    }
}
