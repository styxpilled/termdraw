use crate::{
    data::*,
    handlers::{handle_click, handle_keychar},
};
use crossterm::event::Event;

pub fn pencil(event: &Event, state: &mut State) {
    let data = match &mut state.mode {
        super::Mode::Pencil(data) => data,
        _ => unreachable!(),
    };
    handle_click(event, |_, col, row| {
        state.virtual_display.set(
            col,
            row,
            Layer {
                brush: data.pencil,
                brush_color: state.color,
                changed: true,
            },
        );
    });
    handle_keychar(event, |code| {
        data.pencil = code;
    });
}
