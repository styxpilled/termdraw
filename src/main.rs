use std::{
    io::{stdout, Stdout},
    time::Duration,
};

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

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    Draw,
    Insert,
    Command,
}

fn draw(event: Event, stdo: &mut Stdout, brush: &mut char, mode: &mut Mode) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        if *mode == Mode::Command {
            return false;
        }
        *mode = Mode::Command;
    }
    match event {
        Event::Mouse(ev) => {
            if *mode == Mode::Draw {
                match ev.kind {
                    MouseEventKind::Drag(_) | MouseEventKind::Down(_) => {
                        execute!(
                            stdo,
                            cursor::MoveTo(ev.column, ev.row),
                            crossterm::style::Print(*brush)
                        )
                        .unwrap();
                    }
                    _ => {}
                }
            }
        }
        Event::Key(ev) => {
            // if ev.modifiers == KeyModifiers::SHIFT {
            //     print!("{:?}", ev);
            // }
            match ev.code {
                KeyCode::Char(code) => {
                    match *mode {
                        Mode::Command => {
                            *mode = match code {
                                'i' => {
                                    execute!(stdo, cursor::Show).unwrap();
                                    Mode::Insert
                                }
                                'd' => {
                                    execute!(stdo, cursor::Hide).unwrap();
                                    Mode::Draw
                                }
                                'q' => {
                                    execute!(stdo, Clear(ClearType::All)).unwrap();
                                    Mode::Command
                                }
                                _ => *mode,
                            }
                        }
                        Mode::Insert => {
                            execute!(stdo, crossterm::style::Print(code),).unwrap();
                        }
                        Mode::Draw => {
                            *brush = code;
                        }
                    };
                }
                KeyCode::Left => {
                    execute!(stdo, cursor::MoveLeft(1)).unwrap();
                }
                KeyCode::Right => {
                    execute!(stdo, cursor::MoveRight(1)).unwrap();
                }
                KeyCode::Up => {
                    execute!(stdo, cursor::MoveUp(1)).unwrap();
                }
                KeyCode::Down => {
                    execute!(stdo, cursor::MoveDown(1)).unwrap();
                }
                //  | KeyCode::Up | KeyCode::Down
                KeyCode::Backspace => {
                    execute!(
                        stdo,
                        cursor::MoveLeft(1),
                        crossterm::style::Print(" "),
                        cursor::MoveLeft(1)
                    )
                    .unwrap();
                }
                _ => {}
            }
        }
        _ => {}
    };
    let (x, y) = position().unwrap_or_default();
    let (max_x, googa) = size().unwrap_or_default();
    execute!(stdo, cursor::MoveTo(0, googa)).unwrap();
    let mode_text = match *mode {
        Mode::Command => "COMMAND",
        Mode::Insert => "INSERT",
        Mode::Draw => "DRAW",
    };
    print!(
        "{mode_text} MODE, pos: ({x}, {y}), max_pos: ({max_x}, {googa}), brush: {}",
        brush.clone()
    );
    execute!(stdo, cursor::MoveTo(x, y)).unwrap();
    true
}

async fn event_handler() {
    let mut reader = EventStream::new();
    let mut brush = '*';
    let mut mode = Mode::Draw;

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();
        let mut stdo = stdout();

        select! {
            _ = delay => {  },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        match draw(event, &mut stdo, &mut brush, &mut mode) {
                            false => break,
                            true => {}
                        };
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

    event_handler().await;

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
