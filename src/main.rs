use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use my_app::{Map, ResourceType, Tile, generate_map};
use rand::{prelude::SliceRandom, thread_rng};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Clone, Copy, Debug)]
enum RobotKind {
    Scout,
    Collector,
}

#[derive(Clone, Copy, Debug)]
struct Robot {
    kind: RobotKind,
    x: usize,
    y: usize,
}

impl Robot {
    fn icon(self) -> &'static str {
        match self.kind {
            RobotKind::Scout => "x",
            RobotKind::Collector => "o",
        }
    }

    fn color(self) -> Color {
        match self.kind {
            RobotKind::Scout => Color::Red,
            RobotKind::Collector => Color::LightMagenta,
        }
    }
}

struct SimulationState {
    robots: Vec<Robot>,
    collected_energy: u32,
    collected_crystals: u32,
    tick: u64,
}

impl SimulationState {
    fn new(map: &Map) -> Self {
        let base_x = map.width / 2;
        let base_y = map.height / 2;
        let mut robots = Vec::new();

        for (offset_x, offset_y, kind) in [
            (0, 0, RobotKind::Scout),
            (1, 0, RobotKind::Collector),
            (0, 1, RobotKind::Scout),
            (-1, 0, RobotKind::Collector),
            (0, -1, RobotKind::Scout),
        ] {
            let x = base_x as isize + offset_x;
            let y = base_y as isize + offset_y;
            if x >= 0 && y >= 0 && x < map.width as isize && y < map.height as isize {
                robots.push(Robot {
                    kind,
                    x: x as usize,
                    y: y as usize,
                });
            }
        }

        Self {
            robots,
            collected_energy: 0,
            collected_crystals: 0,
            tick: 0,
        }
    }
}

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, map: &Map) -> io::Result<()> {
    clear_pending_input_events()?;
    let mut simulation = map.clone();
    let mut state = SimulationState::new(map);

    loop {
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

            let map_lines = map_to_lines(&simulation, &state.robots, map_inner);
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
            let stats = build_stats_panel(&state);
            let stats_para = Paragraph::new(stats);
            frame.render_widget(stats_para, stats_inner);
        })?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(event) => {
                    if event.code == KeyCode::Char('q') || event.code == KeyCode::Esc {
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        update_simulation(&mut simulation, &mut state);
    }
}

fn clear_pending_input_events() -> io::Result<()> {
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read()?;
    }
    Ok(())
}

fn update_simulation(map: &mut Map, state: &mut SimulationState) {
    state.tick += 1;

    for robot in &mut state.robots {
        collect_if_on_resource(
            map,
            robot,
            &mut state.collected_energy,
            &mut state.collected_crystals,
        );

        if let Some((target_x, target_y)) = find_nearest_resource(map, robot.x, robot.y) {
            if (robot.x, robot.y) != (target_x, target_y) {
                move_toward(map, robot, target_x, target_y);
            }
        } else if state.tick % 3 == 0 {
            random_step(map, robot);
        }

        collect_if_on_resource(
            map,
            robot,
            &mut state.collected_energy,
            &mut state.collected_crystals,
        );
    }
}

fn collect_if_on_resource(
    map: &mut Map,
    robot: &mut Robot,
    collected_energy: &mut u32,
    collected_crystals: &mut u32,
) {
    let tile = map.get(robot.x, robot.y);
    if let Tile::Resource { kind, amount } = tile {
        let idx = map.index(robot.x, robot.y);
        if amount == 0 {
            map.tiles[idx] = Tile::Empty;
            return;
        }

        let collected = (amount / 8).max(1) as u32;
        let remaining = amount.saturating_sub(collected as u16);

        match kind {
            ResourceType::Energy => {
                *collected_energy += collected;
            }
            ResourceType::Crystal => {
                *collected_crystals += collected;
            }
        }

        if remaining == 0 {
            map.tiles[idx] = Tile::Empty;
        } else {
            map.tiles[idx] = Tile::Resource {
                kind,
                amount: remaining,
            };
        }
    }
}

fn find_nearest_resource(map: &Map, start_x: usize, start_y: usize) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut best_distance = usize::MAX;

    for y in 0..map.height {
        for x in 0..map.width {
            if let Tile::Resource { amount, .. } = map.get(x, y) {
                if amount == 0 {
                    continue;
                }
                let distance = x.abs_diff(start_x) + y.abs_diff(start_y);
                if distance < best_distance {
                    best_distance = distance;
                    best = Some((x, y));
                }
            }
        }
    }

    best
}

fn move_toward(map: &Map, robot: &mut Robot, target_x: usize, target_y: usize) {
    let mut candidates = Vec::new();

    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let next_x = robot.x as isize + dx;
        let next_y = robot.y as isize + dy;

        if next_x < 0 || next_y < 0 {
            continue;
        }

        let next_x = next_x as usize;
        let next_y = next_y as usize;

        if next_x >= map.width || next_y >= map.height {
            continue;
        }

        if !is_walkable(map.get(next_x, next_y)) {
            continue;
        }

        let heuristic = next_x.abs_diff(target_x) + next_y.abs_diff(target_y);
        candidates.push((heuristic, next_x, next_y));
    }

    candidates.sort_by_key(|(heuristic, _, _)| *heuristic);
    if let Some((_, next_x, next_y)) = candidates.first() {
        robot.x = *next_x;
        robot.y = *next_y;
    }
}

fn random_step(map: &Map, robot: &mut Robot) {
    let mut rng = thread_rng();
    let mut candidates = Vec::new();

    for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        let next_x = robot.x as isize + dx;
        let next_y = robot.y as isize + dy;

        if next_x < 0 || next_y < 0 {
            continue;
        }

        let next_x = next_x as usize;
        let next_y = next_y as usize;

        if next_x >= map.width || next_y >= map.height {
            continue;
        }

        if is_walkable(map.get(next_x, next_y)) {
            candidates.push((next_x, next_y));
        }
    }

    if let Some((next_x, next_y)) = candidates.choose(&mut rng) {
        robot.x = *next_x;
        robot.y = *next_y;
    }
}

fn is_walkable(tile: Tile) -> bool {
    !matches!(tile, Tile::Obstacle)
}

fn build_stats_panel(state: &SimulationState) -> Vec<Line<'static>> {
    let scout_count = state
        .robots
        .iter()
        .filter(|robot| matches!(robot.kind, RobotKind::Scout))
        .count();
    let collector_count = state
        .robots
        .iter()
        .filter(|robot| matches!(robot.kind, RobotKind::Collector))
        .count();
    let total = state.collected_energy + state.collected_crystals;

    vec![
        Line::from(vec![Span::styled(
            format!("Énergie récoltée : {}", state.collected_energy),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            format!("Cristaux récoltés : {}", state.collected_crystals),
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
            format!("Tour : {}", state.tick),
            Style::default().fg(Color::Gray),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Légende : E énergie | C cristaux | # base",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(vec![Span::styled(
            "x éclaireur | o collecteur",
            Style::default().fg(Color::LightCyan),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "q / Esc : quitter",
            Style::default().fg(Color::LightCyan),
        )]),
    ]
}

fn map_to_lines(map: &Map, robots: &[Robot], viewport: Rect) -> Vec<Line<'static>> {
    let max_rows = map.height.min(viewport.height as usize);
    let max_cols = map.width.min(viewport.width as usize);
    let mut lines = Vec::with_capacity(max_rows + 1);

    for y in 0..max_rows {
        let mut spans = Vec::with_capacity(max_cols);
        for x in 0..max_cols {
            let robot = robots.iter().find(|robot| robot.x == x && robot.y == y);
            let (symbol, style) = if let Some(robot) = robot {
                (
                    robot.icon(),
                    Style::default()
                        .fg(robot.color())
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                match map.get(x, y) {
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
                }
            };
            spans.push(Span::styled(symbol.to_string(), style));
        }
        lines.push(Line::from(spans));
    }

    lines
}
