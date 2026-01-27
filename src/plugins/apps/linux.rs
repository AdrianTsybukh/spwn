use crate::plugins::apps::{App, AppProvider};
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct LinuxProvider;

impl AppProvider for LinuxProvider {
    fn get_apps(&self) -> Vec<App> {
        let mut apps = Vec::new();

        let xdg_data_dirs = env::var("XDG_DATA_DIRS").unwrap_or_else(|_| {
            "/run/current-system/sw/share:~/.nix-profile/share".to_string()
        });

        for dir in xdg_data_dirs.split(':') {
            let app_dir = PathBuf::from(dir).join("applications");
            
            if let Ok(entries) = fs::read_dir(app_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        if let Some(app) = self.parse_desktop_file(&path) {
                            apps.push(app);
                        }
                    }
                }
            }
        }
        
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        apps.dedup_by(|a, b| a.name == b.name);

        apps
    }
}

impl LinuxProvider {
    fn parse_desktop_file(&self, path: &PathBuf) -> Option<App> {
        let content = fs::read_to_string(path).ok()?;
        let mut name = None;
        let mut exec = None;
        let mut icon = None;
        let mut is_hidden = false;

        for line in content.lines() {
            if line.starts_with("Name=") {
                name = Some(line.replace("Name=", ""));
            } else if line.starts_with("Exec=") {
                exec = Some(line.replace("Exec=", "").split_whitespace().next()?.to_string());
            } else if line.starts_with("Icon=") {
                icon = Some(line.replace("Icon=", ""));
            } else if line.starts_with("NoDisplay=true") {
                is_hidden = true;
            }
        }

        if is_hidden { return None; }

        match (name, exec) {
            (Some(n), Some(e)) => Some(App {
                name: n,
                exec_path: e,
                icon_name: icon,
            }),
            _ => None,
        }
    }
}
