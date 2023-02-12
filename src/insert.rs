use std::io::Stdout;

use crate::data::*;
use crossterm::{
    cursor::{self, position},
    event::{Event, KeyCode},
    execute, queue,
};

pub fn insert(event: Event, stdout: &mut Stdout, state: &mut State, frame_state: &mut FrameState) {
    match event {
        Event::Key(ev) => {
            let (column, row) = position().unwrap_or_default();
            match ev.code {
                KeyCode::Char(code) => {
                    state.virtual_display[usize::from(column)][usize::from(row)] = Layer {
                        brush: code,
                        brush_color: state.brush_color,
                        changed: true,
                        x: column,
                        y: row,
                    };
                    state.pos.0 += 1;
                    execute!(stdout, cursor::MoveRight(1)).unwrap();
                    frame_state.need_repaint = true;
                }
                KeyCode::Left => {
                    execute!(stdout, cursor::MoveLeft(1)).unwrap();
                    frame_state.need_repaint = true;
                }
                KeyCode::Right => {
                    execute!(stdout, cursor::MoveRight(1)).unwrap();
                    frame_state.need_repaint = true;
                }
                KeyCode::Up => {
                    execute!(stdout, cursor::MoveUp(1)).unwrap();
                    frame_state.need_repaint = true;
                }
                KeyCode::Down => {
                    execute!(stdout, cursor::MoveDown(1)).unwrap();
                    frame_state.need_repaint = true;
                }
                KeyCode::Backspace => {
                    queue!(
                        stdout,
                        cursor::MoveLeft(1),
                        crossterm::style::Print(" "),
                        cursor::MoveLeft(1)
                    )
                    .unwrap();
                }
                _ => {}
            };
        }
        _ => {}
    };
}
