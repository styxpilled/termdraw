use std::{io::stdout, time::Duration};

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor,
    cursor::position,
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    Result,
};
const HELP: &str = r#"EventStream based on futures_util::Stream with tokio
 - Keyboard, mouse and terminal resize events enabled
 - Prints "." every second if there's no event
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

// enum Mode {
// Draw,
// Insert,
// }

async fn print_events() {
    let mut reader = EventStream::new();
    let mut brush = '*';
    // let mut mode = Mode::Draw;

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();
        let mut stdo = stdout();

        select! {
            _ = delay => {  },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        match event {
                            Event::Mouse(ev) => {
                                match ev.kind {
                                    MouseEventKind::Drag(_) |
                                    MouseEventKind::Down(_) => {
                                        execute!(stdo, cursor::MoveTo(ev.column, ev.row)).unwrap();
                                        print!("{brush}");
                                    },
                                    _ => {}
                                }

                            },
                            Event::Key(ev) => {
                                if ev.modifiers == KeyModifiers::SHIFT {
                                    print!("{:?}", ev);
                                }
                                match ev.code {
                                    KeyCode::Char(code) => {
                                        brush = code;
                                    },
                                    _ => {}
                                }
                            }
                            _ => {}
                        };
                        if event == Event::Key(KeyCode::Char('c').into()) {
                            println!("Cursor position: {:?}\r", position());
                        }

                        if event == Event::Key(KeyCode::Esc.into()) {
                            break;
                        }
                        let (max_x, googa)= size().unwrap_or_default();
                        execute!(stdo, cursor::MoveTo( 0, googa)).unwrap();
                        print!("INSERT MODE, {max_x}, {googa}");
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", HELP);

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(
        stdout,
        Clear(ClearType::All),
        EnableMouseCapture,
        cursor::Hide
    )?;

    print_events().await;

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
