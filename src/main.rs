use std::{io, time::Duration};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use my_app::{ResourceType, Tile, generate_map};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

fn main() {
    if let Err(err) = run() {
        eprintln!("Erreur d'execution: {}", err);
    }
}

fn run() -> io::Result<()> {
    let map = generate_map(42, 40, 20)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app_result = run_app(&mut terminal, &map);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    app_result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, map: &my_app::Map) -> io::Result<()> {
    clear_pending_input_events()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let block = Block::default().title("Robot Collectors - Appuie sur une touche pour quitter").borders(Borders::ALL);
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let map_lines = map_to_lines(map, inner);
            let paragraph = Paragraph::new(map_lines);
            frame.render_widget(paragraph, inner);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(_) => return Ok(()),
                _ => continue,
            }
        }
    }
}

fn clear_pending_input_events() -> io::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }
    Ok(())
}

fn map_to_lines(map: &my_app::Map, viewport: Rect) -> Vec<Line<'static>> {
    let max_rows = map.height.min(viewport.height as usize);
    let max_cols = map.width.min(viewport.width as usize);
    let mut lines = Vec::with_capacity(max_rows + 1);

    for y in 0..max_rows {
        let mut spans = Vec::with_capacity(max_cols);
        for x in 0..max_cols {
            let (symbol, style) = match map.get(x, y) {
                Tile::Obstacle => ("O", Style::default().fg(Color::Cyan)),
                Tile::Base => ("#", Style::default().fg(Color::LightGreen)),
                Tile::Resource {
                    kind: ResourceType::Energy,
                    ..
                } => ("E", Style::default().fg(Color::Green)),
                Tile::Resource {
                    kind: ResourceType::Crystal,
                    ..
                } => ("C", Style::default().fg(Color::LightMagenta)),
                Tile::Empty => (".", Style::default().fg(Color::DarkGray)),
            };
            spans.push(Span::styled(symbol.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    lines
}
