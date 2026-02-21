use std::io::Write;
use std::process::{Command, Stdio};

use crate::app::App;
use crate::command::path::resolve_path;

pub fn run(app: &mut App, args: &[String]) -> bool {
    if args.len() > 1 {
        app.status_message = "yank: too many arguments".to_string();
        return false;
    }

    let path = if args.is_empty() {
        let Some(entry) = app.selected_entry().cloned() else {
            app.status_message = "yank: no selection".to_string();
            return false;
        };
        entry.path
    } else {
        match resolve_path(&app.current_dir, &args[0]) {
            Ok(p) => p,
            Err(err) => {
                app.status_message = format!("yank: {}", err);
                return false;
            }
        }
    };

    let path_str = path.to_string_lossy();
    match copy_to_clipboard(&path_str) {
        Ok(()) => {
            app.status_message = format!("yanked: {}", path_str);
        }
        Err(reason) => {
            app.status_message = format!("yank: {}", reason);
        }
    }
    false
}

fn copy_to_clipboard(text: &str) -> Result<(), &'static str> {
    #[cfg(target_os = "macos")]
    {
        return run_clipboard_tool("pbcopy", &[], text);
    }

    #[cfg(not(target_os = "macos"))]
    {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            if run_clipboard_tool("wl-copy", &[], text).is_ok() {
                return Ok(());
            }
        }
        if run_clipboard_tool("xclip", &["-selection", "clipboard"], text).is_ok() {
            return Ok(());
        }
        run_clipboard_tool("xsel", &["--clipboard", "--input"], text)
    }
}

fn run_clipboard_tool(program: &str, args: &[&str], text: &str) -> Result<(), &'static str> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "clipboard unavailable")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).map_err(|_| "failed to copy")?;
    }

    let status = child.wait().map_err(|_| "failed to copy")?;
    if status.success() {
        Ok(())
    } else {
        Err("failed to copy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;

    #[test]
    fn yank_returns_false_and_sets_no_selection() {
        let mut app = App::new();
        app.entries = vec![];
        app.selected_index = 0;

        let quit = run(&mut app, &[]);

        assert!(!quit);
        assert_eq!(app.status_message, "yank: no selection");
    }

    #[test]
    fn yank_rejects_multiple_args() {
        let mut app = App::new();

        let quit = run(&mut app, &["a".to_string(), "b".to_string()]);

        assert!(!quit);
        assert_eq!(app.status_message, "yank: too many arguments");
    }

    #[test]
    fn clipboard_tool_unavailable_returns_error() {
        let result = run_clipboard_tool("definitely-not-a-real-clipboard-tool-xyz", &[], "test");
        assert_eq!(result, Err("clipboard unavailable"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn yank_file_copies_path_and_sets_yanked_message() {
        let base = std::env::temp_dir().join(format!(
            "minimum-viewer-yank-file-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create base dir");
        let file = base.join("sample.txt");
        std::fs::write(&file, "x").expect("write file");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "sample.txt")
            .expect("sample.txt must be present");

        let quit = run(&mut app, &[]);

        assert!(!quit);
        assert_eq!(app.status_message, format!("yanked: {}", file.display()));

        let _ = std::fs::remove_dir_all(base);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn yank_directory_copies_path_and_sets_yanked_message() {
        let base = std::env::temp_dir().join(format!(
            "minimum-viewer-yank-dir-{}",
            std::process::id()
        ));
        let subdir = base.join("mydir");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&subdir).expect("create subdir");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|e| e.name == "mydir")
            .expect("mydir must be present");

        let quit = run(&mut app, &[]);

        assert!(!quit);
        assert_eq!(
            app.status_message,
            format!("yanked: {}", subdir.display())
        );

        let _ = std::fs::remove_dir_all(base);
    }
}
