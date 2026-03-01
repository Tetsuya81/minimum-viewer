mod app;
mod command;
mod config;
mod debug_log;
mod ui;

use std::io::{self, stdout, Stdout};
use std::path::Path;
use std::time::Duration;
use std::{fs, io::ErrorKind};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode};
use ui::draw;

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn build_cd_on_quit_command(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    format!("cd -- {}", shell_single_quote(path_str.as_ref()))
}

fn write_cd_on_quit_file(lastdir_path: &Path, cwd: &Path) -> io::Result<()> {
    if let Some(parent) = lastdir_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(lastdir_path, format!("{}\n", build_cd_on_quit_command(cwd)))
}

fn maybe_write_cd_on_quit_file(
    quit: bool,
    enabled: bool,
    lastdir_path: &Path,
    cwd: &Path,
) -> io::Result<()> {
    if quit && enabled {
        write_cd_on_quit_file(lastdir_path, cwd)?;
    }
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> io::Result<bool> {
    loop {
        if app.consume_full_redraw_request() {
            terminal.autoresize()?;
            terminal.clear()?;
        }
        app.ensure_selected_owner_group_resolved();
        terminal.draw(|f| draw(f, &mut *app))?;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if app.show_delete_confirm {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete_yes(),
                            KeyCode::Char('n')
                            | KeyCode::Char('N')
                            | KeyCode::Esc
                            | KeyCode::Enter => app.confirm_delete_no(),
                            _ => {}
                        }
                        continue;
                    }
                    if app.show_shell_popup {
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
                        Mode::Create => match key.code {
                            KeyCode::Esc => app.exit_create_mode(),
                            KeyCode::Enter => app.execute_create(),
                            KeyCode::Backspace => app.create_pop_char(),
                            KeyCode::Char(c) => app.create_push_char(c),
                            _ => {}
                        },
                        Mode::Help => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.exit_help_mode(),
                            KeyCode::Up | KeyCode::Char('k') => app.help_move_up(),
                            KeyCode::Down | KeyCode::Char('j') => app.help_move_down(),
                            KeyCode::Enter => {
                                if app.execute_help_selection() {
                                    return Ok(true);
                                }
                            }
                            _ => {}
                        },
                        Mode::Browse => match key.code {
                            KeyCode::Char('q') => return Ok(true),
                            KeyCode::Char('n') => app.enter_create_mode(),
                            KeyCode::Char(':') => app.enter_command_mode(),
                            KeyCode::Char('!') => app.enter_shell_mode(),
                            KeyCode::Char('/') => app.enter_filter_mode(),
                            KeyCode::Char('?') => app.enter_help_mode(),
                            KeyCode::Char('e') => {
                                command::editor::run(app);
                            }
                            KeyCode::Char('M') => {
                                command::markdown::run(app);
                            }
                            KeyCode::Char('m') => app.toggle_status_bar_expanded(),
                            KeyCode::Char('y') => {
                                command::yank::run(app, &[]);
                            }
                            KeyCode::Char('d')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                command::delete::run(app, &[]);
                            }
                            KeyCode::Char('r')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                app.enter_command_mode();
                                app.command_input = "rename ".to_string();
                                app.filter_command_candidates();
                            }
                            KeyCode::Char('c')
                                if key.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                command::cp::run(app, &[]);
                            }
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

    if let Ok(lastdir_path) = config::resolve_lastdir_path() {
        if let Err(err) = maybe_write_cd_on_quit_file(
            quit,
            app.config.cd_on_quit,
            &lastdir_path,
            &app.current_dir,
        ) {
            if err.kind() != ErrorKind::NotFound {
                eprintln!("mmv: failed to write lastdir file: {}", err);
            }
        }
    } else if quit && app.config.cd_on_quit {
        eprintln!("mmv: failed to resolve lastdir path");
    }

    if quit {
        std::process::exit(0);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn shell_single_quote_escapes_single_quote() {
        assert_eq!(shell_single_quote("a'b"), "'a'\"'\"'b'");
    }

    #[test]
    fn write_cd_on_quit_file_writes_command() {
        let root = std::env::temp_dir().join(format!(
            "minimum-viewer-lastdir-write-{}",
            std::process::id()
        ));
        let lastdir_path = root.join("nested/lastdir");
        let _ = fs::remove_dir_all(&root);

        write_cd_on_quit_file(&lastdir_path, &PathBuf::from("/tmp/work"))
            .expect("write must succeed");
        let written = fs::read_to_string(&lastdir_path).expect("lastdir file must exist");
        assert_eq!(written, "cd -- '/tmp/work'\n");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn maybe_write_cd_on_quit_file_skips_when_disabled() {
        let root = std::env::temp_dir().join(format!(
            "minimum-viewer-lastdir-disabled-{}",
            std::process::id()
        ));
        let lastdir_path = root.join("lastdir");
        let _ = fs::remove_dir_all(&root);

        maybe_write_cd_on_quit_file(true, false, &lastdir_path, &PathBuf::from("/tmp/work"))
            .expect("write check must succeed");
        assert!(!lastdir_path.exists());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn maybe_write_cd_on_quit_file_skips_when_not_quit() {
        let root = std::env::temp_dir().join(format!(
            "minimum-viewer-lastdir-notquit-{}",
            std::process::id()
        ));
        let lastdir_path = root.join("lastdir");
        let _ = fs::remove_dir_all(&root);

        maybe_write_cd_on_quit_file(false, true, &lastdir_path, &PathBuf::from("/tmp/work"))
            .expect("write check must succeed");
        assert!(!lastdir_path.exists());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn write_cd_on_quit_file_escapes_single_quote() {
        let root = std::env::temp_dir().join(format!(
            "minimum-viewer-lastdir-quote-{}",
            std::process::id()
        ));
        let lastdir_path = root.join("lastdir");
        let _ = fs::remove_dir_all(&root);

        write_cd_on_quit_file(&lastdir_path, &PathBuf::from("/tmp/a'b"))
            .expect("write must succeed");
        let written = fs::read_to_string(&lastdir_path).expect("lastdir file must exist");
        assert_eq!(written, "cd -- '/tmp/a'\"'\"'b'\n");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn build_cd_on_quit_command_handles_single_dash_path() {
        let command = build_cd_on_quit_command(&PathBuf::from("-"));
        assert_eq!(command, "cd -- '-'");
    }

    #[test]
    fn build_cd_on_quit_command_handles_leading_dash_path() {
        let command = build_cd_on_quit_command(&PathBuf::from("-foo"));
        assert_eq!(command, "cd -- '-foo'");
    }
}
