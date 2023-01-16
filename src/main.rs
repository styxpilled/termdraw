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
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, MouseButton,
        MouseEventKind,
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

struct State {
    mode: Mode,
    brush: char,
    command: Command,
    drag_pos: (u16, u16),
}

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

fn draw(event: Event, stdout: &mut Stdout, state: &mut State) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        if state.mode == Mode::Command {
            return false;
        }
        execute!(stdout, cursor::Hide).unwrap();
        state.mode = Mode::Command;
        state.command = Command::EnterCommandMode;
    }

    match state.mode {
        Mode::Command => match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Char(code) => {
                    state.command = match code {
                        'i' => {
                            execute!(stdout, cursor::Show).unwrap();
                            state.mode = Mode::Insert;
                            Command::EnterInsertMode
                        }
                        'd' => {
                            execute!(stdout, cursor::Hide).unwrap();
                            state.mode = Mode::Draw;
                            Command::EnterDrawMode
                        }
                        'q' => {
                            execute!(stdout, Clear(ClearType::All)).unwrap();
                            Command::Clear
                        }
                        _ => state.command,
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
                        crossterm::style::Print(state.brush)
                    )
                    .unwrap();
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    state.drag_pos = position().unwrap_or_default();
                }
                MouseEventKind::Drag(MouseButton::Right) => {
                    execute!(stdout, cursor::MoveTo(state.drag_pos.0, state.drag_pos.1),).unwrap();
                    for _ in state.drag_pos.0..ev.column {
                        execute!(stdout, crossterm::style::Print(state.brush),).unwrap();
                    }
                    for _ in state.drag_pos.1..ev.row {
                        execute!(
                            stdout,
                            crossterm::style::Print(state.brush),
                            cursor::MoveLeft(1),
                            cursor::MoveDown(1)
                        )
                        .unwrap();
                    }
                    execute!(stdout, cursor::MoveTo(state.drag_pos.0, state.drag_pos.1),).unwrap();
                    for _ in state.drag_pos.1..ev.row {
                        execute!(
                            stdout,
                            crossterm::style::Print(state.brush),
                            cursor::MoveLeft(1),
                            cursor::MoveDown(1)
                        )
                        .unwrap();
                    }
                    for _ in state.drag_pos.0..ev.column {
                        execute!(stdout, crossterm::style::Print(state.brush),).unwrap();
                    }
                }
                _ => {}
            },
            Event::Key(ev) => match ev.code {
                KeyCode::Char(code) => {
                    state.brush = code;
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
    let mode_text = match state.mode {
        Mode::Command => "COMMAND",
        Mode::Insert => "INSERT",
        Mode::Draw => "DRAW",
    };
    let info_display = format!("{mode_text} MODE, pos: ({x}, {y}), max_pos: ({max_x}, {googa}), drag pos: ({}, {}), brush: {}, last command: {:?}",
    state.drag_pos.0, state.drag_pos.1,
    state.brush, state.command);
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
    // let mut brush_color = Color::White;
    let mut state = State {
        mode: Mode::Command,
        brush: '*',
        command: Command::None,
        drag_pos: (0, 0),
    };

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();
        let mut stdout = stdout();

        select! {
            _ = delay => {  },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        match draw(event, &mut stdout, &mut state) {
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
