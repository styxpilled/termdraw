use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
    vec,
};

use commands::process_shortcuts;
use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor::{self, position},
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
    Result,
};
use handlers::{get_click_pos, handle_click};

use crate::data::*;
use crate::modes::Mode;

mod commands;
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

fn draw(event: Event, stdout: &mut Stdout, state: &mut State) -> bool {
    // Handle qutting
    if event == Event::Key(KeyCode::Esc.into()) {
        match state.mode {
            Mode::Command => {
                return false;
            }
            _ => {}
        }
        queue!(stdout, cursor::Hide).unwrap();
        state.mode = Mode::Command;
        state.command = Command::Enter(state.mode.clone());
    }

    // Get current x, y, size
    let (x, y) = position().unwrap_or_default();
    let (max_width, max_height) = size().unwrap_or_default();
    // Skip gets used to skip processing of an event if it's already been processed
    let mut skip = false;

    // TODO: process resizing
    // match &event {
    //     Event::Resize(new_width, new_height) => {}
    //     _ => {}
    // }

    // TODO: custom colors
    handle_click(&event, |ev, col, row| {
        // Color palette
        // if row + 1 == max_height {
        //     let offset = format!(" {} ", state.mode).len() as u16 + 1;
        //     if col > offset && col < offset + 16 {
        //         state.color = state.colors[(col - offset) as usize];
        //     }
        //     skip = true;
        // }
        // Process ALT-eyedropper

        if ev.modifiers.contains(KeyModifiers::ALT) {
            state.eyedrop(col, row);
            skip = true;
        }
    });

    queue!(stdout, SetForegroundColor(state.color)).unwrap();
    // Process the event onto the virtual display
    if !skip {
        state.run(&event, stdout);
    }

    // process shortcuts
    process_shortcuts(&event, stdout, state);

    // Draw all changes on the canvas if they need changes
    if state.virtual_display.need_repaint {
        state.repaint_counter += 1;
        // We loop over everything instead of using some sort of changed cache because that sounds complicated and we're not looping much. Computers are fast.
        for (col_pos, column) in state.virtual_display.vd.iter_mut().enumerate() {
            for (row_pos, element) in column.iter_mut().enumerate() {
                if !element.changed {
                    continue;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(col_pos as u16, row_pos as u16),
                    SetForegroundColor(element.brush_color),
                    crossterm::style::Print(element.brush)
                )
                .unwrap();
                element.changed = false;
            }
        }
    }

    // Flush all of the canvas re-drawing before drawing the bottom UI
    // Not necessary
    stdout.flush().unwrap();

    // TODO: only redraw ui on changes
    if let Mode::Insert = state.mode {
        queue!(stdout, cursor::Show).unwrap();
    }

    queue!(stdout, cursor::MoveTo(0, max_height)).unwrap();
    let bar_color = state.mode.get_color();

    let mut ui = UI {
        elements: vec![],
        stdout: stdout,
        pos: get_click_pos(&event),
        max: (max_width as usize, max_height as usize),
        bg_color: bar_color.clone(),
        pad: state.pad,
        // state: state,
        offset: 0,
    };
    // ui.elements = vec![];

    if ui.add(|| Widget::new("CYAN", state.color)).clicked() {
        state.color = Color::Cyan;
    };

    ui.push(Widget::new(&state.mode, Color::Black));

    ui.push(Widget::new("T", state.color));

    if ui.push(Widget::new("RED", state.color)).clicked() {
        state.color = Color::Red;
    }

    state.pad = ui.render((max_width.into(), max_height.into()), bar_color);

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
        colors: generate_colors(),
        pad: 0,
        drag_pos: (0, 0),
        // ui: UI { elements: vec![] },
        virtual_display: Canvas::new(termsize.0, termsize.1),
    };

    let mut stdoout_temp = stdout();
    draw(
        crossterm::event::Event::FocusGained,
        &mut stdoout_temp,
        &mut state,
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
                        if let false = draw(event, &mut stdout, &mut state) {
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

fn generate_colors() -> Vec<Color> {
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
