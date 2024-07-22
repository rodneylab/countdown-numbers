#![warn(clippy::all, clippy::pedantic)]

mod app;
mod ui;

use std::{
    io::{self},
    time::{Duration, Instant},
};

use app::CurrentScreen;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use rodio::{OutputStream, Sink};
use ui::{audio::SoundEffects, Ui};

use crate::app::App;

fn play_feedback_sound_effect(
    solution_result: Option<u32>,
    sink: &Sink,
    sound_effects: &SoundEffects,
) {
    match solution_result {
        Some(0) => sink.append(sound_effects.perfect.clone()),
        Some(value) => {
            if value < 11 {
                sink.append(sound_effects.valid.clone());
            }
        }
        None => {}
    }
}

fn update_feedback(app: &mut App) -> Option<u32> {
    let check_solution_result = app.check_solution();
    match check_solution_result {
        Some(0) => app.feedback = String::from(" ✅"),
        Some(value) => app.feedback = format!(" 📏 {value}"),
        None => app.feedback.clear(),
    }
    check_solution_result
}

fn handle_picking_numbers(
    app: &mut App,
    sink: Option<&Sink>,
    sound_effects: &SoundEffects,
    key_code: KeyCode,
) {
    match key_code {
        KeyCode::Enter => {
            if app.is_number_selection_complete() {
                app.current_screen = CurrentScreen::Playing;
                if let Some(value) = sink {
                    value.append(sound_effects.start.clone());
                }
            }
        }
        KeyCode::Char(']') => {
            app.pick_random_large_number();
        }
        KeyCode::Char('[') => {
            app.pick_random_small_number();
        }
        _ => {}
    }
}

fn handle_playing(
    app: &mut App,
    sink: Option<&Sink>,
    sound_effects: &SoundEffects,
    key_code: KeyCode,
) {
    match key_code {
        KeyCode::Backspace => {
            if let Some(value) = app.value_input.pop() {
                if !value.is_ascii_whitespace() {
                    let result_value = update_feedback(app);
                    if let Some(value) = sink {
                        play_feedback_sound_effect(result_value, value, sound_effects);
                    }
                }
            }
        }
        KeyCode::Char(value) => {
            if "01234567890()+-*/ ".contains(value) {
                app.value_input.push(value);
                let result_value = update_feedback(app);
                if let Some(sink_value) = sink {
                    if !value.is_ascii_whitespace() {
                        play_feedback_sound_effect(result_value, sink_value, sound_effects);
                    }
                }
            }
        }
        KeyCode::Enter => {
            app.current_screen = CurrentScreen::DisplayingResult;
        }
        _ => {}
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16);
    let mut app_ui = Ui::new();

    // stream should not be dropped while sink is still needed
    let (_stream, stream_handle) = match OutputStream::try_default() {
        Ok((stream, stream_handle)) => (Some(stream), Some(stream_handle)),
        Err(error) => {
            eprintln!("Error creating getting default audio output stream: {error}");
            (None, None)
        }
    };
    let sink = if let Some(stream_handle_value) = stream_handle {
        match Sink::try_new(&stream_handle_value) {
            Ok(value) => Some(value),
            Err(error) => {
                eprintln!("Error creating sink: {error}");
                None
            }
        }
    } else {
        None
    };

    let sound_effects = SoundEffects::default();

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
                    CurrentScreen::PickingNumbers => {
                        handle_picking_numbers(app, sink.as_ref(), &sound_effects, key.code);
                    }
                    CurrentScreen::Playing => {
                        handle_playing(app, sink.as_ref(), &sound_effects, key.code);
                    }
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
            app_ui.on_tick(app, sink.as_ref());
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
