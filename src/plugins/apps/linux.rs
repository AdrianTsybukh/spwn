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
        let mut in_main_section = false;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_main_section = line == "[Desktop Entry]";

                if !in_main_section && name.is_some() && exec.is_some() {
                    break;
                }
                continue;
            }

            if !in_main_section {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Name" => name = Some(value.trim().to_string()),
                    "Exec" => {
                        if let Some(binary) = value.trim().split_whitespace().next() {
                            exec = Some(binary.to_string());
                        }
                    }
                    "Icon" => icon = Some(value.trim().to_string()),
                    "NoDisplay" => {
                        if value.trim() == "true" {
                            is_hidden = true;
                        }
                    }
                    _ => {}
                }
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
