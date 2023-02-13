use std::io::Stdout;

use crate::data::*;
use crossterm::event::{Event, MouseButton, MouseEventKind};

pub fn eyedropper(
    event: Event,
    _stdout: &mut Stdout,
    state: &mut State,
    _frame_state: &mut FrameState,
) {
    match event {
        Event::Mouse(ev) => match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                state.brush =
                    state.virtual_display[usize::from(ev.column)][usize::from(ev.row)].brush;
                state.brush_color =
                    state.virtual_display[usize::from(ev.column)][usize::from(ev.row)].brush_color;
            }
            _ => {}
        },
        _ => {}
    };
}
