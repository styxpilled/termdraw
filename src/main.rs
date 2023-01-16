use std::{
    io::{stdout, Stdout, Write},
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
    execute, queue,
    style::{Color, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    ExecutableCommand, Result,
};
const HELP: &str = r#"EventStream based on futures_util::Stream with tokio
 - Keyboard, mouse and terminal resize events enabled
 - Prints "." every second if there's no event
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

#[derive(Copy, Clone)]
struct Layer {
    brush: char,
    brush_color: Color,
    x: u16,
    y: u16,
}

struct State {
    mode: Mode,
    brush: char,
    brush_color: Color,
    command: Command,
    drag_pos: (u16, u16),
    layers: Vec<Layer>,
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

fn draw(event: Event, stdout: &mut Stdout, state: &mut State, colors: &Vec<Color>) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        if state.mode == Mode::Command {
            return false;
        }
        queue!(stdout, cursor::Hide).unwrap();
        state.mode = Mode::Command;
        state.command = Command::EnterCommandMode;
    }

    let mut need_repaint = false;

    queue!(stdout, SetForegroundColor(state.brush_color)).unwrap();

    match state.mode {
        Mode::Command => match event {
            Event::Key(ev) => match ev.code {
                KeyCode::Char(code) => {
                    state.command = match code {
                        'i' => {
                            queue!(stdout, cursor::Show).unwrap();
                            state.mode = Mode::Insert;
                            Command::EnterInsertMode
                        }
                        'd' => {
                            queue!(stdout, cursor::Hide).unwrap();
                            state.mode = Mode::Draw;
                            Command::EnterDrawMode
                        }
                        'q' => {
                            queue!(stdout, Clear(ClearType::All)).unwrap();
                            Command::Clear
                        }
                        'f' => {
                            let n = colors
                                .iter()
                                .position(|n| n == &state.brush_color)
                                .unwrap_or_default();
                            let index = if n + 1 < colors.len() { n + 1 } else { 0 };
                            state.brush_color = colors[index];
                            Command::None
                        }
                        'u' => {
                            queue!(stdout, Clear(ClearType::All)).unwrap();
                            state.layers.pop();
                            need_repaint = true;
                            Command::None
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
                        queue!(stdout, crossterm::style::Print(code),).unwrap();
                    }
                    KeyCode::Left => {
                        queue!(stdout, cursor::MoveLeft(1)).unwrap();
                    }
                    KeyCode::Right => {
                        queue!(stdout, cursor::MoveRight(1)).unwrap();
                    }
                    KeyCode::Up => {
                        queue!(stdout, cursor::MoveUp(1)).unwrap();
                    }
                    KeyCode::Down => {
                        queue!(stdout, cursor::MoveDown(1)).unwrap();
                    }
                    KeyCode::Backspace => {
                        queue!(
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
                    // queue!(
                    //     stdout,
                    //     cursor::MoveTo(ev.column, ev.row),
                    //     crossterm::style::Print(state.brush)
                    // )
                    // .unwrap();
                    state.layers.push(Layer {
                        brush: state.brush,
                        brush_color: state.brush_color,
                        x: ev.column,
                        y: ev.row,
                    });
                    need_repaint = true;
                }
                MouseEventKind::Down(MouseButton::Right) => {
                    state.drag_pos = position().unwrap_or_default();
                }
                MouseEventKind::Drag(MouseButton::Right) => {
                    queue!(stdout, cursor::MoveTo(state.drag_pos.0, state.drag_pos.1),).unwrap();
                    for _ in state.drag_pos.0..ev.column {
                        queue!(stdout, crossterm::style::Print(state.brush),).unwrap();
                    }
                    for _ in state.drag_pos.1..ev.row {
                        queue!(
                            stdout,
                            crossterm::style::Print(state.brush),
                            cursor::MoveLeft(1),
                            cursor::MoveDown(1)
                        )
                        .unwrap();
                    }
                    queue!(stdout, cursor::MoveTo(state.drag_pos.0, state.drag_pos.1),).unwrap();
                    for _ in state.drag_pos.1..ev.row {
                        queue!(
                            stdout,
                            crossterm::style::Print(state.brush),
                            cursor::MoveLeft(1),
                            cursor::MoveDown(1)
                        )
                        .unwrap();
                    }
                    for _ in state.drag_pos.0..ev.column {
                        queue!(stdout, crossterm::style::Print(state.brush),).unwrap();
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
    if need_repaint {
        for layer in state.layers.clone() {
            queue!(
                stdout,
                cursor::MoveTo(layer.x, layer.y),
                crossterm::style::Print(layer.brush)
            )
            .unwrap();
        }
    }

    let (x, y) = position().unwrap_or_default();
    let (max_x, googa) = size().unwrap_or_default();
    queue!(
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
    let info_display = (format!("{mode_text} MODE, pos: ({x}, {y}), max_pos: ({max_x}, {googa}), drag pos: ({}, {}), brush: ",
    state.drag_pos.0, state.drag_pos.1),
    format!(", last command: {:?}", state.command));
    let pad = " ".repeat(max_x as usize - (info_display.0.len() + info_display.1.len()));
    print!("{}", info_display.0);
    queue!(stdout, SetForegroundColor(state.brush_color)).unwrap();
    print!("{}", state.brush);
    queue!(stdout, SetForegroundColor(Color::Red)).unwrap();
    print!("{pad}");
    queue!(
        stdout,
        cursor::MoveTo(x, y),
        SetForegroundColor(Color::White)
    )
    .unwrap();
    stdout.flush().unwrap();
    true
}

async fn event_handler() {
    let mut reader = EventStream::new();
    // let mut brush_color = Color::White;
    let mut state = State {
        mode: Mode::Command,
        brush: '*',
        brush_color: Color::White,
        command: Command::None,
        drag_pos: (0, 0),
        layers: vec![],
    };

    let colors: Vec<Color> = {
        use Color::*;
        vec![
            // Reset,
            White,
            Grey,
            Black,
            DarkGrey,
            Red,
            DarkRed,
            Green,
            DarkGreen,
            Yellow,
            DarkYellow,
            Blue,
            DarkBlue,
            Magenta,
            DarkMagenta,
            Cyan,
            DarkCyan,
        ]
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
                        match draw(event, &mut stdout, &mut state, &colors) {
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
