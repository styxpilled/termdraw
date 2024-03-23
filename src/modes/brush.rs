use std::io::Stdout;

use crate::{data::*, handlers::handle_keychar, LUMA_VALUES};
use crossterm::{
    event::{Event, MouseButton, MouseEvent, MouseEventKind},
    terminal,
};

use std::cmp::min;

use super::BrushMode;

pub fn base_brush<F>(ev: &MouseEvent, state: &mut State, radius: i32, mut f: F)
where
    F: FnMut(&mut State, usize, u16, u16),
{
    let (col, row) = (ev.column, ev.row);
    let (mx, my) = terminal::size().unwrap_or_default();
    let (mx, my) = (mx.try_into().unwrap(), my.try_into().unwrap());

    // let row_range = if y > 1 { y - 1 } else { y }..if y + 1 < my { y + 1 } else { y } + 1;
    let col: i32 = col.try_into().unwrap();
    let row: i32 = row.try_into().unwrap();
    for x in -radius..=radius {
        for y in -radius..=radius {
            let grr = ((x * x) * 100 / radius / 2) + ((y * y) * 100 / radius);
            if grr <= 100 {
                let xc: i32 = x + col;
                let yr: i32 = y + row;
                if xc > 0 && xc < mx && yr > 0 && yr < my {
                    f(
                        state,
                        (100 - grr) as usize,
                        xc.try_into().unwrap(),
                        yr.try_into().unwrap(),
                    );
                }
            }
        }
    }
}

pub fn brush(event: &Event, _stdout: &mut Stdout, state: &mut State) {
    let (mode, size) = {
        let m_test = match &mut state.mode {
            super::Mode::Brush(t) => t,
            _ => unreachable!(),
        };
        handle_keychar(&event, |c| match c {
            'a' => m_test.mode = BrushMode::Add,
            'f' => m_test.mode = BrushMode::Subtract,
            's' => m_test.size += 1,
            'd' => m_test.size = if m_test.size == 1 { 1 } else { m_test.size - 1 },
            _ => {}
        });
        (m_test.mode, m_test.size)
    };

    match event {
        Event::Mouse(ev) => match ev.kind {
            MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Down(MouseButton::Left) => {
                base_brush(
                    ev,
                    state,
                    size.try_into().unwrap(),
                    |state: &mut State, new_luma: usize, col: u16, row: u16| {
                        let old_luma = state
                            .virtual_display
                            .get(col, row)
                            .and_then(|el| LUMA_VALUES.into_iter().position(|x| x == el.brush))
                            .unwrap_or(0);
                        let old_luma = old_luma / 4 + (rand::random::<u8>() / 4) as usize;
                        let luma_value = match mode {
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
                        state.virtual_display.set(
                            col,
                            row,
                            Layer {
                                brush: luma_value,
                                brush_color: state.color,
                                changed: true,
                            },
                        );
                    },
                );
            }
            _ => {}
        },
        _ => {}
    }
}
