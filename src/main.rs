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

// use crate::command::command;
// use crate::content_brush::content_brush;
use crate::data::*;
use crate::modes::Mode;
// use crate::modes;
// use crate::eyedropper::eyedropper;
// use crate::insert::insert;
// use crate::pencil::pencil;

mod data;
mod modes;

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
            modes::command(event, stdout, state, &mut frame_state, &colors);
        }
        Mode::Insert => {
            modes::insert(event, stdout, state, &mut frame_state);
        }
        Mode::Pencil => {
            modes::pencil(event, stdout, state, &mut frame_state);
        }
        Mode::ContentBrush => {
            modes::content_brush(event, stdout, state, &mut frame_state);
        }
        Mode::Eyedropper => {
            modes::eyedropper(event, stdout, state, &mut frame_state);
        }
        Mode::Brush => {
            modes::brush(event, stdout, state, &mut frame_state);
        }
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
    let (max_x, max_y) = size().unwrap_or_default();
    queue!(
        stdout,
        cursor::MoveTo(0, max_y),
        SetForegroundColor(Color::Red)
    )
    .unwrap();
    let bar_color = match state.mode {
        Mode::Brush => Color::DarkGreen,
        Mode::Eyedropper => Color::DarkMagenta,
        Mode::Command => Color::DarkRed,
        Mode::Insert => Color::DarkCyan,
        Mode::Pencil => Color::DarkYellow,
        Mode::ContentBrush => Color::Green,
    };
    let mode_text = format!(" {} ", state.mode);
    let pos_text = format!(" repaints: {} | pos: ({x}, {y}) ", state.repaint_counter);
    let mid_pad = " ".repeat((max_x as usize - (mode_text.len() + pos_text.len() + 5)) / 2);
    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetBackgroundColor(bar_color),
        SetForegroundColor(Color::Black),
        crossterm::style::Print(mode_text),
        SetBackgroundColor(Color::DarkGrey),
        crossterm::style::Print(mid_pad.clone()),
        SetBackgroundColor(bar_color),
        crossterm::style::Print(" ["),
        SetForegroundColor(state.brush_color),
        crossterm::style::Print(state.brush),
        SetForegroundColor(Color::Black),
        crossterm::style::Print("] "),
        SetBackgroundColor(Color::DarkGrey),
        crossterm::style::Print(mid_pad),
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
        brush: '*',
        brush_color: Color::White,
        brush_mode: BrushMode::Subtract,
        pos: (0, 0),
        command: Command::None,
        drag_pos: (0, 0),
        virtual_display: Vec::with_capacity(termsize.0.into()),
        // history: vec![],
        // redo_layers: vec![],
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
