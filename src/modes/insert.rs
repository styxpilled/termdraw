use std::io::Stdout;

use crate::{data::*, handlers::handle_keyboard};
use crossterm::{
    cursor::{self, position},
    event::{Event, KeyCode},
    execute, queue,
};

pub fn insert(event: Event, stdout: &mut Stdout, state: &mut State) {
    handle_keyboard(&event, |key| {
        let (col, row) = position().unwrap_or_default();
        match key.code {
            KeyCode::Char(code) => {
                state.virtual_display.set(
                    col,
                    row,
                    Layer {
                        brush: code,
                        brush_color: state.color,
                        changed: true,
                    },
                );
                state.pos.0 += 1;
                execute!(stdout, cursor::MoveRight(1)).unwrap();
            }
            KeyCode::Left => {
                execute!(stdout, cursor::MoveLeft(1)).unwrap();
            }
            KeyCode::Right => {
                execute!(stdout, cursor::MoveRight(1)).unwrap();
            }
            KeyCode::Up => {
                execute!(stdout, cursor::MoveUp(1)).unwrap();
            }
            KeyCode::Down => {
                execute!(stdout, cursor::MoveDown(1)).unwrap();
            }
            KeyCode::Backspace => {
                // TODO: update display
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
    });
}
