use iced::Task;

pub trait Plugin {
    fn can_handle(&self, input: &str) -> bool;

    fn execute(&self, input: &str) -> Task<Result<String, String>>;
}
