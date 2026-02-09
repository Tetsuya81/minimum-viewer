use crate::app::App;

pub fn run(app: &mut App) -> bool {
    app.status_message = "delete: not implemented".to_string();
    false
}
