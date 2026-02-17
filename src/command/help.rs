use crate::app::App;

pub fn run(app: &mut App) -> bool {
    app.enter_help_mode();
    false
}
