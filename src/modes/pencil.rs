use std::io::Stdout;

use crate::data::*;
use crossterm::{
    cursor::{self, position},
    event::{Event, KeyCode, MouseButton, MouseEventKind},
    queue,
};

pub fn pencil(event: Event, stdout: &mut Stdout, state: &mut State, frame_state: &mut FrameState) {
    match event {
        Event::Mouse(ev) => match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                // queue!(
                //     stdout,
                //     cursor::MoveTo(ev.column, ev.row),
                //     crossterm::style::Print(state.brush)
                // )
                // .unwrap();
                // state.history.push(HistoryPage::Pencil(Layer {
                //     brush: state.brush,
                //     brush_color: state.brush_color,
                //     changed: true,
                //     x: ev.column,
                //     y: ev.row,
                // }));
                state.virtual_display[usize::from(ev.column)][usize::from(ev.row)] = Layer {
                    brush: state.brush,
                    brush_color: state.brush_color,
                    changed: true,
                    x: ev.column,
                    y: ev.row,
                };
                // state.redo_layers = vec![];
                frame_state.need_repaint = true;
            }
            // MouseEventKind::Up()
            MouseEventKind::Down(MouseButton::Right) => {
                state.drag_pos = position().unwrap_or_default();
            }
            MouseEventKind::Drag(MouseButton::Right) => {
                queue!(stdout, cursor::MoveTo(state.drag_pos.0, state.drag_pos.1),).unwrap();
                for _ in state.drag_pos.0..ev.column {
                    queue!(stdout, crossterm::style::Print(state.brush),).unwrap();
                }
                for _ in state.drag_pos.1..ev.row {
                    queue!(
                        stdout,
                        crossterm::style::Print(state.brush),
                        cursor::MoveLeft(1),
                        cursor::MoveDown(1)
                    )
                    .unwrap();
                }
                queue!(stdout, cursor::MoveTo(state.drag_pos.0, state.drag_pos.1),).unwrap();
                for _ in state.drag_pos.1..ev.row {
                    queue!(
                        stdout,
                        crossterm::style::Print(state.brush),
                        cursor::MoveLeft(1),
                        cursor::MoveDown(1)
                    )
                    .unwrap();
                }
                for _ in state.drag_pos.0..ev.column {
                    queue!(stdout, crossterm::style::Print(state.brush),).unwrap();
                }
            }
            _ => {}
        },
        Event::Key(ev) => match ev.code {
            KeyCode::Char(code) => {
                state.brush = code;
            }
            _ => {}
        },
        _ => {}
    }
}
