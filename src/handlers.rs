use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

pub fn handle_keyboard<F>(event: &Event, mut f: F)
where
    F: FnMut(&KeyEvent),
{
    if let Event::Key(e) = event {
        f(e);
    }
}

pub fn handle_keycode<F>(event: &Event, mut f: F)
where
    F: FnMut(KeyCode),
{
    handle_keyboard(event, |ev| f(ev.code));
}

pub fn handle_keychar<F>(event: &Event, mut f: F)
where
    F: FnMut(char),
{
    handle_keycode(event, |code| {
        if let KeyCode::Char(c) = code {
            f(c);
        }
    })
}

pub fn handle_mouse<F>(event: &Event, mut f: F)
where
    F: FnMut(&MouseEvent),
{
    if let Event::Mouse(ev) = event {
        f(ev);
    }
}

pub fn handle_click<F>(event: &Event, mut f: F)
where
    F: FnMut(&MouseEvent, u16, u16),
{
    handle_mouse(event, |ev| match ev.kind {
        MouseEventKind::Drag(_) | MouseEventKind::Down(_) => {
            f(ev, ev.column, ev.row);
        }
        _ => {}
    });
}

pub fn get_click_pos(event: &Event) -> Option<(u8, u8)> {
    let mut v = None;
    handle_mouse(event, |ev| {
        v = match ev.kind {
            MouseEventKind::Drag(_) | MouseEventKind::Down(_) => {
                Some((ev.column as u8, ev.row as u8))
            }
            _ => None,
        }
    });
    v
}
