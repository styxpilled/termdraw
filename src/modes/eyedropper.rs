use crate::{data::*, handlers::handle_click};
use crossterm::event::Event;

pub fn eyedropper(event: &Event, state: &mut State) {
    handle_click(event, |_, col, row| {
        state.eyedrop(col, row);
    });
}
