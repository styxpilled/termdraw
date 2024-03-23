use crate::{data::*, handlers::handle_click};
use crossterm::event::Event;
use std::io::Stdout;

pub fn eyedropper(event: &Event, _: &mut Stdout, state: &mut State) {
    handle_click(event, |_, col, row| {
        state
            .virtual_display
            .get(col, row)
            .and_then(|el| Some(el.brush_color))
            .unwrap_or_else(|| crossterm::style::Color::Black);
    });
}
