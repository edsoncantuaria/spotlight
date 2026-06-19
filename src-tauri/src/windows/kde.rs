use super::fallback::WmctrlManager;
use super::WindowManager;

pub struct KdeWindowManager;

impl WindowManager for KdeWindowManager {
    fn list_windows(&self) -> Vec<super::WindowInfo> {
        WmctrlManager.list_windows()
    }
    fn focus(&self, id: &str) -> Result<(), String> {
        WmctrlManager.focus(id)
    }
    fn move_active(&self, direction: &str) -> Result<(), String> {
        WmctrlManager.move_active(direction)
    }
    fn toggle_fullscreen(&self) -> Result<(), String> {
        WmctrlManager.toggle_fullscreen()
    }
    fn close_active(&self) -> Result<(), String> {
        WmctrlManager.close_active()
    }
    fn minimize_active(&self) -> Result<(), String> {
        WmctrlManager.minimize_active()
    }
}
