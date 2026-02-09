use crate::app::App;

pub fn run(app: &mut App) -> bool {
    app.status_message = format!("commands: {}", super::command_names_csv());
    false
}
