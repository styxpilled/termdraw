use std::io::Stdout;

use crate::{
    data::*,
    handlers::{handle_click, handle_keychar, handle_mouse},
    LUMA_VALUES,
};
use crossterm::{
    event::{Event, MouseEventKind},
    terminal,
};

use std::cmp::min;

use super::BrushMode;

pub fn base_brush<F>(state: &mut State, col: u16, row: u16, radius: i32, mut f: F)
where
    F: FnMut(&mut State, usize, u16, u16),
{
    let (mx, my) = terminal::size().unwrap_or_default();
    let (mx, my) = (mx.try_into().unwrap(), my.try_into().unwrap());

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
        let data = match &mut state.mode {
            super::Mode::Brush(t) => t,
            _ => unreachable!(),
        };
        handle_keychar(&event, |c| match c {
            'a' => data.mode = BrushMode::Add,
            'f' => data.mode = BrushMode::Subtract,
            's' => data.size += 1,
            'd' => data.size = if data.size == 1 { 1 } else { data.size - 1 },
            _ => {}
        });
        handle_mouse(event, |ev| match ev.kind {
            MouseEventKind::ScrollUp => data.size += 1,
            MouseEventKind::ScrollDown => {
                data.size = if data.size == 1 { 1 } else { data.size - 1 }
            }
            _ => {}
        });
        (data.mode.clone(), data.size)
    };

    handle_click(event, |_, col, row| {
        base_brush(
            state,
            col,
            row,
            size.try_into().unwrap(),
            |state: &mut State, new_luma: usize, col: u16, row: u16| {
                let old_luma = state
                    .virtual_display
                    .get(col, row)
                    .and_then(|el| LUMA_VALUES.into_iter().position(|x| x == el.brush))
                    .unwrap_or(0);
                let luma_value = match mode {
                    BrushMode::Add => {
                        let old_luma = old_luma / 4 + (rand::random::<u8>() / 4) as usize;
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
    });
}
