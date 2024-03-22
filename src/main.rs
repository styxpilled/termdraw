use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor,
    cursor::position,
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute, queue,
    style::{Attribute, Color, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    Result,
};

use crate::data::*;
use crate::modes::Mode;

mod data;
mod handlers;
mod modes;

const LUMA_VALUES: [char; 92] = [
    ' ', '`', '.', '-', '\'', ':', '_', ',', '^', '=', ';', '>', '<', '+', '!', 'r', 'c', '*', '/',
    'z', '?', 's', 'L', 'T', 'v', ')', 'J', '7', '(', '|', 'F', 'i', '{', 'C', '}', 'f', 'I', '3',
    '1', 't', 'l', 'u', '[', 'n', 'e', 'o', 'Z', '5', 'Y', 'x', 'j', 'y', 'a', ']', '2', 'E', 'S',
    'w', 'q', 'k', 'P', '6', 'h', '9', 'd', '4', 'V', 'p', 'O', 'G', 'b', 'U', 'A', 'K', 'X', 'H',
    'm', '8', 'R', 'D', '#', '$', 'B', 'g', '0', 'M', 'N', 'W', 'Q', '%', '&', '@',
];

fn draw(event: Event, stdout: &mut Stdout, state: &mut State, colors: &Vec<Color>) -> bool {
    if event == Event::Key(KeyCode::Esc.into()) {
        match state.mode {
            Mode::Command => {
                return false;
            }
            _ => {}
        }
        queue!(stdout, cursor::Hide).unwrap();
        state.mode = Mode::Command;
        state.command = Command::EnterCommandMode;
    }

    queue!(stdout, SetForegroundColor(state.color)).unwrap();
    match state.mode {
        Mode::Command => {
            modes::command(event, stdout, state, &colors);
        }
        Mode::Insert => {
            modes::insert(event, stdout, state);
        }
        Mode::Pencil(_) => {
            modes::pencil(event, stdout, state);
        }
        Mode::ContentBrush => {
            modes::content_brush(event, stdout, state);
        }
        Mode::Eyedropper => {
            modes::eyedropper(event, stdout, state);
        }
        Mode::Brush(_) => {
            modes::brush(event, stdout, state);
        }
    }
    // if ev.modifiers == KeyModifiers::SHIFT {
    //     print!("{:?}", ev);
    // }
    let mut repainted = vec![];
    if state.virtual_display.need_repaint {
        state.repaint_counter += 1;
        for (col, column) in state.virtual_display.vd.iter().enumerate() {
            for (row, element) in column.iter().enumerate() {
                if element.changed {
                    queue!(
                        stdout,
                        cursor::MoveTo(col as u16, row as u16),
                        SetForegroundColor(element.brush_color),
                        crossterm::style::Print(element.brush)
                    )
                    .unwrap();
                    repainted.push((col, row));
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
        state.virtual_display.vd[r.0][r.1].changed = false;
    }

    stdout.flush().unwrap();

    match state.mode {
        Mode::Insert => queue!(stdout, cursor::Show).unwrap(),
        _ => {}
    }

    let (x, y) = position().unwrap_or_default();
    let (max_x, max_y) = size().unwrap_or_default();
    queue!(stdout, cursor::MoveTo(0, max_y)).unwrap();
    let bar_color = state.mode.get_color();
    let mode_text = format!(" {} ", state.mode);
    let pos_text = format!(" repaints: {} | pos: ({x}, {y}) ", state.repaint_counter);
    let left_pad = " ".repeat((max_x as usize / 2) - (mode_text.len() + 5));
    let right_pad =
        " ".repeat((max_x as usize / 2) - pos_text.len() + if max_x % 2 == 0 { 0 } else { 1 });
    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetBackgroundColor(bar_color),
        SetForegroundColor(Color::Black),
        crossterm::style::Print(mode_text),
        SetBackgroundColor(Color::DarkGrey),
        crossterm::style::Print(left_pad),
        SetBackgroundColor(bar_color),
        crossterm::style::Print(" ["),
        SetForegroundColor(state.color),
        crossterm::style::Print("T"),
        SetForegroundColor(Color::Black),
        crossterm::style::Print("] "),
        SetBackgroundColor(Color::DarkGrey),
        crossterm::style::Print(right_pad),
        SetBackgroundColor(bar_color),
        crossterm::style::Print(pos_text),
    )
    .unwrap();
    // let info_display = (format!("{mode_text} MODE, pos: ({x}, {y}), max_pos: ({max_x}, {googa}), drag pos: ({}, {}), brush: ",
    // state.drag_pos.0, state.drag_pos.1),
    // format!(", last command: {:?}", state.command));
    // let pad = " ".repeat(max_x as usize - (info_display.0.len() + info_display.1.len()));
    // print!("{}", info_display.0);
    // queue!(stdout, SetForegroundColor(state.brush_color)).unwrap();
    // print!("{}", state.brush);
    // queue!(stdout, SetForegroundColor(Color::Red)).unwrap();
    // print!("{pad}");
    queue!(
        stdout,
        cursor::MoveTo(x, y),
        SetForegroundColor(Color::White),
        SetBackgroundColor(Color::Reset),
        SetAttribute(Attribute::Reset)
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
        color: Color::White,
        pos: (0, 0),
        command: Command::None,
        drag_pos: (0, 0),
        virtual_display: Display::new(termsize.0, termsize.1),
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
