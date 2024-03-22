use std::io::Stdout;

use crate::{data::*, LUMA_VALUES};
use crossterm::{
    event::{Event, MouseButton, MouseEventKind},
    terminal,
};

pub fn content_brush(event: Event, _stdout: &mut Stdout, state: &mut State) {
    match event {
        Event::Mouse(ev) => match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                let (x, y) = (ev.column, ev.row);
                let mut average_luma = 0;
                let (mx, my) = terminal::size().unwrap_or_default();
                let mut divider = 0;
                let col_range =
                    if x > 1 { x - 1 } else { x }..if x + 1 < mx { x + 1 } else { x } + 1;
                let row_range =
                    if y > 1 { y - 1 } else { y }..if y + 1 < my { y + 1 } else { y } + 1;
                for n in col_range {
                    for i in row_range.clone() {
                        divider += 1;
                        average_luma += LUMA_VALUES
                            .iter()
                            .position(|&val| {
                                val == state
                                    .virtual_display
                                    .get(n, i)
                                    .and_then(|el| Some(el.brush))
                                    .unwrap_or('a')
                            })
                            .unwrap_or(50);
                    }
                }
                average_luma = average_luma / divider;
                state.virtual_display.set(
                    ev.column,
                    ev.row,
                    Layer {
                        brush: LUMA_VALUES[average_luma],
                        brush_color: state.color,
                        changed: true,
                    },
                );
            }
            _ => {}
        },
        _ => {}
    }
}
