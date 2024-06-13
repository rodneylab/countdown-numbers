#![warn(clippy::all, clippy::pedantic)]

mod app;
mod ui;

use std::{
    io,
    time::{Duration, Instant},
};

use app::CurrentScreen;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use ui::Ui;

use crate::app::App;

fn update_feedback(app: &mut App) {
    match app.check_solution() {
        Some(0) => app.feedback = String::from(" ✅"),
        Some(value) => app.feedback = format!(" 📏 {value}"),
        None => app.feedback.clear(),
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16);
    let mut app_ui = Ui::new();
    loop {
        terminal.draw(|frame| app_ui.ui(frame, app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    continue;
                }
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                    return Ok(());
                }

                match app.current_screen {
                    CurrentScreen::Introduction => {
                        if key.code == KeyCode::Enter {
                            app.current_screen = CurrentScreen::PickingNumbers;
                        }
                    }
                    CurrentScreen::PickingNumbers => match key.code {
                        KeyCode::Enter => {
                            if app.is_number_selection_complete() {
                                app.current_screen = CurrentScreen::Playing;
                            }
                        }
                        KeyCode::Char(']') => {
                            app.pick_random_large_number();
                        }
                        KeyCode::Char('[') => {
                            app.pick_random_small_number();
                        }
                        _ => {}
                    },
                    CurrentScreen::Playing => match key.code {
                        KeyCode::Backspace => {
                            if let Some(value) = app.value_input.pop() {
                                if !value.is_ascii_whitespace() {
                                    update_feedback(app);
                                }
                            }
                        }
                        KeyCode::Char(value) => {
                            if "01234567890()+-*/ ".contains(value) {
                                app.value_input.push(value);
                                update_feedback(app);
                            }
                        }
                        KeyCode::Enter => {
                            app.current_screen = CurrentScreen::DisplayingResult;
                        }
                        _ => {}
                    },
                    CurrentScreen::DisplayingResult => {
                        // User requests replay
                        if key.code == KeyCode::Enter {
                            *app = app::App::new();
                            app.current_screen = CurrentScreen::PickingNumbers;
                            app_ui = Ui::new();
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app_ui.on_tick(app);
            last_tick = Instant::now();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    let _ = execute!(stderr, EnterAlternateScreen, EnableMouseCapture);

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let _result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
