use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn start_file_watcher() {
    thread::spawn(|| {
        let Some(home) = dirs::home_dir() else {
            return;
        };

        let watch_dirs: Vec<PathBuf> = ["Documents", "Downloads", "Desktop"]
            .into_iter()
            .map(|d| home.join(d))
            .filter(|p| p.exists())
            .collect();

        if watch_dirs.is_empty() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        let Ok(mut watcher) = RecommendedWatcher::new(tx, Config::default()) else {
            return;
        };

        for dir in &watch_dirs {
            let _ = watcher.watch(dir, RecursiveMode::Recursive);
        }

        loop {
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(Ok(_event)) => {
                    // Phase 5: incremental cache refresh hook
                }
                Ok(Err(e)) => eprintln!("[spotlight] watcher error: {e}"),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });
}
