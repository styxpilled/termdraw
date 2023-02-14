use std::io::Stdout;

use crate::{data::*, LUMA_VALUES};
use crossterm::{
    event::{Event, KeyCode, MouseButton, MouseEventKind},
    terminal,
};

use std::cmp::{max, min};

pub fn brush(event: Event, _stdout: &mut Stdout, state: &mut State, frame_state: &mut FrameState) {
    match event {
        Event::Key(ev) => match ev.code {
            KeyCode::Char(code) => {
                state.brush_mode = match code {
                    'a' => BrushMode::Add,
                    'b' => BrushMode::Subtract,
                    _ => state.brush_mode,
                }
            }
            _ => {}
        },
        Event::Mouse(ev) => match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                let (x, y) = (ev.column, ev.row);
                let (mx, my) = terminal::size().unwrap_or_default();
                let col_range = if x > 2 {
                    x - 2
                } else if x > 1 {
                    x - 1
                } else {
                    x
                }..if x + 2 < mx {
                    x + 2
                } else if x + 1 < mx {
                    x + 1
                } else {
                    x
                } + 1;
                let row_range =
                    if y > 1 { y - 1 } else { y }..if y + 1 < my { y + 1 } else { y } + 1;
                for n in col_range {
                    for i in row_range.clone() {
                        let new_luma = if (n + 2 - x == 0 || n + 2 - x == 4) && i != y {
                            0
                        } else if (i != y && n != x) || (n + 2 - x) % 4 == 0 {
                            4
                        } else {
                            8
                        };
                        let old_luma = LUMA_VALUES
                            .iter()
                            .position(|&val| {
                                val == state.virtual_display[usize::from(n)][usize::from(i)].brush
                            })
                            .unwrap_or(0);
                        let luma_value = match state.brush_mode {
                            BrushMode::Add => {
                                LUMA_VALUES[min(old_luma + new_luma, LUMA_VALUES.len() - 1)]
                            }

                            BrushMode::Subtract => {
                                LUMA_VALUES[if old_luma < new_luma {
                                    0
                                } else {
                                    old_luma - new_luma
                                }]
                            }
                        };
                        state.virtual_display[usize::from(n)][usize::from(i)] = Layer {
                            brush: luma_value,
                            brush_color: state.brush_color,
                            changed: true,
                            x: n,
                            y: i,
                        };
                    }
                }
                state.redo_layers = vec![];
                frame_state.need_repaint = true;
            }
            _ => {}
        },
        _ => {}
    }
}
