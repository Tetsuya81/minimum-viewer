mod app;
mod command;
mod debug_log;
mod ui;

use std::io::{self, stdout, Stdout};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode};
use ui::draw;

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<bool> {
    loop {
        if app.consume_full_redraw_request() {
            terminal.autoresize()?;
            terminal.clear()?;
        }
        app.ensure_selected_owner_group_resolved();
        terminal.draw(|f| draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if app.show_shell_popup || app.show_help_popup {
                        match key.code {
                            KeyCode::Esc | KeyCode::Enter => app.close_active_popup(),
                            _ => {}
                        }
                        continue;
                    }
                    match app.mode {
                        Mode::Filter => match key.code {
                            KeyCode::Esc => app.exit_filter_mode(true),
                            KeyCode::Enter => app.exit_filter_mode(false),
                            KeyCode::Backspace => app.filter_pop_char(),
                            KeyCode::Delete => app.move_to_parent_directory(),
                            KeyCode::Up | KeyCode::Down => {
                                if key.code == KeyCode::Up {
                                    app.move_selection_up();
                                } else {
                                    app.move_selection_down();
                                }
                            }
                            KeyCode::Char(c) => app.filter_push_char(c),
                            _ => {}
                        },
                        Mode::Command => match key.code {
                            KeyCode::Esc => app.exit_command_mode(),
                            KeyCode::Enter => {
                                if app.execute_selected_command() {
                                    return Ok(true);
                                }
                            }
                            KeyCode::Backspace => app.command_pop_char(),
                            KeyCode::Tab => app.command_select_next(),
                            KeyCode::BackTab => app.command_select_prev(),
                            KeyCode::Char(c) => app.command_push_char(c),
                            _ => {}
                        },
                        Mode::Shell => match key.code {
                            KeyCode::Esc => app.exit_shell_mode(),
                            KeyCode::Enter => app.execute_shell_input(),
                            KeyCode::Backspace => app.shell_pop_char(),
                            KeyCode::Char(c) => app.shell_push_char(c),
                            _ => {}
                        },
                        Mode::Browse => match key.code {
                            KeyCode::Char('q') => return Ok(true),
                            KeyCode::Char(':') => app.enter_command_mode(),
                            KeyCode::Char('!') => app.enter_shell_mode(),
                            KeyCode::Char('/') => app.enter_filter_mode(),
                            KeyCode::Char('e') => {
                                command::editor::run(app);
                            }
                            KeyCode::Char('m') => app.toggle_status_bar_expanded(),
                            KeyCode::Delete | KeyCode::Backspace => app.move_to_parent_directory(),
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
    debug_log::log(
        "main.rs:main",
        "app exit",
        std::collections::BTreeMap::from([("quit", quit.to_string())]),
        "H1",
    );
    // #endregion

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if quit {
        std::process::exit(0);
    }
    Ok(())
}
