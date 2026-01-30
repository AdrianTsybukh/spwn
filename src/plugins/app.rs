use crate::plugin::Plugin;
use crate::plugins::utils::command::run_command;
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
            run_command(clean_input).await
        }, |res| res)
    }
}
