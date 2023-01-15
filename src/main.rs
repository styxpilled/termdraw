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

fn draw(event: Event, stdout: &mut Stdout, brush: &mut char, mode: &mut Mode) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        if *mode == Mode::Command {
            return false;
        }
        execute!(stdout, cursor::Hide).unwrap();
        *mode = Mode::Command;
    }

    match *mode {
        Mode::Command => match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Char(code) => {
                    *mode = match code {
                        'i' => {
                            execute!(stdout, cursor::Show).unwrap();
                            Mode::Insert
                        }
                        'd' => {
                            execute!(stdout, cursor::Hide).unwrap();
                            Mode::Draw
                        }
                        'q' => {
                            execute!(stdout, Clear(ClearType::All)).unwrap();
                            Mode::Command
                        }
                        _ => *mode,
                    }
                }
                _ => {}
            },
            _ => {}
        },
        Mode::Insert => {
            match event {
                Event::Key(ev) => match ev.code {
                    KeyCode::Char(code) => {
                        execute!(stdout, crossterm::style::Print(code),).unwrap();
                    }
                    KeyCode::Left => {
                        execute!(stdout, cursor::MoveLeft(1)).unwrap();
                    }
                    KeyCode::Right => {
                        execute!(stdout, cursor::MoveRight(1)).unwrap();
                    }
                    KeyCode::Up => {
                        execute!(stdout, cursor::MoveUp(1)).unwrap();
                    }
                    KeyCode::Down => {
                        execute!(stdout, cursor::MoveDown(1)).unwrap();
                    }
                    KeyCode::Backspace => {
                        execute!(
                            stdout,
                            cursor::MoveLeft(1),
                            crossterm::style::Print(" "),
                            cursor::MoveLeft(1)
                        )
                        .unwrap();
                    }
                    _ => {}
                },
                _ => {}
            };
        }
        Mode::Draw => match event {
            Event::Mouse(ev) => {
                if *mode == Mode::Draw {
                    match ev.kind {
                        MouseEventKind::Drag(_) | MouseEventKind::Down(_) => {
                            execute!(
                                stdout,
                                cursor::MoveTo(ev.column, ev.row),
                                crossterm::style::Print(*brush)
                            )
                            .unwrap();
                        }
                        _ => {}
                    }
                }
            }
            Event::Key(ev) => match ev.code {
                KeyCode::Char(code) => {
                    *brush = code;
                }
                _ => {}
            },
            _ => {}
        },
    }
    // if ev.modifiers == KeyModifiers::SHIFT {
    //     print!("{:?}", ev);
    // }

    let (x, y) = position().unwrap_or_default();
    let (max_x, googa) = size().unwrap_or_default();
    execute!(stdout, cursor::MoveTo(0, googa)).unwrap();
    let mode_text = match *mode {
        Mode::Command => "COMMAND",
        Mode::Insert => "INSERT",
        Mode::Draw => "DRAW",
    };
    print!(
        "{mode_text} MODE, pos: ({x}, {y}), max_pos: ({max_x}, {googa}), brush: {}",
        brush.clone()
    );
    execute!(stdout, cursor::MoveTo(x, y)).unwrap();
    true
}

async fn event_handler() {
    let mut reader = EventStream::new();
    let mut brush = '*';
    let mut mode = Mode::Draw;

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();
        let mut stdout = stdout();

        select! {
            _ = delay => {  },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        match draw(event, &mut stdout, &mut brush, &mut mode) {
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
