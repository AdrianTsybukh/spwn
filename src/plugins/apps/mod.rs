#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug)]
pub struct App {
    pub name: String,
    pub exec_path: String,
    pub icon_name: Option<String>,
}

pub trait AppProvider {
    fn get_apps(&self) -> Vec<App>;
}

pub fn get_all_apps() -> Vec<App> {
    #[cfg(target_os = "linux")]
    return linux::LinuxProvider.get_apps();

    #[cfg(target_os = "windows")]
    return windows::WindowsProvider.get_apps();
}
