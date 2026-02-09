mod app;
mod debug_log;
mod ui;

use std::io::{self, stdout, Stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode};
use ui::draw;

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match app.mode {
                        Mode::Command => match key.code {
                            KeyCode::Esc => app.exit_command_mode(),
                            KeyCode::Enter => {
                                if app.execute_selected_command() {
                                    return Ok(true);
                                }
                            }
                            KeyCode::Backspace => app.command_pop_char(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
                            KeyCode::Char(c) => app.command_push_char(c),
                            _ => {}
                        },
                        Mode::Browse => match key.code {
                            KeyCode::Char('q') => return Ok(true),
                            KeyCode::Char(':') => app.enter_command_mode(),
                            KeyCode::Enter => app.open_selected(),
                            KeyCode::Up | KeyCode::Char('k') => app.move_selection_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.move_selection_down(),
                            _ => {}
                        },
                    }
                }
                Event::Resize(_, _) => {
                    terminal.autoresize()?;
                }
                _ => {}
            }
        }
    }
}

fn main() -> io::Result<()> {
    // #region agent log
    debug_log::log(
        "main.rs:main",
        "app start",
        std::collections::BTreeMap::from([("phase", "start".to_string())]),
        "H1",
    );
    // #endregion
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let quit = run_app(&mut terminal, &mut app)?;
    // #region agent log
    debug_log::log("main.rs:main", "app exit", std::collections::BTreeMap::from([("quit", quit.to_string())]), "H1");
    // #endregion

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if quit {
        std::process::exit(0);
    }
    Ok(())
}
