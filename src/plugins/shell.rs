use crate::plugin::Plugin;
use crate::plugins::utils::command::run_command;
use iced::Task;

pub struct Shell;

impl Plugin for Shell {
    fn can_handle(&self, input: &str) -> bool {
        input.trim().starts_with(">")
    }

    fn execute(&self, input: &str) -> Task<Result<String, String>> {
        let clean_input = input.trim().trim_start_matches('>').trim();

        if clean_input.is_empty() {
            return Task::future(async { Ok(String::new()) });
        }

        let clean_input = clean_input.to_string();

        Task::perform(async move {
            run_command(clean_input).await
        }, |res| res)
    }
}


