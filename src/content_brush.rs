use std::io::Stdout;

use crate::{data::*, LUMA_VALUES};
use crossterm::{
    event::{Event, MouseButton, MouseEventKind},
    terminal,
};

pub fn content_brush(
    event: Event,
    _stdout: &mut Stdout,
    state: &mut State,
    frame_state: &mut FrameState,
) {
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
                                val == state.virtual_display[usize::from(n)][usize::from(i)].brush
                            })
                            .unwrap_or(50);
                    }
                }
                average_luma = average_luma / divider;
                state.virtual_display[usize::from(ev.column)][usize::from(ev.row)] = Layer {
                    brush: LUMA_VALUES[average_luma],
                    brush_color: state.brush_color,
                    changed: true,
                    x: ev.column,
                    y: ev.row,
                };
                state.redo_layers = vec![];
                frame_state.need_repaint = true;
            }
            _ => {}
        },
        _ => {}
    }
}
