pub mod desktop;
pub mod shortcuts;

pub fn log_startup_environment() {
    desktop::log_environment();
}

pub fn setup_desktop_integration(app: &tauri::AppHandle) {
    desktop::setup(app);
}

pub fn handle_second_instance(app: &tauri::AppHandle, args: &[String]) {
    desktop::handle_cli_args(app, args);
}
