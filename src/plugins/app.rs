use crate::plugin::Plugin;
use crate::plugins::shell::run_command_logic;
use iced::Task;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn can_handle(&self, _input: &str) -> bool {
        false // treats everything without special prefix as an app
    }

    fn execute(&self, input: &str) -> Task<Result<String, String>> {
        if input.is_empty() {
            return Task::future(async { Ok(String::new()) });
        }

        let clean_input = input.to_string();

        Task::perform(async move {
            run_command_logic(clean_input).await
        }, |res| res)
    }
}
