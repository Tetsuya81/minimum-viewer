use std::path::PathBuf;

use crate::app::App;

pub fn run(app: &mut App, args: &[String]) -> bool {
    if args.is_empty() {
        app.status_message = "mkdir: missing directory name".to_string();
        return false;
    }
    if args.len() > 1 {
        app.status_message = "mkdir: too many arguments".to_string();
        return false;
    }

    let dir_name = args[0].trim();
    if dir_name.is_empty() {
        app.status_message = "mkdir: missing directory name".to_string();
        return false;
    }

    let target = PathBuf::from(&app.current_dir).join(dir_name);
    match std::fs::create_dir(&target) {
        Ok(()) => {
            app.reload_entries();
            app.status_message = format!("mkdir: created '{}'", dir_name);
        }
        Err(err) => {
            app.status_message = format!("mkdir: {}: {}", dir_name, err);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::app::App;

    use super::run;

    #[test]
    fn mkdir_requires_argument() {
        let mut app = App::new();
        run(&mut app, &[]);
        assert_eq!(app.status_message, "mkdir: missing directory name");
    }

    #[test]
    fn mkdir_rejects_multiple_arguments() {
        let mut app = App::new();
        run(&mut app, &["a".to_string(), "b".to_string()]);
        assert_eq!(app.status_message, "mkdir: too many arguments");
    }

    #[test]
    fn mkdir_creates_directory() {
        let base =
            std::env::temp_dir().join(format!("minimum-viewer-mkdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("create temp dir");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();

        run(&mut app, &["child".to_string()]);

        assert!(base.join("child").is_dir());
        assert_eq!(app.status_message, "mkdir: created 'child'");

        let _ = std::fs::remove_dir_all(base);
    }
}
