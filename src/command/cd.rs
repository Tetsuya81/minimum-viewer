use std::collections::BTreeMap;

use crate::app::App;

pub fn run(app: &mut App) -> bool {
    if let Some((path, name, is_dir)) = app
        .selected_entry()
        .map(|e| (e.path.clone(), e.name.clone(), e.is_dir))
    {
        if is_dir {
            crate::debug_log::log(
                "command/cd.rs:run",
                "cd target",
                BTreeMap::from([("path", path.to_string_lossy().to_string())]),
                "H4",
            );
            app.current_dir = path;
            app.reload_entries();
            app.selected_index = 0;
            app.status_message = format!("cd: {}", app.current_dir.display());
        } else {
            app.status_message = format!("cd: '{}' is not a directory", name);
        }
    } else {
        app.status_message = "cd: no selection".to_string();
    }
    false
}
