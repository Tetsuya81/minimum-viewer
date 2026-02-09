use crate::app::App;

pub fn run(app: &mut App) -> bool {
    app.status_message = "mkdir: not implemented".to_string();
    false
}
