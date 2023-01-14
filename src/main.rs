use std::{io::stdout, time::Duration};

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor,
    cursor::position,
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Result,
};
const HELP: &str = r#"EventStream based on futures_util::Stream with tokio
 - Keyboard, mouse and terminal resize events enabled
 - Prints "." every second if there's no event
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

async fn print_events() {
    let mut reader = EventStream::new();
    let mut brush = '*';

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();

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
                                        let t = cursor::MoveTo( ev.column,  ev.row);
                                        execute!(stdout(),t).unwrap();
                                        print!("{brush}");
                                    },
                                    _ => {}
                                }

                            },
                            Event::Key(ev) => {
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
