use std::io::Stdout;

use crate::data::*;
use crossterm::event::{Event, MouseButton, MouseEventKind};

pub fn eyedropper(event: Event, _stdout: &mut Stdout, state: &mut State) {
    match event {
        Event::Mouse(ev) => match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                // state.brush =
                //     state.virtual_display[usize::from(ev.column)][usize::from(ev.row)].brush;
                state.color = state
                    .virtual_display
                    .get(ev.column, ev.row)
                    .and_then(|el| Some(el.brush_color))
                    .unwrap_or_else(|| crossterm::style::Color::Black);
            }
            _ => {}
        },
        _ => {}
    };
}
