use crate::{data::*, handlers::handle_keychar};
use crossterm::{event::Event, style::Color};

use super::{BrushData, Mode};

pub fn hex(event: &Event, state: &mut State) {
    handle_keychar(event, |code| {
        let data = match &mut state.mode {
            super::Mode::Hex(t) => t,
            _ => unreachable!(),
        };
        let maybe_hex = code.to_digit(16);
        // TODO: this is awful
        if let Some(val) = maybe_hex {
            if data.r.0.is_none() {
                data.r.0 = Some(val.try_into().unwrap());
            } else if data.r.1.is_none() {
                data.r.1 = Some(val.try_into().unwrap());
            } else if data.r.1.is_none() {
                data.r.1 = Some(val.try_into().unwrap());
            } else if data.g.0.is_none() {
                data.g.0 = Some(val.try_into().unwrap());
            } else if data.g.1.is_none() {
                data.g.1 = Some(val.try_into().unwrap());
            } else if data.b.0.is_none() {
                data.b.0 = Some(val.try_into().unwrap());
            } else if data.b.1.is_none() {
                data.b.1 = Some(val.try_into().unwrap());
            }
            // for v in [data.r, data.g, data.b].iter_mut() {
            //     if v.0.is_none() {
            //         v.0 = Some(val.try_into().unwrap());
            //         state.repaint_counter = val.clone();
            //         break;
            //     } else if v.1.is_none() {
            //         v.1 = Some(val.try_into().unwrap());
            //         state.repaint_counter = val.clone();
            //         break;
            //     }
            // }
        }
        // not the finest rust ever written
        // TODO make this nice somehow
        match (data.r, data.g, data.b) {
            ((Some(r1), Some(r2)), (Some(g1), Some(g2)), (Some(b1), Some(b2))) => {
                state.color = Color::Rgb {
                    r: r1 * 16 + r2,
                    g: g1 * 16 + g2,
                    b: b1 * 16 + b2,
                };
                state.mode = Mode::Brush(BrushData::default())
            }
            _ => {}
        }
    });
}
