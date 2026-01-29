use crate::plugins::{
    apps::{App, AppProvider},
};
use applications::{AppInfo, AppInfoContext};

pub struct WindowsProvider;

impl AppProvider for WindowsProvider {
    fn get_apps(&self) -> Vec<App> {
        let mut ctx = AppInfoContext::new(vec![]);
        ctx.refresh_apps().unwrap();
        ctx
            .get_all_apps()
            .iter()
            .map(|app| App {
                name: app.name.clone(),
                exec_path: app.app_path_exe.clone().unwrap().display().to_string(),
                icon_name: None,
            })
            .collect()
    }
}
