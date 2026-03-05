use crate::app::App;

fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else if index >= len {
        len - 1
    } else {
        index
    }
}

pub fn run(app: &mut App, args: &[String]) -> bool {
    if !args.is_empty() {
        app.status_message = "reload: unexpected arguments".to_string();
        return false;
    }

    let previous_name = app.selected_entry().map(|entry| entry.name.clone());
    let previous_index = app.selected_index;

    app.reload_entries();

    if let Some(name) = previous_name {
        if let Some(index) = app.entries.iter().position(|entry| entry.name == name) {
            app.selected_index = index;
        } else {
            app.selected_index = clamp_index(previous_index, app.entries.len());
        }
    }

    let refreshed = app.entries.iter().filter(|entry| entry.name != "..").count();
    app.status_message = format!("reload: refreshed {} entries", refreshed);
    false
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::app::App;

    use super::run;

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "minimum-viewer-{}-{}-{}",
            prefix,
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn reload_rejects_arguments() {
        let mut app = App::new();
        run(&mut app, &["extra".to_string()]);
        assert_eq!(app.status_message, "reload: unexpected arguments");
    }

    #[test]
    fn reload_updates_entries_and_keeps_selected_name_when_present() {
        let base = unique_temp_dir("reload-selected-name");
        std::fs::create_dir_all(&base).expect("create temp dir");
        std::fs::write(base.join("a.txt"), "a").expect("create a.txt");
        std::fs::write(base.join("b.txt"), "b").expect("create b.txt");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|entry| entry.name == "b.txt")
            .expect("b.txt must exist");

        std::fs::write(base.join("c.txt"), "c").expect("create c.txt");
        run(&mut app, &[]);

        let selected = app.selected_entry().expect("selection must exist");
        assert_eq!(selected.name, "b.txt");
        assert!(app.entries.iter().any(|entry| entry.name == "c.txt"));
        assert!(app.status_message.starts_with("reload: refreshed "));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn reload_falls_back_to_clamped_previous_index_when_selected_name_disappears() {
        let base = unique_temp_dir("reload-fallback-index");
        std::fs::create_dir_all(&base).expect("create temp dir");
        std::fs::write(base.join("a.txt"), "a").expect("create a.txt");
        std::fs::write(base.join("b.txt"), "b").expect("create b.txt");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.selected_index = app
            .entries
            .iter()
            .position(|entry| entry.name == "b.txt")
            .expect("b.txt must exist");

        std::fs::remove_file(base.join("b.txt")).expect("remove b.txt");
        run(&mut app, &[]);

        let selected = app.selected_entry().expect("selection must exist");
        assert_eq!(selected.name, "a.txt");

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn reload_preserves_filter_and_reapplies_to_updated_entries() {
        let base = unique_temp_dir("reload-filter");
        std::fs::create_dir_all(&base).expect("create temp dir");
        std::fs::write(base.join("alpha.txt"), "alpha").expect("create alpha.txt");
        std::fs::write(base.join("beta.txt"), "beta").expect("create beta.txt");

        let mut app = App::new();
        app.current_dir = base.clone();
        app.reload_entries();
        app.filter_input = "alpha".to_string();
        app.apply_entry_filter();
        assert!(app.entries.iter().any(|entry| entry.name == "alpha.txt"));
        assert!(!app.entries.iter().any(|entry| entry.name == "beta.txt"));

        std::fs::write(base.join("alpha2.txt"), "alpha2").expect("create alpha2.txt");
        std::fs::remove_file(base.join("alpha.txt")).expect("remove alpha.txt");
        run(&mut app, &[]);

        assert_eq!(app.filter_input, "alpha");
        assert!(app.entries.iter().any(|entry| entry.name == "alpha2.txt"));
        assert!(!app.entries.iter().any(|entry| entry.name == "beta.txt"));

        let _ = std::fs::remove_dir_all(&base);
    }
}
