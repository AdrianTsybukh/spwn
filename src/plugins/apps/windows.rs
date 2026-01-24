use crate::plugins::apps::{App, AppProvider};

pub struct WindowsProvider;

impl AppProvider for WindowsProvider {
    fn get_apps(&self) -> Vec<App> {
        todo!()
    }
}
