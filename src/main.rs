use std::{
    io,
    sync::{Arc, RwLock},
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use my_app::{ResourceType, SimState, Tile, generate_map, start_simulation, RobotKind};
use rand::random;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

fn main() {
    if let Err(err) = run() {
        eprintln!("Erreur d'execution: {}", err);
    }
}

fn run() -> io::Result<()> {
    let map = generate_map(random(), 40, 20)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let sim = start_simulation(map);
    let app_result = run_app(&mut terminal, sim);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    app_result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    sim: Arc<RwLock<SimState>>,
) -> io::Result<()> {
    clear_pending_input_events()?;

    loop {
        let state = sim.read().unwrap().clone();

        terminal.draw(|frame| {
            let area = frame.area();
            let outer = Block::default()
                .title(" Robot Collectors ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan));
            frame.render_widget(&outer, area);

            let inner = outer.inner(area);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
                .split(inner);

            let map_block = Block::default()
                .title(" Carte ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));
            frame.render_widget(&map_block, chunks[0]);
            let map_inner = map_block.inner(chunks[0]);

            let map_lines = map_to_lines(&state, map_inner);
            let paragraph = Paragraph::new(map_lines)
                .block(Block::default())
                .wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(paragraph, map_inner);

            let stats_block = Block::default()
                .title(" Statistiques ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightMagenta));
            frame.render_widget(&stats_block, chunks[1]);
            let stats_inner = stats_block.inner(chunks[1]);
            let stats_para = Paragraph::new(build_stats_panel(&state));
            frame.render_widget(stats_para, stats_inner);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                return Ok(());
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

fn map_to_lines(state: &SimState, viewport: Rect) -> Vec<Line<'static>> {
    let max_rows = state.map_height.min(viewport.height as usize);
    let max_cols = state.map_width.min(viewport.width as usize);
    let mut lines = Vec::with_capacity(max_rows);

    for y in 0..max_rows {
        let mut spans = Vec::with_capacity(max_cols);
        for x in 0..max_cols {
            let robot = state.robots.iter().find(|r| r.x == x && r.y == y);
            let (symbol, style) = if let Some(robot) = robot {
                match robot.kind {
                    RobotKind::Scout => (
                        "x",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    RobotKind::Collector => (
                        "o",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                }
            } else {
                let idx = y * state.map_width + x;
                match state.map_tiles[idx] {
                    Tile::Obstacle => ("O", Style::default().fg(Color::LightCyan)),
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
                }
            };
            spans.push(Span::styled(symbol.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    lines
}

fn build_stats_panel(state: &SimState) -> Vec<Line<'static>> {
    let scout_count = state
        .robots
        .iter()
        .filter(|r| matches!(r.kind, RobotKind::Scout))
        .count();
    let collector_count = state
        .robots
        .iter()
        .filter(|r| matches!(r.kind, RobotKind::Collector))
        .count();
    let total = state.total_energy + state.total_crystal;

    vec![
        Line::from(vec![Span::styled(
            format!("Énergie récoltée : {}", state.total_energy),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("Cristaux récoltés : {}", state.total_crystal),
            Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("Total récolté : {}", total),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Éclaireurs : {}", scout_count),
            Style::default().fg(Color::Red),
        )]),
        Line::from(vec![Span::styled(
            format!("Collecteurs : {}", collector_count),
            Style::default().fg(Color::LightMagenta),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("Ressources connues : {}", state.known_resources.len()),
            Style::default().fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Légende :",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(vec![Span::styled(
            "E énergie | C cristaux | # base",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(vec![Span::styled(
            "x éclaireur | o collecteur",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "q / Esc : quitter",
            Style::default().fg(Color::DarkGray),
        )]),
    ]
}
