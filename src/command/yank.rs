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
        if run_clipboard_tool("xsel", &["--clipboard", "--input"], text).is_ok() {
            return Ok(());
        }
        if is_ssh_session() {
            return copy_via_osc52(text);
        }
        Err("clipboard unavailable")
    }
}

#[cfg(not(target_os = "macos"))]
fn is_ssh_session() -> bool {
    is_ssh_session_with_env(|k| std::env::var_os(k))
}

#[cfg(any(not(target_os = "macos"), test))]
fn is_ssh_session_with_env<F>(get_env: F) -> bool
where
    F: for<'a> Fn(&'a str) -> Option<std::ffi::OsString>,
{
    get_env("SSH_CONNECTION").is_some()
        || get_env("SSH_CLIENT").is_some()
        || get_env("SSH_TTY").is_some()
}

#[cfg(not(target_os = "macos"))]
fn copy_via_osc52(text: &str) -> Result<(), &'static str> {
    let encoded = base64_encode(text.as_bytes());
    let seq = format!("\x1b]52;c;{}\x07", encoded);
    std::io::stdout()
        .write_all(seq.as_bytes())
        .map_err(|_| "osc52: write failed")
}

#[cfg(any(not(target_os = "macos"), test))]
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
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

    #[test]
    fn base64_encode_basic() {
        assert_eq!(base64_encode(b"Hello"), "SGVsbG8=");
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn osc52_encodes_path_correctly() {
        let encoded = base64_encode(b"/tmp/foo");
        assert_eq!(encoded, "L3RtcC9mb28=");
    }

    #[test]
    fn ssh_detection_no_vars_returns_false() {
        assert!(!is_ssh_session_with_env(|_| None));
    }

    #[test]
    fn ssh_detection_uses_ssh_connection() {
        assert!(is_ssh_session_with_env(|k| {
            if k == "SSH_CONNECTION" { Some("10.0.0.1 12345 10.0.0.2 22".into()) } else { None }
        }));
    }

    #[test]
    fn ssh_detection_falls_back_to_ssh_client() {
        assert!(is_ssh_session_with_env(|k| {
            if k == "SSH_CLIENT" { Some("10.0.0.1 12345 22".into()) } else { None }
        }));
    }

    #[test]
    fn ssh_detection_falls_back_to_ssh_tty() {
        assert!(is_ssh_session_with_env(|k| {
            if k == "SSH_TTY" { Some("/dev/pts/0".into()) } else { None }
        }));
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
