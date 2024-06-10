use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentScreen};

fn create_title_block(app: &App) -> Paragraph {
    let title_block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::horizontal(1))
        .style(Style::default());

    let title_text = match app.current_screen {
        CurrentScreen::Introduction => "Numbers Game",
        CurrentScreen::PickingNumbers => {
            if app.is_number_selection_complete() {
                "Hit (Enter) to start the challenge"
            } else {
                "Pick some numbers"
            }
        }
        CurrentScreen::Playing => "Solve the challenge",
        CurrentScreen::DisplayingResult => "How did you do?",
    };

    Paragraph::new(Text::styled(title_text, Style::default())).block(title_block)
}

fn create_selected_numbers_block(app: &App) -> Paragraph {
    let mut selected_numbers_text = app.selected_numbers.into_iter().fold(
        vec![Span::styled("Numbers: ", Style::default())],
        |mut accum, val| {
            if let Some(value) = val {
                accum.push(Span::styled(
                    format!("{value} "),
                    Style::default().fg(Color::Green),
                ));
            } else {
                accum.push(Span::styled("_ ", Style::default().fg(Color::Green)));
            };
            accum
        },
    );

    match app.current_screen {
        CurrentScreen::Introduction => {}
        CurrentScreen::PickingNumbers => {
            selected_numbers_text.push(Span::styled("    Target:", Style::default()));
            selected_numbers_text.push(Span::styled(" ???", Style::default().fg(Color::Green)));
        }
        CurrentScreen::DisplayingResult | CurrentScreen::Playing => {
            selected_numbers_text.push(Span::styled("   Target: ", Style::default()));
            selected_numbers_text.push(Span::styled(
                app.target.to_string(),
                Style::default().fg(Color::Green),
            ));
        }
    };

    Paragraph::new(Line::from(selected_numbers_text).centered())
        .block(Block::default().padding(Padding::top(1)))
}

fn create_objective(_app: &App) -> Paragraph {
    Paragraph::new(Span::styled(
        "Use your 6 (randomly picked) numbers with +, -, * and / operations to match the target number.",
        Style::default().fg(Color::Green),
    ))
    .wrap(Wrap { trim: true })
    .block(Block::default().padding(Padding::horizontal(2)).padding(Padding::top(1)))
}

fn create_instructions(_app: &App) -> Paragraph {
    Paragraph::new(vec![
        Line::from("  — You pick 6 numbers, from 4 available large numbers and 20 small ones."),
        Line::from(
            "  — Combine your numbers with arithemetic operations to match the random target.",
        ),
        Line::from("  — You don’t have to use all 6 numbers."),
        Line::from("  — Any division operations should result in a whole number."),
        Line::from(
            "  — If it’s not possible to reach the target exactly, get as close as you can.",
        ),
    ])
    .wrap(Wrap { trim: false })
}

fn create_large_number_selection(app: &App) -> Paragraph {
    let large_number_selection_text = app
        .available_large_numbers
        .into_iter()
        .map(|val| {
            if val.is_some() {
                Span::styled("** ", Style::default().fg(Color::Green))
            } else {
                Span::styled("XX ", Style::default().fg(Color::Red))
            }
        })
        .collect::<Vec<Span>>();

    Paragraph::new(vec![
        Line::from("Large numbers (]):"),
        Line::from(large_number_selection_text).centered(),
    ])
    .block(Block::default().padding(Padding::horizontal(1)))
}

fn create_small_number_line(numbers: &[Option<u32>]) -> Vec<Span> {
    numbers
        .iter()
        .copied()
        .map(|val| {
            if val.is_some() {
                Span::styled("* ", Style::default().fg(Color::Green))
            } else {
                Span::styled("X ", Style::default().fg(Color::Red))
            }
        })
        .collect::<Vec<Span>>()
}

fn create_small_number_selection(app: &App) -> Paragraph {
    Paragraph::new(vec![
        Line::from("Small numbers ([):"),
        Line::from(create_small_number_line(&app.available_small_numbers[..7])).centered(),
        Line::from(create_small_number_line(
            &app.available_small_numbers[7..14],
        ))
        .centered(),
        Line::from(create_small_number_line(&app.available_small_numbers[14..])).centered(),
    ])
    .block(Block::default().padding(Padding::horizontal(1)))
}

fn create_hint_footer(app: &App) -> Paragraph {
    let hint_text = match app.current_screen {
        CurrentScreen::Introduction => "Press (Enter) to skip",
        CurrentScreen::PickingNumbers => {
            if app.is_number_selection_complete() {
                "Press (Enter) to start"
            } else {
                "Pick 6 numbers [: small, ]: large"
            }
        }
        CurrentScreen::Playing => "Use ( + - / * ) to hit the target",
        CurrentScreen::DisplayingResult => "",
    };

    Paragraph::new(Line::from(hint_text)).block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1)),
    )
}

fn create_key_notes_footer(app: &App) -> Paragraph {
    let hint_text = match app.current_screen {
        CurrentScreen::Introduction | CurrentScreen::PickingNumbers => {
            "(q) to quit, (Enter) to start"
        }
        CurrentScreen::Playing => "(q) to quit, (Enter) to submit",
        CurrentScreen::DisplayingResult => "(q) to quit, (Enter) to play again",
    };
    let current_keys_hint = Span::styled(hint_text, Style::default().fg(Color::Yellow));

    Paragraph::new(Line::from(current_keys_hint)).block(
        Block::default()
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1)),
    )
}

fn create_solution_attempt_block(app: &App) -> Paragraph {
    let hint = Line::from(Span::styled(
        "Enter your solution here (using 0-9, +, -, *, / and ()):",
        Style::default(),
    ));
    let input_text = if app.value_input.is_empty() {
        Span::styled("    _", Style::default().add_modifier(Modifier::SLOW_BLINK))
    } else {
        Span::from(format!("    {}", &app.value_input))
    };

    let input_feedback = Line::from(vec![
        input_text,
        Span::styled(&app.feedback, Style::default().fg(Color::Green)),
    ]);
    Paragraph::new(vec![hint, Line::from(""), input_feedback])
}

fn create_result_block(app: &App) -> Paragraph {
    let solution_text = match app.check_solution() {
        Some(value) => match value {
            0 => String::from("You nailed it 🔨. You hit the target!"),
            1..=5 => format!("Awesome result 🏅 only {value} from the target!"),
            6 => format!("Great result 🥈 just {value} from the target!"),
            7..=10 => format!("Nice result 🥉 {value} from the target!"),
            _ => format!("You got within {value} of the target 🏹"),
        },
        None => String::from("Unlucky! You can always try again 🎲"),
    };
    Paragraph::new(Line::from(solution_text).centered())
}

pub fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let title = create_title_block(app);
    frame.render_widget(title, chunks[0]);

    match app.current_screen {
        CurrentScreen::PickingNumbers | CurrentScreen::Playing => {
            let selected_numbers = create_selected_numbers_block(app);
            frame.render_widget(selected_numbers, chunks[1]);
        }
        CurrentScreen::Introduction => {
            let objective = create_objective(app);
            frame.render_widget(objective, chunks[1]);
        }
        CurrentScreen::DisplayingResult => {}
    }

    match app.current_screen {
        CurrentScreen::Introduction => {
            let instructions = create_instructions(app);
            frame.render_widget(instructions, chunks[2]);
        }
        CurrentScreen::PickingNumbers => {
            let number_selection_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Length(4)])
                .split(chunks[2]);
            let large_number_selection = create_large_number_selection(app);
            frame.render_widget(large_number_selection, number_selection_chunks[0]);

            let small_number_list = create_small_number_selection(app);
            frame.render_widget(small_number_list, number_selection_chunks[1]);
        }
        CurrentScreen::Playing => {
            let solution_attempt = create_solution_attempt_block(app);
            frame.render_widget(solution_attempt, chunks[2]);
        }
        CurrentScreen::DisplayingResult => {
            let result = create_result_block(app);
            frame.render_widget(result, chunks[2]);
        }
    }

    let hint_footer = create_hint_footer(app);

    let key_notes_footer = create_key_notes_footer(app);

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);

    frame.render_widget(hint_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);
}

#[cfg(test)]
mod tests {

    use super::{create_title_block, App, CurrentScreen};
    use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

    #[test]
    fn create_title_displays_as_exected_in_introduction_view() {
        // arrange
        let app = App::new();
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 3));

        let title_block = create_title_block(&app);

        // act
        title_block.render(buf.area, &mut buf);

        // assert
        let expected = Buffer::with_lines(vec![
            "┌──────────────────────────────────────────────────────────────────────────────┐",
            "│ Numbers Game                                                                 │",
            "└──────────────────────────────────────────────────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn create_title_displays_as_exected_in_picking_numbers_view() {
        // arrange
        let mut app = App::new();
        app.current_screen = CurrentScreen::PickingNumbers;
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 3));

        let title_block = create_title_block(&app);

        // act
        title_block.render(buf.area, &mut buf);

        // assert
        let expected = Buffer::with_lines(vec![
            "┌──────────────────────────────────────────────────────────────────────────────┐",
            "│ Pick some numbers                                                            │",
            "└──────────────────────────────────────────────────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);

        // arrange
        let mut app = App::new();
        app.current_screen = CurrentScreen::PickingNumbers;
        app.pick_random_large_number();
        app.pick_random_large_number();
        app.pick_random_large_number();
        app.pick_random_small_number();
        app.pick_random_small_number();
        app.pick_random_small_number();
        let title_block = create_title_block(&app);

        // act
        title_block.render(buf.area, &mut buf);

        // assert
        let expected = Buffer::with_lines(vec![
            "┌──────────────────────────────────────────────────────────────────────────────┐",
            "│ Hit (Enter) to start the challenge                                           │",
            "└──────────────────────────────────────────────────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn create_title_displays_as_exected_in_playing_view() {
        // arrange
        let mut app = App::new();
        app.current_screen = CurrentScreen::Playing;
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 3));

        let title_block = create_title_block(&app);

        // act
        title_block.render(buf.area, &mut buf);

        // assert
        let expected = Buffer::with_lines(vec![
            "┌──────────────────────────────────────────────────────────────────────────────┐",
            "│ Solve the challenge                                                          │",
            "└──────────────────────────────────────────────────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }

    #[test]
    fn create_title_displays_as_exected_in_result_view() {
        // arrange
        let mut app = App::new();
        app.current_screen = CurrentScreen::DisplayingResult;
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 3));

        let title_block = create_title_block(&app);

        // act
        title_block.render(buf.area, &mut buf);

        // assert
        let expected = Buffer::with_lines(vec![
            "┌──────────────────────────────────────────────────────────────────────────────┐",
            "│ How did you do?                                                              │",
            "└──────────────────────────────────────────────────────────────────────────────┘",
        ]);
        assert_eq!(buf, expected);
    }
}
