use crate::{
    data::*,
    handlers::{handle_click, handle_keychar},
};
use crossterm::event::Event;

pub fn pencil(event: &Event, state: &mut State) {
    let m_test = match &mut state.mode {
        super::Mode::Pencil(t) => t,
        _ => unreachable!(),
    };
    handle_click(event, |_, col, row| {
        state.virtual_display.set(
            col,
            row,
            Layer {
                brush: m_test.pencil,
                brush_color: state.color,
                changed: true,
            },
        );
    });
    handle_keychar(event, |code| {
        m_test.pencil = code;
    });
}
