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
        MouseButton, MouseEventKind,
    },
    execute,
    style::{Color, SetForegroundColor},
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

#[derive(PartialEq, Clone, Copy, Debug)]
enum Command {
    EnterCommandMode,
    EnterInsertMode,
    EnterDrawMode,
    Clear,
    None,
}

fn draw(
    event: Event,
    stdout: &mut Stdout,
    brush: &mut char,
    mode: &mut Mode,
    command: &mut Command,
    drag_pos: &mut (u16, u16),
) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        if *mode == Mode::Command {
            return false;
        }
        execute!(stdout, cursor::Hide).unwrap();
        *mode = Mode::Command;
        *command = Command::EnterCommandMode;
    }

    match *mode {
        Mode::Command => match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Char(code) => {
                    *command = match code {
                        'i' => {
                            execute!(stdout, cursor::Show).unwrap();
                            *mode = Mode::Insert;
                            Command::EnterInsertMode
                        }
                        'd' => {
                            execute!(stdout, cursor::Hide).unwrap();
                            *mode = Mode::Draw;
                            Command::EnterDrawMode
                        }
                        'q' => {
                            execute!(stdout, Clear(ClearType::All)).unwrap();
                            Command::Clear
                        }
                        _ => *command,
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
            Event::Mouse(ev) => match ev.kind {
                MouseEventKind::Drag(MouseButton::Left)
                | MouseEventKind::Down(MouseButton::Left) => {
                    execute!(
                        stdout,
                        cursor::MoveTo(ev.column, ev.row),
                        crossterm::style::Print(*brush)
                    )
                    .unwrap();
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    *drag_pos = position().unwrap_or_default();
                }
                MouseEventKind::Drag(MouseButton::Right) => {
                    execute!(
                        stdout,
                        // thing
                        cursor::MoveTo(drag_pos.0, drag_pos.1),
                        // crossterm::style::Print(*brush)
                    )
                    .unwrap();
                    for _ in drag_pos.0..ev.column {
                        execute!(
                            stdout,
                            // thing
                            // cursor::MoveRight(1),
                            crossterm::style::Print(*brush),
                        )
                        .unwrap();
                    }
                    for _ in drag_pos.1..ev.row {
                        execute!(
                            stdout,
                            // thing
                            crossterm::style::Print(*brush),
                            cursor::MoveLeft(1),
                            cursor::MoveDown(1)
                        )
                        .unwrap();
                    }
                    execute!(
                        stdout,
                        // thing
                        cursor::MoveTo(drag_pos.0, drag_pos.1),
                        // crossterm::style::Print(*brush)
                    )
                    .unwrap();
                    for _ in drag_pos.1..ev.row {
                        execute!(
                            stdout,
                            // thing
                            crossterm::style::Print(*brush),
                            cursor::MoveLeft(1),
                            cursor::MoveDown(1)
                        )
                        .unwrap();
                    }
                    for _ in drag_pos.0..ev.column {
                        execute!(
                            stdout,
                            // thing
                            // cursor::MoveRight(1),
                            crossterm::style::Print(*brush),
                        )
                        .unwrap();
                    }
                    // let thing: Vec<(cursor::MoveLeft, crossterm::style::Print<char>)> = (drag_pos.0
                    //     ..position().unwrap_or_default().0)
                    //     .map(|f| (cursor::MoveLeft(1), crossterm::style::Print(*brush)))
                    //     .collect();

                    // thing.
                }
                _ => {}
            },
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
    execute!(
        stdout,
        cursor::MoveTo(0, googa),
        SetForegroundColor(Color::Red)
    )
    .unwrap();
    let mode_text = match *mode {
        Mode::Command => "COMMAND",
        Mode::Insert => "INSERT",
        Mode::Draw => "DRAW",
    };
    let info_display = format!("{mode_text} MODE, pos: ({x}, {y}), max_pos: ({max_x}, {googa}), drag pos: ({}, {}), brush: {}, last command: {:?}",
    drag_pos.0, drag_pos.1,
    brush.clone(), command);
    let pad = " ".repeat(max_x as usize - info_display.len());
    print!("{info_display}{pad}");
    execute!(
        stdout,
        cursor::MoveTo(x, y),
        SetForegroundColor(Color::White)
    )
    .unwrap();
    true
}

async fn event_handler() {
    let mut reader = EventStream::new();
    let mut brush = '*';
    let mut mode = Mode::Draw;
    let mut command = Command::None;
    let mut drag_pos: (u16, u16) = (0, 0);

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();
        let mut stdout = stdout();

        select! {
            _ = delay => {  },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        match draw(event, &mut stdout, &mut brush, &mut mode, &mut command, &mut drag_pos) {
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
