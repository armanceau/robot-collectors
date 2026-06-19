mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use my_app::{generate_map, start_simulation};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() {
    if let Err(err) = run() {
        eprintln!("Erreur d'execution: {}", err);
    }
}

fn run() -> io::Result<()> {
    let map = generate_map(130, 40, 20)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let sim = start_simulation(map);
    let app_result = ui::run_app(&mut terminal, sim);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    app_result
}
