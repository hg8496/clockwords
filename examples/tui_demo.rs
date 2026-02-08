use clockwords::{
    ParserConfig, ResolvedTime, TimeExpressionScanner, Tz,
    lang::{self, LanguageParser},
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, Stdout};
use std::time::Duration;

/// Detect the system's IANA timezone and parse it into a `chrono_tz::Tz`.
/// Falls back to UTC if detection or parsing fails.
fn detect_local_timezone() -> Tz {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse::<Tz>().ok())
        .unwrap_or(Tz::UTC)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<ratatui::backend::CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut input = String::new();

    // Detect the local timezone for timezone-aware parsing
    let local_tz = detect_local_timezone();

    // Build scanner with local timezone
    let languages: Vec<Box<dyn LanguageParser>> = vec![
        Box::new(lang::en::English::new()),
        Box::new(lang::de::German::new()),
        Box::new(lang::fr::French::new()),
        Box::new(lang::es::Spanish::new()),
    ];
    let config = ParserConfig {
        timezone: local_tz,
        ..Default::default()
    };
    let scanner = TimeExpressionScanner::new(languages, config);

    loop {
        terminal.draw(|f| ui(f, &input, &scanner, local_tz))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, input: &str, scanner: &TimeExpressionScanner, tz: Tz) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input box
            Constraint::Min(1),    // Results
            Constraint::Length(1), // Help footer
        ])
        .split(f.area());

    // Input widget
    let title = format!(
        "Input — timezone: {} (Type a time expression like 'tomorrow', 'in 3 days')",
        tz
    );
    let input_paragraph = Paragraph::new(input)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(input_paragraph, chunks[0]);

    // Scan the input
    let now = chrono::Utc::now();
    let matches = scanner.scan(input, now);

    // Results widget
    let mut result_lines = vec![];
    if matches.is_empty() {
        result_lines.push(Line::from(Span::raw("No matches found.")));
    } else {
        for (i, m) in matches.iter().enumerate() {
            result_lines.push(Line::from(vec![
                Span::styled(format!("Match #{}: ", i + 1), Style::default().bold()),
                Span::raw(format!("'{}'", &input[m.span.as_range()])),
            ]));
            result_lines.push(Line::from(vec![
                Span::raw("  Span: "),
                Span::styled(format!("{:?}", m.span), Style::default().fg(Color::Cyan)),
            ]));
            result_lines.push(Line::from(vec![
                Span::raw("  Kind: "),
                Span::styled(format!("{:?}", m.kind), Style::default().fg(Color::Magenta)),
            ]));
            result_lines.push(Line::from(vec![
                Span::raw("  Confidence: "),
                Span::styled(
                    format!("{:?}", m.confidence),
                    Style::default().fg(Color::Green),
                ),
            ]));

            let resolved_str = match m.resolved {
                ResolvedTime::Point(dt) => {
                    let local = dt.with_timezone(&tz);
                    format!("{}", local)
                }
                ResolvedTime::Range { start, end } => {
                    let local_start = start.with_timezone(&tz);
                    let local_end = end.with_timezone(&tz);
                    format!("{} — {}", local_start, local_end)
                }
            };
            result_lines.push(Line::from(vec![
                Span::raw("  Resolved: "),
                Span::styled(resolved_str, Style::default().fg(Color::Blue)),
            ]));
            result_lines.push(Line::from(""));
        }
    }

    let results_paragraph = Paragraph::new(result_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Parsed Results"),
    );
    f.render_widget(results_paragraph, chunks[1]);

    // Help footer
    let help_text = Line::from("Press ESC to quit");
    let help_paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help_paragraph, chunks[2]);

    // Position cursor at the end of input
    f.set_cursor_position((chunks[0].x + 1 + input.len() as u16, chunks[0].y + 1));
}
