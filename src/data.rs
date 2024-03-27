use crate::{
    handlers::{self, get_click_pos, handle_click},
    modes::{self, Mode},
};
use crossterm::{
    cursor::{self, position},
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
    pub pad: usize,
    // pub history: Vec<HistoryPage>,
    // pub ui: UI,
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

pub struct UI<'a> {
    pub elements: Vec<Widget>,
    pub stdout: &'a mut Stdout,
    // pub state: &mut State,
    pub pos: Option<(u8, u8)>,
    pub max: (usize, usize),
    pub pad: usize,
    pub offset: u8,
    pub bg_color: Color,
}

impl<'a> UI<'a> {
    pub fn render(&mut self, max: (usize, usize), bg_color: Color) -> usize {
        let (max_width, _max_height) = max;
        let used_width: usize = self.elements.iter().map(|el| el.get_width()).sum();
        let free_space = if used_width > max_width {
            0
        } else {
            max_width - used_width
        };
        self.pad = free_space / (self.elements.len() - 1);
        let final_pad_len = free_space % (self.elements.len() - 1);
        let len = self.elements.len();

        for (i, element) in self.elements.iter_mut().enumerate() {
            element.paint(self.stdout, bg_color);
            let pad = if i + 1 == len {
                final_pad_len
            } else {
                self.pad
            };
            queue!(
                self.stdout,
                SetBackgroundColor(Color::DarkGrey),
                // Print(format!("{}", offset)),
                Print(" ".repeat(pad)),
            )
            .unwrap();
            self.offset += pad as u8;
        }
        self.pad
    }

    pub fn add<F>(&mut self, mut f: F) -> Widget
    where
        F: FnMut() -> Widget,
    {
        self.push(f())
    }

    pub fn push(&mut self, mut el: Widget) -> Widget {
        if !self.elements.is_empty() {
            self.offset += self.pad as u8;
        }
        el.process(
            self.stdout,
            self.pos,
            self.max.1,
            &mut self.offset,
            Color::White,
        );
        self.elements.push(el.clone());
        el
    }
}

#[derive(Clone)]
pub struct Widget {
    pub text: String,
    pub color: Color,
    pub bg: Option<Color>,
    clicked: bool,
}

impl Widget {
    pub fn new<S: Into<String>>(text: S, color: Color) -> Self {
        Widget {
            text: text.into(),
            color,
            bg: None,
            clicked: false,
        }
    }

    pub fn clicked(&self) -> bool {
        self.clicked
    }

    pub fn process(
        &mut self,
        _stdout: &mut Stdout,
        // state: &mut State,
        pos: Option<(u8, u8)>,
        max_height: usize,
        offset: &mut u8,
        _bg_color: Color,
    ) {
        *offset += 1;
        if let Some((col, row)) = pos {
            if row == (max_height - 1) as u8
                && col > *offset
                && col < *offset + (self.text.len() + 1) as u8
            {
                self.clicked = true;
            };
        }

        *offset += self.text.len() as u8;

        *offset += 1;
    }

    pub fn paint(&self, stdout: &mut Stdout, bg_color: Color) {
        let bg_color = if let Some(color) = self.bg {
            color
        } else {
            bg_color
        };
        queue!(
            stdout,
            SetBackgroundColor(bg_color),
            SetForegroundColor(self.color),
            Print(" "),
            Print(&self.text),
            Print(" ")
        )
        .unwrap();
    }

    pub fn get_width(&self) -> usize {
        // let width: usize = self.nodes.iter().map(|node| node.value.len()).sum();
        let width = self.text.len();
        width + 2
    }
}
