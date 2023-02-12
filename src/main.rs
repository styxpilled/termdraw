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
    terminal::{self, disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    Result,
};

use crate::command::command;
use crate::data::*;

mod command;
mod data;

const HELP: &str = r#"EventStream based on futures_util::Stream with tokio
 - Keyboard, mouse and terminal resize events enabled
 - Prints "." every second if there's no event
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

const LUMA_VALUES: [char; 92] = [
    ' ', '`', '.', '-', '\'', ':', '_', ',', '^', '=', ';', '>', '<', '+', '!', 'r', 'c', '*', '/',
    'z', '?', 's', 'L', 'T', 'v', ')', 'J', '7', '(', '|', 'F', 'i', '{', 'C', '}', 'f', 'I', '3',
    '1', 't', 'l', 'u', '[', 'n', 'e', 'o', 'Z', '5', 'Y', 'x', 'j', 'y', 'a', ']', '2', 'E', 'S',
    'w', 'q', 'k', 'P', '6', 'h', '9', 'd', '4', 'V', 'p', 'O', 'G', 'b', 'U', 'A', 'K', 'X', 'H',
    'm', '8', 'R', 'D', '#', '$', 'B', 'g', '0', 'M', 'N', 'W', 'Q', '%', '&', '@',
];

fn draw(event: Event, stdout: &mut Stdout, state: &mut State, colors: &Vec<Color>) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        if state.mode == Mode::Command {
            return false;
        }
        queue!(stdout, cursor::Hide).unwrap();
        state.mode = Mode::Command;
        state.command = Command::EnterCommandMode;
    }

    let mut frame_state = FrameState {
        need_repaint: false,
    };

    queue!(stdout, SetForegroundColor(state.brush_color)).unwrap();

    match state.mode {
        Mode::Command => {
            command(event, stdout, state, &mut frame_state, &colors);
        }
        Mode::Insert => {
            match event {
                Event::Key(ev) => match ev.code {
                    KeyCode::Char(code) => {
                        state.history.push(HistoryPage::Insert(TextLayer {
                            brush: code,
                            color: state.brush_color,
                        }));
                        state.redo_layers = vec![];
                        frame_state.need_repaint = true;
                        // queue!(stdout, crossterm::style::Print(code),).unwrap();
                    }
                    KeyCode::Left => {
                        state.history.push(HistoryPage::Cmd(Cmdnum::MoveLeft(1)));
                        state.redo_layers = vec![];
                        frame_state.need_repaint = true;
                    }
                    KeyCode::Right => {
                        state.history.push(HistoryPage::Cmd(Cmdnum::MoveRight(1)));
                        state.redo_layers = vec![];
                        frame_state.need_repaint = true;
                    }
                    KeyCode::Up => {
                        state.history.push(HistoryPage::Cmd(Cmdnum::MoveUp(1)));
                        state.redo_layers = vec![];
                        frame_state.need_repaint = true;
                    }
                    KeyCode::Down => {
                        state.history.push(HistoryPage::Cmd(Cmdnum::MoveDown(1)));
                        state.redo_layers = vec![];
                        frame_state.need_repaint = true;
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
        Mode::Pencil => match event {
            Event::Mouse(ev) => match ev.kind {
                MouseEventKind::Drag(MouseButton::Left)
                | MouseEventKind::Down(MouseButton::Left) => {
                    // queue!(
                    //     stdout,
                    //     cursor::MoveTo(ev.column, ev.row),
                    //     crossterm::style::Print(state.brush)
                    // )
                    // .unwrap();
                    state.history.push(HistoryPage::Pencil(Layer {
                        brush: state.brush,
                        brush_color: state.brush_color,
                        changed: true,
                        x: ev.column,
                        y: ev.row,
                    }));
                    state.virtual_display[usize::from(ev.column)][usize::from(ev.row)] = Layer {
                        brush: state.brush,
                        brush_color: state.brush_color,
                        changed: true,
                        x: ev.column,
                        y: ev.row,
                    };
                    state.redo_layers = vec![];
                    frame_state.need_repaint = true;
                }
                // MouseEventKind::Up()
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
        Mode::Brush => match event {
            Event::Mouse(ev) => match ev.kind {
                MouseEventKind::Drag(MouseButton::Left)
                | MouseEventKind::Down(MouseButton::Left) => {
                    let (x, y) = (ev.column, ev.row);
                    let mut average_luma = 0;
                    let (mx, my) = terminal::size().unwrap_or_default();
                    let mut divider = 0;
                    let col_range =
                        if x > 1 { x - 1 } else { x }..if x + 1 < mx { x + 1 } else { x } + 1;
                    let row_range =
                        if y > 1 { y - 1 } else { y }..if y + 1 < my { y + 1 } else { y } + 1;
                    for n in col_range {
                        for i in row_range.clone() {
                            divider += 1;
                            average_luma += LUMA_VALUES
                                .iter()
                                .position(|&val| {
                                    val == state.virtual_display[usize::from(n)][usize::from(i)]
                                        .brush
                                })
                                .unwrap_or(50);
                        }
                    }
                    average_luma = average_luma / divider;
                    state.virtual_display[usize::from(ev.column)][usize::from(ev.row)] = Layer {
                        brush: LUMA_VALUES[average_luma],
                        brush_color: state.brush_color,
                        changed: true,
                        x: ev.column,
                        y: ev.row,
                    };
                    state.redo_layers = vec![];
                    frame_state.need_repaint = true;
                }
                _ => {}
            },
            _ => {}
        },
        Mode::Eyedropper => match event {
            Event::Mouse(ev) => match ev.kind {
                MouseEventKind::Drag(MouseButton::Left)
                | MouseEventKind::Down(MouseButton::Left) => {
                    state.brush =
                        state.virtual_display[usize::from(ev.column)][usize::from(ev.row)].brush;
                    state.brush_color = state.virtual_display[usize::from(ev.column)]
                        [usize::from(ev.row)]
                    .brush_color;
                }
                _ => {}
            },
            _ => {}
        },
    }
    // if ev.modifiers == KeyModifiers::SHIFT {
    //     print!("{:?}", ev);
    // }
    let mut repainted = vec![];
    if frame_state.need_repaint {
        state.repaint_counter += 1;
        for (col_index, column) in state.virtual_display.iter().enumerate() {
            for (i, element) in column.iter().enumerate() {
                if element.changed {
                    queue!(
                        stdout,
                        cursor::MoveTo(element.x, element.y),
                        SetForegroundColor(element.brush_color),
                        crossterm::style::Print(element.brush)
                    )
                    .unwrap();
                    repainted.push((col_index, i));
                    // element.changed = false;
                }
            }
        }
        // for page in state.history.clone() {
        //     match page {
        //         HistoryPage::Pencil(page) => {
        //             queue!(
        //                 stdout,
        //                 cursor::MoveTo(page.x, page.y),
        //                 SetForegroundColor(page.brush_color),
        //                 crossterm::style::Print(page.brush)
        //             )
        //             .unwrap();
        //         }
        //         HistoryPage::Insert(page) => {
        //             queue!(
        //                 stdout,
        //                 SetForegroundColor(page.color),
        //                 crossterm::style::Print(page.brush)
        //             )
        //             .unwrap();
        //         }
        //         HistoryPage::Cmd(cmd) => match cmd {
        //             Cmdnum::MoveLeft(n) => queue!(stdout, cursor::MoveLeft(n)).unwrap(),
        //             Cmdnum::MoveRight(n) => queue!(stdout, cursor::MoveRight(n)).unwrap(),
        //             Cmdnum::MoveUp(n) => queue!(stdout, cursor::MoveUp(n)).unwrap(),
        //             Cmdnum::MoveDown(n) => queue!(stdout, cursor::MoveDown(n)).unwrap(),
        //             Cmdnum::MoveTo(x, y) => queue!(stdout, cursor::MoveTo(x, y)).unwrap(),
        //         },
        //     }
        // }
    }

    for r in repainted.iter() {
        state.virtual_display[r.0][r.1].changed = false;
    }

    stdout.flush().unwrap();

    match state.mode {
        Mode::Insert => queue!(stdout, cursor::Show).unwrap(),
        _ => {}
    }

    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        crossterm::style::Print(state.repaint_counter)
    )
    .unwrap();

    let (x, y) = position().unwrap_or_default();
    let (max_x, googa) = size().unwrap_or_default();
    queue!(
        stdout,
        cursor::MoveTo(0, googa),
        SetForegroundColor(Color::Red)
    )
    .unwrap();
    let mode_text = match state.mode {
        Mode::Eyedropper => "EYEDROPPER",
        Mode::Command => "COMMAND",
        Mode::Insert => "INSERT",
        Mode::Pencil => "PENCIL",
        Mode::Brush => "BRUSH",
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
    let termsize = terminal::size().unwrap_or_default();

    let mut state = State {
        repaint_counter: 0,
        mode: Mode::Command,
        brush: '*',
        brush_color: Color::White,
        command: Command::None,
        drag_pos: (0, 0),
        virtual_display: Vec::with_capacity(termsize.0.into()),
        history: vec![],
        redo_layers: vec![],
    };

    for n in 0..termsize.0 {
        let mut nested = Vec::with_capacity(termsize.1.into());
        for i in 0..termsize.1 {
            nested.push(Layer {
                brush: ' ',
                brush_color: Color::White,
                changed: false,
                x: n,
                y: i,
            })
        }
        state.virtual_display.push(nested);
    }

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

    let mut stdoout_temp = stdout();
    draw(
        crossterm::event::Event::FocusGained,
        &mut stdoout_temp,
        &mut state,
        &colors,
    );

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
        cursor::EnableBlinking,
        cursor::SetCursorShape(cursor::CursorShape::Line),
        cursor::Hide
    )?;

    event_handler().await;

    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()
}
