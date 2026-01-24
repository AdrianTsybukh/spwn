use crate::plugins::apps::{App, AppProvider};
use freedesktop_desktop_entry::{default_paths, get_languages_from_env, Iter};

pub struct LinuxProvider;

impl AppProvider for LinuxProvider {
    fn get_apps(&self) -> Vec<App> {
        let locales = get_languages_from_env();

        Iter::new(default_paths())
            .entries(Some(&locales))
            .filter(|entry| !entry.name(&locales).is_none())
            .filter_map(|entry| {
                let exec = entry.exec()?;
                Some(App {
                    name: entry.name(&locales).unwrap_or_default().as_ref().to_string(),
                    exec_path: exec.to_string(),
                    icon_name: entry.icon().map(|val| val.to_string())
                })
            })
            .collect()
    }
}
