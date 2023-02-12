use std::io::Stdout;

use crate::data::*;
use crossterm::{
    cursor,
    event::{Event, KeyCode},
    queue,
    style::Color,
    terminal::{Clear, ClearType},
};

pub fn command(
    event: Event,
    stdout: &mut Stdout,
    state: &mut State,
    frame_state: &mut FrameState,
    colors: &Vec<Color>,
) {
    match event {
        Event::Key(ev) => match ev.code {
            KeyCode::Char(code) => {
                state.command = match code {
                    'i' => {
                        queue!(stdout, cursor::Show).unwrap();
                        state.mode = Mode::Insert;
                        Command::EnterInsertMode
                    }
                    'd' => {
                        state.mode = Mode::Pencil;
                        Command::EnterPencilMode
                    }
                    'e' => {
                        state.mode = Mode::Eyedropper;
                        Command::EnterEyedropperMode
                    }
                    'b' => {
                        state.mode = Mode::Brush;
                        Command::EnterBrushMode
                    }
                    'q' => {
                        queue!(stdout, Clear(ClearType::All)).unwrap();
                        state.history = vec![];
                        frame_state.need_repaint = true;
                        Command::Clear
                    }
                    'f' => {
                        let n = colors
                            .iter()
                            .position(|n| n == &state.brush_color)
                            .unwrap_or_default();
                        let index = if n + 1 < colors.len() { n + 1 } else { 0 };
                        state.brush_color = colors[index];
                        Command::Undo
                    }
                    'u' => {
                        queue!(stdout, Clear(ClearType::All)).unwrap();
                        let undo = state.history.pop();
                        if undo.is_some() {
                            state.redo_layers.push(undo.unwrap());
                        }
                        frame_state.need_repaint = true;
                        Command::Undo
                    }
                    'y' => {
                        let redo = state.redo_layers.pop();
                        if redo.is_some() {
                            state.history.push(redo.unwrap());
                        }
                        frame_state.need_repaint = true;
                        Command::Redo
                    }
                    _ => state.command,
                }
            }
            _ => {}
        },
        _ => {}
    }
}
