use crate::modes::Mode;
use crate::{data::*, handlers::handle_keychar};
use crossterm::{
    cursor,
    event::Event,
    queue,
    style::Color,
    terminal::{Clear, ClearType},
};
use std::io::Stdout;

pub fn command(event: &Event, stdout: &mut Stdout, state: &mut State) {
    handle_keychar(&event, |c| {
        state.command = match c {
            'i' => {
                queue!(stdout, cursor::Show).unwrap();
                state.mode = Mode::Insert;
                Command::EnterInsertMode
            }
            'd' => {
                state.mode = Mode::Pencil(super::PencilData { pencil: '*' });
                Command::EnterPencilMode
            }
            'e' => {
                state.mode = Mode::Eyedropper;
                Command::EnterEyedropperMode
            }
            'b' => {
                state.mode = Mode::Brush(super::BrushData {
                    size: 1,
                    mode: super::BrushMode::Add,
                });
                Command::EnterBrushMode
            }
            'c' => {
                state.mode = Mode::ContentBrush;
                Command::EnterContentBrushMode
            }
            'q' => {
                queue!(stdout, Clear(ClearType::All)).unwrap();
                // state.history = vec![];
                Command::Clear
            }
            'f' => {
                // TODO: support rgb color
                let n = state
                    .colors
                    .iter()
                    .position(|n| n == &state.color)
                    .unwrap_or_default();
                let index = if n + 1 < state.colors.len() { n + 1 } else { 0 };
                state.color = state.colors[index];
                Command::Undo
            }
            'u' => {
                queue!(stdout, Clear(ClearType::All)).unwrap();
                // let undo = state.history.pop();
                // if undo.is_some() {
                // state.redo_layers.push(undo.unwrap());
                // }
                // frame_state.need_repaint = true;
                Command::Undo
            }
            'y' => {
                // let redo = state.redo_layers.pop();
                // if redo.is_some() {
                //     state.history.push(redo.unwrap());
                // }
                Command::Redo
            }
            _ => state.command,
        }
    });
}
