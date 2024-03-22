use crossterm::event::{Event, KeyCode, KeyEvent};

pub fn handle_keyboard<F>(event: &Event, mut f: F)
where
    F: FnMut(&KeyEvent),
{
    match event {
        Event::Key(e) => f(e),
        _ => {}
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
    handle_keycode(event, |code| match code {
        KeyCode::Char(c) => f(c),
        _ => {}
    })
}
